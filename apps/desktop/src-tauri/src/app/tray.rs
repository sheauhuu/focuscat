use crate::environment::configs::app::AppConfig;
use std::io::Cursor;
use std::sync::Mutex;
use tauri::{
    tray::{TrayIcon, TrayIconBuilder},
    App, AppHandle, Manager,
};

// MARK: - Setup

pub fn setup(app: &mut App) -> tauri::Result<()> {
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    app.manage(TrayState {
        icon: Tray::setup(app.handle())?,
        minute_digits: Mutex::new(2),
    });
    return Ok(());
}

// MARK: - State

pub struct TrayState {
    icon: TrayIcon<tauri::Wry>,
    minute_digits: Mutex<usize>,
}

impl TrayState {
    /// Update tray with timer display. Pass None to clear.
    pub fn set_timer(app: &AppHandle, seconds: Option<u32>) {
        let Some(state) = app.try_state::<TrayState>() else {
            return;
        };
        let title = timer_title(seconds, &mut state.minute_digits.lock().unwrap());
        // Release the formatting lock before Tauri waits for the main thread
        let _ = state.icon.set_title(Some(&title));
    }
}

fn timer_title(seconds: Option<u32>, minute_digits: &mut usize) -> String {
    let Some(seconds) = seconds else {
        *minute_digits = 2;
        return String::new();
    };
    let minutes = format!("{:02}", seconds / 60);
    *minute_digits = (*minute_digits).max(minutes.len());
    // Figure spaces retain the width when a countdown crosses 100 minutes
    let padding = "\u{2007}".repeat(*minute_digits - minutes.len());
    return format!(" · {padding}{minutes}:{:02}", seconds % 60);
}

// MARK: - Tray

pub struct Tray;

impl Tray {
    pub fn setup(app: &AppHandle) -> tauri::Result<TrayIcon<tauri::Wry>> {
        let icon = Self::load_icon().unwrap_or_else(|e| panic!("Failed to load tray icon: {e}"));

        let tray = TrayIconBuilder::with_id(super::menubar::TRAY_ID)
            .icon(icon)
            .icon_as_template(true)
            .tooltip(AppConfig::tray_tooltip())
            .on_tray_icon_event(|tray, event| match event {
                tauri::tray::TrayIconEvent::Click {
                    button: tauri::tray::MouseButton::Left,
                    button_state: tauri::tray::MouseButtonState::Up,
                    ..
                } => {
                    let app = tray.app_handle();
                    if let Err(error) = super::menubar::toggle(app) {
                        eprintln!("[Tray] Failed to open menu bar panel: {error}");
                    }
                }
                _ => {}
            })
            .build(app)?;

        #[cfg(target_os = "macos")]
        tray.with_inner_tray_icon(|tray| {
            use tauri_nspanel::{objc2::MainThreadMarker, objc2_app_kit::NSFont};

            let Some(button) =
                MainThreadMarker::new().and_then(|mtm| tray.ns_status_item()?.button(mtm))
            else {
                return;
            };

            // Keep the status item width stable as countdown digits change
            let size = button
                .font()
                .map_or(NSFont::systemFontSize(), |font| font.pointSize());
            let font = NSFont::monospacedDigitSystemFontOfSize_weight(size, 0.0);
            button.setFont(Some(&font));
        })?;

        return Ok(tray);
    }

    fn load_icon() -> Result<tauri::image::Image<'static>, String> {
        let icon_bytes = AppConfig::tray_icon_bytes();

        // Decode PNG to RGBA
        let decoder = png::Decoder::new(Cursor::new(icon_bytes));
        let mut reader = decoder
            .read_info()
            .map_err(|e| format!("Failed to decode PNG: {e}"))?;

        let buffer_size = reader.output_buffer_size().unwrap_or(0);
        let mut buf = vec![0u8; buffer_size];
        let info = reader
            .next_frame(&mut buf)
            .map_err(|e| format!("Failed to read PNG frame: {e}"))?;

        // Convert to RGBA if needed
        let rgba = match info.color_type {
            png::ColorType::Rgb => {
                let mut rgba_buf = Vec::with_capacity(buf.len() / 3 * 4);
                for chunk in buf.chunks_exact(3) {
                    rgba_buf.extend_from_slice(chunk);
                    rgba_buf.push(255); // Alpha
                }
                rgba_buf
            }
            png::ColorType::Rgba => buf,
            _ => return Err("Unsupported PNG color type".to_string()),
        };

        return Ok(tauri::image::Image::new_owned(
            rgba,
            info.width,
            info.height,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::timer_title;

    #[test]
    fn countdown_keeps_its_minute_columns_until_cleared() {
        let mut minute_digits = 2;
        assert_eq!(timer_title(Some(6000), &mut minute_digits), " · 100:00");
        assert_eq!(
            timer_title(Some(5999), &mut minute_digits),
            " · \u{2007}99:59"
        );
        assert_eq!(
            timer_title(Some(599), &mut minute_digits),
            " · \u{2007}09:59"
        );
        assert_eq!(timer_title(None, &mut minute_digits), "");
        assert_eq!(timer_title(Some(599), &mut minute_digits), " · 09:59");
    }
}
