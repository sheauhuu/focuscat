# macOS Menu Bar Presentation

## Decision

Use a dedicated `NSPanel` managed through `tauri-nspanel` for menu bar presentation. Keep the existing Tauri tray icon and reuse the main timer routes inside the panel's WebView.

The panel is a separate native surface with one WebView and a full timer view. It is always available on macOS alongside the main window or floating cat. Settings and Activity remain independent windows.

## Ownership

- `app/menubar.rs` owns the native panel, tray anchoring, focus dismissal, and close behavior
- `app/presentation.rs` routes automatic session flows to the visible panel, the visible main window, or the menu bar when no main window is visible
- `focuscat-macos` owns the Swift material background within the existing Tauri content view
- React routes own page content and user interactions, with presentation-specific window chrome
- The Rust timer runner owns automatic advancement, including requesting setup or rating when user input is required
- The Rust timer owns the selected next work duration for progressive sessions, so changing views cannot lose it

The panel is created hidden at startup alongside the main window. Closing it hides the panel and preserves its current page. Clicking the icon opens or closes it; Escape closes it after nested controls have had an opportunity to handle the key.

## Native Integration

Use a nonactivating panel that can become key for keyboard input, appears alongside full-screen windows, and does not become the application's main window. Keep native operations on the macOS main thread.

Read the status item's native frame through Tauri's inner tray accessor. Center the panel beneath the icon with its top edge flush against the menu bar's bottom edge. Calculate placement in AppKit screen points and constrain it to the display's work area. This avoids mixing the icon's backing scale with the panel's scale on another display. Tauri is constrained to the `2.12` minor series because its inner tray dependency can change between minor releases. The panel dependency is pinned to `2.1.1`.

An icon click can cause the panel to resign key before the tray click callback runs. Let the tray handler process that click to prevent closing and immediately reopening the panel. Other focus changes hide it, including keyboard-driven app changes.

The Swift bridge inserts `NSGlassEffectView` on macOS 26 or later, or `NSVisualEffectView` with the popover material on earlier systems, behind the WebView in Tauri's existing content view. It keeps WebKit attached to its original window because reparenting after panel conversion invalidates WebKit visibility observers. The window and WebView are transparent; React page backgrounds allow the native material through. The native material inherits the window appearance, including application theme overrides, and lets AppKit handle accessibility material preferences.

An initialization script marks the menu bar surface before the first HTML paint. Its stylesheet uses the system font and translucent separators. Wheel fades use alpha masks instead of opaque background overlays so the glass stays visible.

The panel is 300 by 400 points. The dashboard uses content-based spacing, with 24 points between the time display and controls. Standard windows retain their existing layout. Setup and rating pages scroll within the same panel size.

## Interaction

Left-clicking the icon toggles the panel. The icon has no separate context menu; the panel footer provides Show App and Quit. Show App opens the main window and hides the floating cat. The main window retains its minimize-to-cat behavior; the menu bar panel has no compact view or minimize control.

The panel dashboard has a fixed footer with Show App on the left and Keep Screen Awake, History, Settings, and Quit icons on the right. Icon buttons have accessible labels and hover titles. Opening History or Settings hides only the panel and preserves desktop windows. Closing auxiliary windows hides them without reopening another surface.

Keep Screen Awake uses a monitor toggle before History in both the panel footer and main window header. Rust owns one native display sleep assertion and broadcasts changes to both surfaces. The toggle is independent of timer state and window visibility. It starts disabled on launch and releases the assertion when turned off or the app exits.

Session setup and progressive rating stay in the surface where the user started the flow. Returns from focus profile editing carry an explicit main-window or menu-bar target. Automatic flows use the visible panel first, then a visible main window, and otherwise the menu bar. Opening an automatic flow in the panel preserves the floating cat.

No preference controls menu bar availability or presentation. Settings version `0.0.4` removes obsolete window preferences; settings from earlier development builds with those fields still deserialize and omit them when saved. Panel position, visibility, and the current route stay in memory.

## Alternatives

- An ordinary borderless window requires more manual control of activation and Spaces behavior
- `NSPopover` provides native popover semantics but requires additional integration with Tauri's ownership of the WebView and its containing window
- Reconfiguring the desktop window in place couples desktop placement and chrome to menu bar presentation

## Validation

Unit tests cover settings migration, anchoring at display edges, and automatic transitions. Native interaction checks should cover icon toggling, outside clicks, Escape with nested controls, keyboard input, display scaling, full-screen Spaces, and coexistence with the floating cat, and returning from focus profile editing.
