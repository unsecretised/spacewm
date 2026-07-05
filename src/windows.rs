pub enum WindowMessages {
    WindowDestroyed,
    WindowCreated,
    WindowResized,
}

#[derive(Debug, Clone)]
pub struct RawWindow {
    window_pid: i32,
    window_id: i32,
    window_title: String,
}

impl RawWindow {
    pub fn get_open_windows() -> Vec<RawWindow> {
        use objc2_core_foundation::{CFDictionary, CFNumber, CFString};
        use objc2_core_graphics::{
            CGWindowListCopyWindowInfo, CGWindowListOption, kCGNullWindowID, kCGWindowLayer,
            kCGWindowName, kCGWindowNumber, kCGWindowOwnerPID,
        };
        use std::ffi::c_void;

        unsafe {
            let list = CGWindowListCopyWindowInfo(
                CGWindowListOption::OptionOnScreenOnly | CGWindowListOption::ExcludeDesktopElements,
                kCGNullWindowID,
            );

            let Some(arr) = list else { return vec![] };
            let count = arr.count();
            let mut windows = Vec::new();

            for i in 0..count {
                let dict_ptr = arr.value_at_index(i);
                if dict_ptr.is_null() {
                    continue;
                }
                let dict = &*(dict_ptr as *const CFDictionary);

                let pid_val = dict.value(&*kCGWindowOwnerPID as *const CFString as *const c_void);
                if pid_val.is_null() {
                    continue;
                }
                let num = &*(pid_val as *const CFNumber);
                let Some(win_pid) = num.as_i32() else {
                    continue;
                };

                // Skip non-zero layer windows (menus, floating panels, etc.)
                let layer_val = dict.value(&*kCGWindowLayer as *const CFString as *const c_void);
                if !layer_val.is_null() {
                    let layer_num = &*(layer_val as *const CFNumber);
                    if let Some(layer) = layer_num.as_i32() {
                        if layer != 0 {
                            continue;
                        }
                    }
                }

                let layer_val = dict.value(&*kCGWindowLayer as *const CFString as *const c_void);
                if !layer_val.is_null() {
                    let layer_num = &*(layer_val as *const CFNumber);
                    if let Some(layer) = layer_num.as_i32() {
                        if layer != 0 {
                            continue;
                        }
                    }
                }

                let id_val = dict.value(&*kCGWindowNumber as *const CFString as *const c_void);
                if id_val.is_null() {
                    continue;
                }
                let id_num = &*(id_val as *const CFNumber);
                let Some(window_id) = id_num.as_i32() else {
                    continue;
                };

                let name_val = dict.value(&*kCGWindowName as *const CFString as *const c_void);
                if name_val.is_null() {
                    continue;
                }
                let name_str = &*(name_val as *const CFString);
                let title = name_str.to_string();
                if title.is_empty() {
                    continue;
                }

                windows.push(RawWindow {
                    window_id,
                    window_title: title,
                    window_pid: win_pid,
                });
            }

            windows
        }
    }

    pub fn resize(&self) {}

    pub fn focus(&self) {
        #[allow(deprecated)]
        use objc2::rc::Retained;
        use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication, NSWorkspace};
        use objc2_application_services::AXUIElement;
        use objc2_core_foundation::{CFArray, CFBoolean, CFString, CFType};
        use std::ptr::NonNull;

        unsafe {
            // First activate the app itself
            let ws = NSWorkspace::sharedWorkspace();
            if let Some(ns_app) =
                ws.runningApplications()
                    .iter()
                    .find(|ns: &Retained<NSRunningApplication>| {
                        ns.processIdentifier() == self.window_pid
                    })
            {
                ns_app.activateWithOptions(NSApplicationActivationOptions::empty());
            }

            // Then find and focus the specific window via AX
            let app = AXUIElement::new_application(self.window_pid);

            let windows_attr = CFString::from_str("AXWindows");
            let mut windows_val: *const CFType = std::ptr::null();
            let error =
                app.copy_attribute_value(&windows_attr, NonNull::new(&mut windows_val).unwrap());

            if error != objc2_application_services::AXError::Success || windows_val.is_null() {
                return;
            }

            let windows_arr = &*(windows_val as *const CFArray);
            let count = windows_arr.count();

            let title_attr = CFString::from_str("AXTitle");
            let raise_action = CFString::from_str("AXRaise");
            let main_attr = CFString::from_str("AXMain");
            let focused_attr = CFString::from_str("AXFocused");
            let lower_title = self.window_title.to_lowercase();

            for i in 0..count {
                let window_ptr = windows_arr.value_at_index(i);
                if window_ptr.is_null() {
                    continue;
                }
                let window = &*(window_ptr as *const AXUIElement);

                let mut win_title_val: *const CFType = std::ptr::null();
                let err = window
                    .copy_attribute_value(&title_attr, NonNull::new(&mut win_title_val).unwrap());

                if err != objc2_application_services::AXError::Success || win_title_val.is_null() {
                    continue;
                }

                let win_title = &*(win_title_val as *const CFString);
                if win_title.to_string().to_lowercase() == lower_title {
                    window.perform_action(&raise_action);
                    let cf_true: &'static CFBoolean = CFBoolean::new(true);
                    let cf_type_ref: &CFType = &*(cf_true as *const CFBoolean as *const CFType);
                    window.set_attribute_value(&main_attr, cf_type_ref);
                    window.set_attribute_value(&focused_attr, cf_type_ref);
                    break;
                }
            }
        }
    }
}
