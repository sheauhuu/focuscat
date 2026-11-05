# `@repo/desktop`

On macOS, the menu bar panel is always available alongside the main window and floating cat. Left-click the icon to open or close the full timer panel. Click outside or press Escape to hide it. The panel footer places **Show App** on the left and Keep Screen Awake, History, Settings, and Quit icons on the right. History and Settings open their own windows without hiding the floating cat. Closing those windows does not reopen another surface.

The main window retains its minimize-to-cat control. The panel has no compact mode. Use the panel footer for **Show App** and **Quit**.

The monitor button in the main window header and menu bar footer toggles **Keep Screen Awake**. Both buttons share one native macOS display sleep assertion. Blue indicates it is enabled. It works independently of the timer and remains active while windows are hidden. Turn it off or quit FocusCat to release it. It starts disabled on each launch and does not change the system's sleep settings or override manual sleep.

See [macOS Menu Bar Presentation](../../docs/decisions/macos-menubar-presentation.md) for the native window and timer ownership decisions.

Regenerate frontend command and event bindings without launching the app:

```sh
cargo test -p desktop --lib export_typescript_bindings -- --ignored
```
