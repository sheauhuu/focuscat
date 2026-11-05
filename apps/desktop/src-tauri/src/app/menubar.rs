//! Presents the timer in a macOS panel anchored to the menu bar icon.

use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl};
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelBuilder, PanelLevel, StyleMask,
};

use tauri_nspanel::objc2_app_kit::NSScreen;

pub const LABEL: &str = "menubar";
pub const TRAY_ID: &str = "main-tray";

tauri_panel! {
    panel!(MenubarPanel {
        config: {
            can_become_key_window: true,
            can_become_main_window: false,
            is_floating_panel: true
        }
    })
    panel_event!(MenubarPanelEvents {
        window_did_resign_key(notification: &NSNotification) -> (),
        window_should_close(window: &NSWindow) -> Bool
    })
}

pub fn toggle(app: &AppHandle) -> Result<(), String> {
    if let Ok(panel) = app.get_webview_panel(LABEL) {
        if panel.is_visible() {
            panel.hide();
            return Ok(());
        }
    }
    return show(app);
}

pub fn show(app: &AppHandle) -> Result<(), String> {
    prepare(app)?;
    position(app)?;
    app.get_webview_panel(LABEL)
        .map_err(|e| format!("Menu bar panel is unavailable: {e:?}"))?
        .show_and_make_key();
    return Ok(());
}

pub fn prepare(app: &AppHandle) -> Result<(), String> {
    return ensure_panel(app, "/window/main");
}

pub fn show_at_path(app: &AppHandle, path: &str) -> Result<(), String> {
    let existed = app.get_webview_panel(LABEL).is_ok();
    ensure_panel(app, path)?;
    let window = app
        .get_webview_window(LABEL)
        .ok_or("Menu bar webview is unavailable")?;
    if existed {
        super::window::navigate(&window, path).map_err(|e| e.to_string())?;
    }
    return show(app);
}

pub fn hide(app: &AppHandle) {
    if let Ok(panel) = app.get_webview_panel(LABEL) {
        panel.hide();
    }
}

pub fn is_visible(app: &AppHandle) -> bool {
    return app
        .get_webview_panel(LABEL)
        .is_ok_and(|panel| panel.is_visible());
}

fn ensure_panel(app: &AppHandle, path: &str) -> Result<(), String> {
    if app.get_webview_panel(LABEL).is_ok() {
        return Ok(());
    }

    let panel = PanelBuilder::<_, MenubarPanel>::new(app, LABEL)
        .url(WebviewUrl::App(path.into()))
        .title("FocusCat")
        .size(LogicalSize::new(300.0, 400.0).into())
        .level(PanelLevel::Status)
        .style_mask(StyleMask::new().borderless().nonactivating_panel())
        .collection_behavior(
            CollectionBehavior::new()
                .can_join_all_spaces()
                .full_screen_auxiliary()
                .ignores_cycle(),
        )
        .hides_on_deactivate(false)
        .has_shadow(true)
        .transparent(true)
        .corner_radius(16.0)
        .with_window(|builder| {
            builder
                .visible(false)
                .transparent(true)
                .initialization_script("window.__FOCUSCAT_SURFACE__ = 'menubar';")
                .decorations(false)
                .resizable(false)
                .maximizable(false)
                .minimizable(false)
                .skip_taskbar(true)
                .accept_first_mouse(true)
        })
        .build()
        .map_err(|e| e.to_string())?;

    let window = app
        .get_webview_window(LABEL)
        .ok_or("Menu bar webview is unavailable")?;
    let native_window = window.ns_window().map_err(|e| e.to_string())?;
    // SAFETY: Tauri owns the live NSWindow, and panel creation runs on the main thread.
    let material_applied = tauri_nspanel::objc2::exception::catch(|| unsafe {
        focuscat_macos::apply_menubar_material(native_window, 16.0)
    })
    .map_err(|exception| format!("Menu bar material failed: {exception:?}"))?;
    if !material_applied {
        return Err("Failed to apply the menu bar material".into());
    }

    let events = MenubarPanelEvents::new();
    let app_handle = app.clone();
    events.window_did_resign_key(move |_| {
        // Leave icon clicks to the tray handler so the same click cannot close and reopen the panel.
        if !clicking_tray(&app_handle) {
            hide(&app_handle);
        }
    });
    let app_handle = app.clone();
    events.window_should_close(move |_| {
        hide(&app_handle);
        Bool::NO
    });
    panel.set_event_handler(Some(events.as_ref()));
    return Ok(());
}

