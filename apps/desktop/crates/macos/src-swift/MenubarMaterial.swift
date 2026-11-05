import AppKit

@MainActor
@_cdecl("macos_apply_menubar_material")
public func applyMenubarMaterial(
    _ rawWindow: UnsafeMutableRawPointer?,
    _ cornerRadius: Double
) -> Bool {
    guard Thread.isMainThread, let rawWindow else { return false }
    let window = Unmanaged<NSWindow>.fromOpaque(rawWindow).takeUnretainedValue()
    guard let content = window.contentView else { return false }

    let material: NSView
    if #available(macOS 26.0, *) {
        let glass = NSGlassEffectView(frame: content.bounds)
        glass.style = .regular
        glass.cornerRadius = cornerRadius
        glass.contentView = NSView(frame: content.bounds)
        material = glass
    } else {
        let blur = NSVisualEffectView(frame: content.bounds)
        blur.material = .popover
        blur.blendingMode = .behindWindow
        blur.state = .active
        material = blur
    }

    material.autoresizingMask = [.width, .height]
    content.wantsLayer = true
    content.layer?.cornerRadius = cornerRadius
    content.layer?.masksToBounds = true
    // Keep WebKit attached to its original window to preserve its visibility observers.
    content.addSubview(material, positioned: .below, relativeTo: nil)
    window.isOpaque = false
    window.backgroundColor = .clear
    return true
}
