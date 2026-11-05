import Foundation
import IOKit.pwr_mgt

@_cdecl("macos_create_display_sleep_assertion")
public func createDisplaySleepAssertion(_ assertionID: UnsafeMutablePointer<UInt32>?) -> Int32 {
    guard let assertionID else { return kIOReturnBadArgument }

    return IOPMAssertionCreateWithName(
        kIOPMAssertionTypePreventUserIdleDisplaySleep as CFString,
        IOPMAssertionLevel(kIOPMAssertionLevelOn),
        "FocusCat is keeping the screen awake" as CFString,
        assertionID
    )
}

@_cdecl("macos_release_display_sleep_assertion")
public func releaseDisplaySleepAssertion(_ assertionID: UInt32) -> Int32 {
    return IOPMAssertionRelease(assertionID)
}
