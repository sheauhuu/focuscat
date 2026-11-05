use tauri::{
    AppHandle, CloseRequestApi, Manager, PhysicalPosition, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

// MARK: - Window

/// Standard application windows. The menu bar panel lives in `app::menubar`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Window {
    Main,
    Cat,
    Settings,
    Activity,
    Blocker,
}

impl Window {
    /// Unique window label for Tauri.
    pub fn label(&self) -> &'static str {
        return match self {
            Self::Main => "main",
            Self::Cat => "cat",
            Self::Settings => "settings",
            Self::Activity => "activity",
            Self::Blocker => "blocker",
        };
    }

    /// Window title.
    pub fn title(&self) -> &'static str {
        return match self {
            Self::Main => "FocusCat",
            Self::Cat => "FocusCat",
            Self::Settings => "FocusCat Settings",
            Self::Activity => "FocusCat Activity",
            Self::Blocker => "FocusCat",
        };
    }

    /// Route path for the window.
    pub fn path(&self) -> &'static str {
        return match self {
            Self::Main => "/window/main/splash",
            Self::Cat => "/window/cat",
            Self::Settings => "/window/settings",
            Self::Activity => "/window/activity",
            Self::Blocker => "/window/blocker",
        };
    }

    /// Window size (width, height).
    pub fn size(&self) -> (f64, f64) {
        return match self {
            Self::Main => (300.0, 500.0),
            Self::Cat => (170.0, 170.0),
            Self::Settings => (600.0, 450.0),
            Self::Activity => (600.0, 450.0),
            Self::Blocker => (400.0, 300.0),
        };
    }

    /// Minimum window size, if any.
    pub fn min_size(&self) -> Option<(f64, f64)> {
        return match self {
            Self::Main => None,
            Self::Cat => None,
            Self::Settings => Some((500.0, 400.0)),
            Self::Activity => Some((500.0, 400.0)),
            Self::Blocker => None,
        };
    }

    /// Get existing window if it exists.
    pub fn get(&self, app: &AppHandle) -> Option<WebviewWindow> {
        return app.get_webview_window(self.label());
    }

    /// Show window.
    ///
    /// Handles two cases:
    /// - Window exists: Show and focus
    /// - Window doesn't exist: Create, show, and focus
    pub fn show(&self, app: &AppHandle) -> tauri::Result<WebviewWindow> {
        // Reuse existing window if available
        if let Some(window) = self.get(app) {
            window.show()?;
            window.set_focus()?;
            return Ok(window);
        }

        // Create new window
        let window = self.build(app)?;
        window.show()?;
        window.set_focus()?;
        return Ok(window);
    }

    /// Hide window if it exists.
    pub fn hide(&self, app: &AppHandle) -> tauri::Result<()> {
        if let Some(window) = self.get(app) {
            window.hide()?;
        }
        return Ok(());
    }

    /// Navigate an existing window to a specific path without changing visibility.
    pub fn navigate_to_path(&self, app: &AppHandle, path: &str) -> tauri::Result<()> {
        if let Some(window) = self.get(app) {
            navigate(&window, path)?;
        }
        return Ok(());
    }

    /// Show window at a specific path (for dynamic routing).
    ///
    /// Handles two cases:
    /// - Window exists: Show, focus, and navigate via router
    /// - Window doesn't exist: Create with path, show, and focus
    pub fn show_at_path(&self, app: &AppHandle, path: &str) -> tauri::Result<WebviewWindow> {
        // Reuse existing window if available
        if let Some(window) = self.get(app) {
            window.show()?;
            window.set_focus()?;
            navigate(&window, path)?;
            return Ok(window);
        }

        // Create new window
        let window = self.build_at_path(app, path)?;
        window.show()?;
        window.set_focus()?;
        return Ok(window);
    }

    /// Show window centered over the given bounds.
    pub fn show_at_bounds(
        &self,
        app: &AppHandle,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> tauri::Result<WebviewWindow> {
        let window = if let Some(window) = self.get(app) {
            window
        } else {
            self.build(app)?
        };

        let (card_w, card_h) = self.size();
        let center_x = x + (width - card_w) / 2.0;
        let center_y = y + (height - card_h) / 2.0;

        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(
            center_x, center_y,
        )));
        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(
            card_w, card_h,
        )));
        window.show()?;
        window.set_focus()?;
        return Ok(window);
    }

    // MARK: - Build

    fn build(&self, app: &AppHandle) -> tauri::Result<WebviewWindow> {
        return match self {
            Self::Main => self.build_main(app),
            Self::Cat => self.build_cat(app),
            Self::Settings => self.build_settings(app),
            Self::Activity => self.build_activity(app),
            Self::Blocker => self.build_blocker(app),
        };
    }

    /// Build window with a custom path (for dynamic routing).
    fn build_at_path(&self, app: &AppHandle, path: &str) -> tauri::Result<WebviewWindow> {
        return match self {
            Self::Main => self.build_main_at_path(app, path),
            Self::Activity => self.build_activity_at_path(app, path),
            Self::Settings => self.build_settings_at_path(app, path),
            // Other windows don't support dynamic paths, fall back to default
            _ => self.build(app),
        };
    }

    fn build_main(&self, app: &AppHandle) -> tauri::Result<WebviewWindow> {
        return self.build_main_at_path(app, self.path());
    }

    fn build_main_at_path(&self, app: &AppHandle, path: &str) -> tauri::Result<WebviewWindow> {
        let mut builder = self
            .base_builder_at_path(app, path)
            .resizable(false)
            .maximizable(false)
            .minimizable(true)
            .always_on_top(false);

        #[cfg(target_os = "macos")]
        {
            builder = builder
                .decorations(true)
                .title_bar_style(TitleBarStyle::Overlay)
                .hidden_title(true);
        }

        #[cfg(not(target_os = "macos"))]
        {
            builder = builder.decorations(false);
        }

        return builder.build();
    }

    fn build_cat(&self, app: &AppHandle) -> tauri::Result<WebviewWindow> {
        let mut builder = self
            .base_builder(app)
            .resizable(false)
            .maximizable(false)
            .minimizable(false)
            .decorations(false)
            .always_on_top(true)
            .shadow(false)
            .skip_taskbar(true);

        #[cfg(not(feature = "app-store"))]
        {
            builder = builder.transparent(true);
        }

        #[cfg(target_os = "macos")]
        {
            // Allow first click on unfocused window to reach webview (enables drag without pre-focusing)
            builder = builder.accept_first_mouse(true);
        }

        return builder.build();
    }

    fn build_blocker(&self, app: &AppHandle) -> tauri::Result<WebviewWindow> {
        let mut builder = self
            .base_builder(app)
            .resizable(false)
            .maximizable(false)
            .minimizable(false)
            .always_on_top(true)
            .skip_taskbar(true);

        #[cfg(target_os = "macos")]
        {
            builder = builder
                .decorations(true)
                .title_bar_style(TitleBarStyle::Overlay)
                .hidden_title(true);
        }

        #[cfg(not(target_os = "macos"))]
        {
            builder = builder.decorations(false);
        }

        return builder.build();
    }

    fn build_settings(&self, app: &AppHandle) -> tauri::Result<WebviewWindow> {
        return self.build_settings_at_path(app, self.path());
    }

    fn build_settings_at_path(&self, app: &AppHandle, path: &str) -> tauri::Result<WebviewWindow> {
        let (width, height) = self.size();

        let mut builder = self
            .base_builder_at_path(app, path)
            .resizable(true)
            .maximizable(false)
            .minimizable(true)
            .always_on_top(false);

        // Center over main window if available, otherwise center on screen
        if let Some(pos) = Self::position_centered_over(app, Self::Main, width, height) {
            builder = builder.position(pos.x as f64, pos.y as f64);
        } else {
            builder = builder.center();
        }

        #[cfg(target_os = "macos")]
        {
            builder = builder
                .decorations(true)
                .title_bar_style(TitleBarStyle::Overlay)
                .hidden_title(true);
        }

        #[cfg(not(target_os = "macos"))]
        {
            builder = builder.decorations(false);
        }

        return builder.build();
    }

    fn build_activity(&self, app: &AppHandle) -> tauri::Result<WebviewWindow> {
        return self.build_activity_at_path(app, self.path());
    }

    fn build_activity_at_path(&self, app: &AppHandle, path: &str) -> tauri::Result<WebviewWindow> {
        let (width, height) = self.size();

        let mut builder = self
            .base_builder_at_path(app, path)
            .resizable(true)
            .maximizable(false)
            .minimizable(true)
            .always_on_top(false);

        // Center over main window if available, otherwise center on screen
        if let Some(pos) = Self::position_centered_over(app, Self::Main, width, height) {
            builder = builder.position(pos.x as f64, pos.y as f64);
        } else {
            builder = builder.center();
        }

        #[cfg(target_os = "macos")]
        {
            builder = builder
                .decorations(true)
                .title_bar_style(TitleBarStyle::Overlay)
                .hidden_title(true);
        }

        #[cfg(not(target_os = "macos"))]
        {
            builder = builder.decorations(false);
        }

        return builder.build();
    }

    fn base_builder<'a>(
        &self,
        app: &'a AppHandle,
    ) -> WebviewWindowBuilder<'a, tauri::Wry, AppHandle> {
        return self.base_builder_at_path(app, self.path());
    }

    fn base_builder_at_path<'a>(
        &self,
        app: &'a AppHandle,
        path: &str,
    ) -> WebviewWindowBuilder<'a, tauri::Wry, AppHandle> {
        let (width, height) = self.size();

        let mut builder =
            WebviewWindowBuilder::new(app, self.label(), WebviewUrl::App(path.into()))
                .title(self.title())
                .visible(false)
                .inner_size(width, height);

        if let Some((min_w, min_h)) = self.min_size() {
            builder = builder.min_inner_size(min_w, min_h);
        }

        return builder;
    }

    // MARK: - Events

    /// Handle window close request.
    pub fn handle_close(label: &str, window: &tauri::Window, api: &CloseRequestApi) {
        // Main window hides instead of closing (can reopen from tray).
        // Auxiliary windows hide without reopening another surface.
        match label {
            "main" => {
                api.prevent_close();
                let _ = window.hide();
            }
            "settings" | "activity" => {
                api.prevent_close();
                let _ = window.hide();
            }
            "blocker" => {
                api.prevent_close();
                let _ = window.hide();
            }
            "menubar" => {
                api.prevent_close();
                #[cfg(target_os = "macos")]
                super::menubar::hide(window.app_handle());
            }
            _ => {}
        }
    }

    // MARK: - Helpers

    /// Calculate position to center a window over another window.
    fn position_centered_over(
        app: &AppHandle,
        target: Window,
        width: f64,
        height: f64,
    ) -> Option<PhysicalPosition<i32>> {
        let target_window = target.get(app)?;
        let target_pos = target_window.outer_position().ok()?;
        let target_size = target_window.outer_size().ok()?;

        let x = target_pos.x + (target_size.width as i32 - width as i32) / 2;
        let y = target_pos.y + (target_size.height as i32 - height as i32) / 2;

        return Some(PhysicalPosition::new(x, y));
    }
}

/// Navigate now, or let the frontend consume the path when its router is ready.
pub fn navigate(window: &WebviewWindow, path: &str) -> tauri::Result<()> {
    let href = serde_json::to_string(path)?;
    return window.eval(&format!(
        "if (window.__TAURI_ROUTER__) {{ window.__TAURI_ROUTER__.navigate({{ href: {href} }}); }} else {{ window.__TAURI_PENDING_HREF__ = {href}; }}"
    ));
}
