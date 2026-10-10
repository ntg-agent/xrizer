use crate::cstr_arg;
use log::debug;
use openvr as vr;
use openvr::EVRSettingsError;
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_char;

#[derive(Default, macros::InterfaceImpl)]
#[interface = "IVRSettings"]
#[versions(003, 002)]
pub struct Settings {
    vtables: Vtables,
}

/// Sets the error out-parameter, if it is non-null.
fn set_error(error: *mut EVRSettingsError, value: EVRSettingsError) {
    if let Some(error) = unsafe { error.as_mut() } {
        *error = value;
    }
}

/// The section and key of a setting. Null is not a valid section or key.
fn section_and_key<'a>(
    section: *const c_char,
    key: *const c_char,
) -> Option<(Cow<'a, str>, Cow<'a, str>)> {
    let section = unsafe { cstr_arg(section) }?;
    let key = unsafe { cstr_arg(key) }?;
    Some((section.to_string_lossy(), key.to_string_lossy()))
}

impl vr::IVRSettings003_Interface for Settings {
    fn GetSettingsErrorNameFromEnum(&self, error: EVRSettingsError) -> *const c_char {
        #[allow(unreachable_patterns)]
        let error: &'static CStr = match error {
            EVRSettingsError::None => c"",
            EVRSettingsError::IPCFailed => c"IPC Failed",
            EVRSettingsError::WriteFailed => c"Write Failed",
            EVRSettingsError::ReadFailed => c"Read Failed",
            EVRSettingsError::JsonParseFailed => c"JSON Parse Failed",
            EVRSettingsError::UnsetSettingHasNoDefault => c"Unset setting has no default",
            EVRSettingsError::AccessDenied => c"Access denied",
            _ => c"Unknown error",
        };
        error.as_ptr()
    }

    fn SetBool(
        &self,
        section: *const c_char,
        settings_key: *const c_char,
        value: bool,
        error: *mut EVRSettingsError,
    ) {
        let Some((section, key)) = section_and_key(section, settings_key) else {
            return set_error(error, EVRSettingsError::WriteFailed);
        };
        debug!("Setting bool on {section}/{key} to {value}");
        set_error(error, EVRSettingsError::None);
    }

    fn SetInt32(
        &self,
        section: *const c_char,
        settings_key: *const c_char,
        value: i32,
        error: *mut EVRSettingsError,
    ) {
        let Some((section, key)) = section_and_key(section, settings_key) else {
            return set_error(error, EVRSettingsError::WriteFailed);
        };
        debug!("Setting int on {section}/{key} to {value}");
        set_error(error, EVRSettingsError::None);
    }

    fn SetFloat(
        &self,
        section: *const c_char,
        settings_key: *const c_char,
        value: f32,
        error: *mut EVRSettingsError,
    ) {
        let Some((section, key)) = section_and_key(section, settings_key) else {
            return set_error(error, EVRSettingsError::WriteFailed);
        };
        debug!("Setting float on {section}/{key} to {value}");
        set_error(error, EVRSettingsError::None);
    }

    fn SetString(
        &self,
        section: *const c_char,
        settings_key: *const c_char,
        value: *const c_char,
        error: *mut EVRSettingsError,
    ) {
        let setting = section_and_key(section, settings_key);
        let value = unsafe { cstr_arg(value) }.map(CStr::to_string_lossy);
        let (Some((section, key)), Some(value)) = (setting, value) else {
            return set_error(error, EVRSettingsError::WriteFailed);
        };
        debug!("Setting string on {section}/{key} to {value}");
        set_error(error, EVRSettingsError::None);
    }

    fn GetBool(
        &self,
        section: *const c_char,
        settings_key: *const c_char,
        error: *mut EVRSettingsError,
    ) -> bool {
        let Some((section, key)) = section_and_key(section, settings_key) else {
            set_error(error, EVRSettingsError::ReadFailed);
            return false;
        };
        set_error(error, EVRSettingsError::None);
        debug!("Getting bool on {section}/{key}");
        false
    }

    fn GetInt32(
        &self,
        section: *const c_char,
        settings_key: *const c_char,
        error: *mut EVRSettingsError,
    ) -> i32 {
        let Some((section, key)) = section_and_key(section, settings_key) else {
            set_error(error, EVRSettingsError::ReadFailed);
            return 0;
        };
        set_error(error, EVRSettingsError::None);
        debug!("Getting int on {section}/{key}");
        0
    }

    fn GetFloat(
        &self,
        section: *const c_char,
        settings_key: *const c_char,
        error: *mut EVRSettingsError,
    ) -> f32 {
        let Some((section, key)) = section_and_key(section, settings_key) else {
            set_error(error, EVRSettingsError::ReadFailed);
            return 0.0;
        };
        set_error(error, EVRSettingsError::None);
        debug!("Getting float on {section}/{key}");
        0.0
    }

    fn GetString(
        &self,
        section: *const c_char,
        settings_key: *const c_char,
        value: *mut c_char,
        value_len: u32,
        error: *mut EVRSettingsError,
    ) {
        if !value.is_null() && value_len > 0 {
            unsafe {
                *value = 0;
            }
        }
        let Some((section, key)) = section_and_key(section, settings_key) else {
            return set_error(error, EVRSettingsError::ReadFailed);
        };
        set_error(error, EVRSettingsError::None);
        debug!("Getting string on {section}/{key}");
    }

    fn RemoveSection(&self, section: *const c_char, error: *mut EVRSettingsError) {
        let Some(section) = (unsafe { cstr_arg(section) }) else {
            return set_error(error, EVRSettingsError::WriteFailed);
        };
        set_error(error, EVRSettingsError::None);
        debug!("Removing section {}", section.to_string_lossy());
    }

    fn RemoveKeyInSection(
        &self,
        section: *const c_char,
        settings_key: *const c_char,
        error: *mut EVRSettingsError,
    ) {
        let Some((section, key)) = section_and_key(section, settings_key) else {
            return set_error(error, EVRSettingsError::WriteFailed);
        };
        set_error(error, EVRSettingsError::None);
        debug!("Removing {section}/{key}");
    }
}