// AppKit reports both the icon and cursor in screen points. Keep positioning in points
// so moving between displays with different backing scales does not mix coordinate systems.
fn tray_anchor(app: &AppHandle) -> Result<TrayAnchor, String> {
    app.tray_by_id(TRAY_ID)
        .ok_or("Menu bar icon is unavailable")?
        .with_inner_tray_icon(|tray| {
            let mtm = MainThreadMarker::new()?;
            let window = tray.ns_status_item()?.button(mtm)?.window()?;
            let frame = window.frame();
            let primary = NSScreen::screens(mtm).firstObject()?.frame();
            let cursor = NSEvent::mouseLocation();
            let mouse_event = tauri_nspanel::objc2_app_kit::NSApplication::sharedApplication(mtm)
                .currentEvent()
                .is_some_and(|event| {
                    use tauri_nspanel::objc2_app_kit::NSEventType;
                    matches!(
                        event.r#type(),
                        NSEventType::LeftMouseDown | NSEventType::LeftMouseUp
                    )
                });
            Some(TrayAnchor {
                x: frame.origin.x,
                y: primary.origin.y + primary.size.height - frame.origin.y - frame.size.height,
                width: frame.size.width,
                height: frame.size.height,
                clicking: mouse_event
                    && cursor.x >= frame.origin.x
                    && cursor.x <= frame.origin.x + frame.size.width
                    && cursor.y >= frame.origin.y
                    && cursor.y <= frame.origin.y + frame.size.height,
            })
        })
        .map_err(|e| e.to_string())?
        .ok_or("Menu bar icon position is unavailable".into())
}

struct TrayAnchor {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    clicking: bool,
}

fn clicking_tray(app: &AppHandle) -> bool {
    return tray_anchor(app).is_ok_and(|anchor| anchor.clicking);
}

fn position(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(LABEL)
        .ok_or("Menu bar webview is unavailable")?;
    let anchor = tray_anchor(app)?;
    let monitor = window
        .monitor_from_point(
            anchor.x + anchor.width / 2.0,
            anchor.y + anchor.height / 2.0,
        )
        .map_err(|e| e.to_string())?
        .ok_or("Menu bar display is unavailable")?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let size = window
        .inner_size()
        .map_err(|e| e.to_string())?
        .to_logical::<f64>(scale);
    let bounds = monitor.work_area();
    let origin = bounds.position.to_logical::<f64>(monitor.scale_factor());
    let area = bounds.size.to_logical::<f64>(monitor.scale_factor());
    let (x, y) = anchored_position(
        (anchor.x, anchor.y, anchor.width, anchor.height),
        (size.width, size.height),
        (origin.x, origin.y, area.width, area.height),
    );
    return window
        .set_position(LogicalPosition::new(x, y))
        .map_err(|e| e.to_string());
}

fn anchored_position(
    anchor: (f64, f64, f64, f64),
    size: (f64, f64),
    bounds: (f64, f64, f64, f64),
) -> (f64, f64) {
    let x = anchor.0 + (anchor.2 - size.0) / 2.0;
    let y = anchor.1 + anchor.3;
    return (
        x.clamp(bounds.0, (bounds.0 + bounds.2 - size.0).max(bounds.0)),
        y.clamp(bounds.1, (bounds.1 + bounds.3 - size.1).max(bounds.1)),
    );
}

#[cfg(test)]
mod tests {
    use super::anchored_position;

    #[test]
    fn sits_flush_under_menu_bar_and_avoids_display_edges() {
        let bounds = (-1920.0, -200.0, 1920.0, 1080.0);
        assert_eq!(
            anchored_position((-1000.0, -224.0, 20.0, 24.0), (300.0, 500.0), bounds),
            (-1140.0, -200.0)
        );
        assert_eq!(
            anchored_position((-20.0, -224.0, 20.0, 24.0), (300.0, 500.0), bounds),
            (-300.0, -200.0)
        );
        assert_eq!(
            anchored_position((-1920.0, -224.0, 20.0, 24.0), (300.0, 500.0), bounds),
            (-1920.0, -200.0)
        );
    }
}
