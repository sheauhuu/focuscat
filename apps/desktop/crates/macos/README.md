# focuscat-macos

Native macOS bridges used by the desktop app. Swift owns input monitoring permission checks, the menu bar material background, and display sleep assertions.

`apply_menubar_material` adds Liquid Glass behind a live window's existing content on macOS 26 or later and popover vibrancy on earlier systems. Call it once on the main thread after creating a transparent menu bar panel. The existing content view retains the material as a background subview. WebKit stays attached to its original window.

`DisplaySleepAssertion::acquire` requests `PreventUserIdleDisplaySleep` through IOKit. The Rust guard releases the assertion on explicit `release` or drop. Failed creation and release return IOKit errors. Process termination also releases the assertion. Non-macOS acquisition returns an unsupported-platform error.