impl vr::IVRSettings002On003 for Settings {
    fn Sync(&self, _force: bool, error: *mut EVRSettingsError) -> bool {
        set_error(error, EVRSettingsError::None);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use EVRSettingsError as E;
    use std::fmt::Debug;
    use std::ptr::{null, null_mut};
    use vr::IVRSettings003_Interface;

    const SECTION: *const c_char = c"section".as_ptr();
    const KEY: *const c_char = c"key".as_ptr();
    const VALUE: *const c_char = c"value".as_ptr();

    /// Calls `f` with an error out-parameter that starts out as `IPCFailed`, and checks the
    /// result and the error it gets. A null error pointer must be tolerated too.
    #[track_caller]
    fn check<R: PartialEq + Debug>(want: R, want_err: E, f: impl Fn(*mut E) -> R) {
        let mut err = E::IPCFailed;
        assert_eq!(f(&mut err), want);
        assert_eq!(err, want_err);
        assert_eq!(f(null_mut()), want);
    }

    #[test]
    fn valid_arguments() {
        let s = Settings::default();
        check((), E::None, |e| s.SetBool(SECTION, KEY, true, e));
        check((), E::None, |e| s.SetInt32(SECTION, KEY, 1, e));
        check((), E::None, |e| s.SetFloat(SECTION, KEY, 1.0, e));
        check((), E::None, |e| s.SetString(SECTION, KEY, VALUE, e));
        check((), E::None, |e| s.RemoveKeyInSection(SECTION, KEY, e));
        check((), E::None, |e| s.RemoveSection(SECTION, e));
        check(false, E::None, |e| s.GetBool(SECTION, KEY, e));
        check(0, E::None, |e| s.GetInt32(SECTION, KEY, e));
        check(0.0, E::None, |e| s.GetFloat(SECTION, KEY, e));
    }

    #[test]
    fn null_strings_are_errors() {
        let s = Settings::default();
        for (section, key) in [(null(), KEY), (SECTION, null()), (null(), null())] {
            check((), E::WriteFailed, |e| s.SetBool(section, key, true, e));
            check((), E::WriteFailed, |e| s.SetInt32(section, key, 1, e));
            check((), E::WriteFailed, |e| s.SetFloat(section, key, 1.0, e));
            check((), E::WriteFailed, |e| s.SetString(section, key, VALUE, e));
            check((), E::WriteFailed, |e| {
                s.RemoveKeyInSection(section, key, e)
            });
            check(false, E::ReadFailed, |e| s.GetBool(section, key, e));
            check(0, E::ReadFailed, |e| s.GetInt32(section, key, e));
            check(0.0, E::ReadFailed, |e| s.GetFloat(section, key, e));
        }
        check((), E::WriteFailed, |e| s.SetString(SECTION, KEY, null(), e));
        check((), E::WriteFailed, |e| s.RemoveSection(null(), e));
    }

    #[test]
    fn get_string() {
        let s = Settings::default();
        // The result is an empty string even if the arguments are bad.
        for (section, key, want) in [
            (SECTION, KEY, E::None),
            (null(), KEY, E::ReadFailed),
            (SECTION, null(), E::ReadFailed),
            (null(), null(), E::ReadFailed),
        ] {
            let mut buf = [0x7f as c_char; 4];
            let ptr = buf.as_mut_ptr();
            check((), want, |e| s.GetString(section, key, ptr, 4, e));
            assert_eq!(buf, [0, 0x7f, 0x7f, 0x7f]);

            // A null or empty buffer is not written.
            check((), want, |e| s.GetString(section, key, null_mut(), 4, e));
            buf = [0x7f; 4];
            let ptr = buf.as_mut_ptr();
            check((), want, |e| s.GetString(section, key, ptr, 0, e));
            assert_eq!(buf, [0x7f; 4]);
        }
    }
}
