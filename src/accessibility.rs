use std::ffi::c_void;
use std::thread;
use std::time::{Duration, Instant};

use log::info;
use objc2::rc::autoreleasepool;
use objc2::runtime::AnyObject;
use objc2::{class, msg_send};

// ── Accessibility ────────────────────────────────────────────────────────────

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
    static kAXTrustedCheckOptionPrompt: *const c_void;
}

// ── Input Monitoring ─────────────────────────────────────────────────────────

/// IOHIDAccessType values (from <IOKit/hid/IOHIDManager.h>)
#[repr(u32)]
enum IOHIDAccessType {
    /// kIOHIDAccessTypeGranted
    Granted = 0,
    // kIOHIDAccessTypeDenied  = 1,
    // kIOHIDAccessTypeUnknown = 2,
}

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    /// Returns the current IOHIDAccessType for the calling process.
    /// Requires macOS 10.15+.
    fn IOHIDCheckAccess(request_type: u32) -> u32;

    /// Prompts the user for the given access type (shows the system dialog).
    /// Returns the resulting IOHIDAccessType.
    fn IOHIDRequestAccess(request_type: u32) -> bool;
}

/// kIOHIDRequestTypeListenEvent — the type used for input monitoring.
/// Value is 1 (see IOHIDManager.h).
const IOHID_REQUEST_TYPE_LISTEN_EVENT: u32 = 1;

// ── Screen Recording ─────────────────────────────────────────────────────────

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

pub fn screen_recording_is_trusted() -> bool {
    unsafe { CGPreflightScreenCaptureAccess() }
}

pub fn ensure_screen_recording_permission() {
    if screen_recording_is_trusted() {
        info!("Screen Recording permission already granted.");
        return;
    }

    info!("Screen Recording permission is not granted; prompting user.");
    unsafe { CGRequestScreenCaptureAccess() };

    let start = Instant::now();
    loop {
        if screen_recording_is_trusted() {
            info!("Screen Recording permission granted.");
            return;
        }
        if start.elapsed() >= AX_POLL_TIMEOUT {
            info!(
                "Sxitch still does not have Screen Recording permission. \
                 Enable it in System Settings > Privacy & Security > Screen Recording, \
                 then restart Sxitch."
            );
            return;
        }
        thread::sleep(AX_POLL_INTERVAL);
    }
}

// ── CoreFoundation ───────────────────────────────────────────────────────────

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFBooleanTrue: *const c_void;
    static kCFBooleanFalse: *const c_void;
}

// ── Shared constants ─────────────────────────────────────────────────────────

const AX_POLL_INTERVAL: Duration = Duration::from_millis(250);
const AX_POLL_TIMEOUT: Duration = Duration::from_secs(30);

// ── Accessibility helpers ────────────────────────────────────────────────────

#[inline]
pub fn ax_is_trusted() -> bool {
    unsafe {
        autoreleasepool(|_| {
            let keys: [*mut AnyObject; 1] = [kAXTrustedCheckOptionPrompt as *mut AnyObject];
            let vals: [*mut AnyObject; 1] = [kCFBooleanFalse as *mut AnyObject];
            let dict: *mut AnyObject = msg_send![
                class!(NSDictionary),
                dictionaryWithObjects: vals.as_ptr(),
                forKeys:              keys.as_ptr(),
                count:                1usize
            ];
            AXIsProcessTrustedWithOptions(dict.cast())
        })
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn prompt_ax_trust_dialog() {
    autoreleasepool(|_| {
        let keys: [*mut AnyObject; 1] = [kAXTrustedCheckOptionPrompt as *mut AnyObject];
        let vals: [*mut AnyObject; 1] = [kCFBooleanTrue as *mut AnyObject];
        let dict: *mut AnyObject = msg_send![
            class!(NSDictionary),
            dictionaryWithObjects: vals.as_ptr(),
            forKeys:              keys.as_ptr(),
            count:                1usize
        ];
        let _ = AXIsProcessTrustedWithOptions(dict.cast());
    });
}

// ── Input Monitoring helpers ─────────────────────────────────────────────────

/// Returns `true` when the process already has input-monitoring permission.
#[inline]
pub fn input_monitoring_is_trusted() -> bool {
    // IOHIDCheckAccess returns kIOHIDAccessTypeGranted (0) when allowed.
    unsafe { IOHIDCheckAccess(IOHID_REQUEST_TYPE_LISTEN_EVENT) == IOHIDAccessType::Granted as u32 }
}

/// Opens the system prompt that asks the user to grant input-monitoring.
/// After the call returns the user may not have acted yet — poll afterwards.
unsafe fn prompt_input_monitoring_dialog() {
    // IOHIDRequestAccess shows the TCC dialog; we ignore the return value
    // because we poll IOHIDCheckAccess ourselves.
    let _ = unsafe { IOHIDRequestAccess(IOHID_REQUEST_TYPE_LISTEN_EVENT) };
}

// ── Public API ───────────────────────────────────────────────────────────────

pub fn ensure_accessibility_permission() {
    if ax_is_trusted() {
        return;
    }

    info!("Accessibility permission is not granted; prompting user.");
    unsafe { prompt_ax_trust_dialog() };

    wait_for_permission(
        ax_is_trusted,
        "Accessibility",
        "System Settings > Privacy & Security > Accessibility",
    );
}

pub fn ensure_input_monitoring_permission() {
    if input_monitoring_is_trusted() {
        return;
    }

    info!("Input Monitoring permission is not granted; prompting user.");
    unsafe { prompt_input_monitoring_dialog() };

    wait_for_permission(
        input_monitoring_is_trusted,
        "Input Monitoring",
        "System Settings > Privacy & Security > Input Monitoring",
    );
}

// ── Internal ─────────────────────────────────────────────────────────────────

/// Generic polling loop shared by both permission kinds.
fn wait_for_permission(check: impl Fn() -> bool, name: &str, settings_path: &str) {
    let start = Instant::now();

    loop {
        if check() {
            info!("{name} permission granted.");
            return;
        }

        if start.elapsed() >= AX_POLL_TIMEOUT {
            break;
        }

        thread::sleep(AX_POLL_INTERVAL);
    }

    println!(
        "Sxitch still does not have {name} permission. \
         Enable it in {settings_path}, then restart Sxitch."
    );

    std::process::exit(1);
}
