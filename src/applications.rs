use openvr as vr;
use std::ffi::{CStr, c_char};

use log::info;
use std::process::Command;

#[derive(Default, macros::InterfaceImpl)]
#[interface = "IVRApplications"]
#[versions(008, 007, 006, 005, 004, 003, 002)]
pub struct Applications {
    vtables: Vtables,
}

/// Writes an empty string to `buf`, if there is room for it.
fn write_empty_string(buf: *mut c_char, size: u32) {
    if !buf.is_null() && size > 0 {
        unsafe { buf.write(0) };
    }
}

/// Sets the error out-parameter, if it is non-null.
fn set_error(err: *mut vr::EVRApplicationError, value: vr::EVRApplicationError) {
    if let Some(err) = unsafe { err.as_mut() } {
        *err = value;
    }
}

impl vr::IVRApplications008_Interface for Applications {
    fn GetCurrentSceneProcessId(&self) -> u32 {
        std::process::id()
    }
    fn LaunchInternalProcess(
        &self,
        binary_path: *const c_char,
        arguments: *const c_char,
        working_directory: *const c_char,
    ) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("LaunchInternalProcess");

        if binary_path.is_null() || arguments.is_null() || working_directory.is_null() {
            return vr::EVRApplicationError::InvalidParameter;
        }

        let binary_path = unsafe { std::ffi::CStr::from_ptr(binary_path) }
            .to_str()
            .unwrap_or("")
            .to_owned();
        let arguments = unsafe { std::ffi::CStr::from_ptr(arguments) }
            .to_str()
            .unwrap_or("")
            .to_owned();
        let working_directory = unsafe { std::ffi::CStr::from_ptr(working_directory) }
            .to_str()
            .unwrap_or("")
            .to_owned();

        info!(
            "LaunchInternalProcess called: {:?}, ARGS: {:?}, WD: {:?}",
            binary_path, arguments, working_directory
        );

        let process = Command::new(binary_path)
            .args(arguments.split_whitespace())
            .current_dir(working_directory)
            .spawn();

        match process {
            Ok(_) => vr::EVRApplicationError::None,
            Err(e) => {
                info!("Failed to launch internal process: {}", e);
                vr::EVRApplicationError::LaunchFailed
            }
        }
    }
    fn GetSceneApplicationStateNameFromEnum(
        &self,
        state: vr::EVRSceneApplicationState,
    ) -> *const c_char {
        match state {
            vr::EVRSceneApplicationState::None => c"None".as_ptr(),
            vr::EVRSceneApplicationState::Starting => c"Starting".as_ptr(),
            vr::EVRSceneApplicationState::Quitting => c"Quitting".as_ptr(),
            vr::EVRSceneApplicationState::Running => c"Running".as_ptr(),
            vr::EVRSceneApplicationState::Waiting => c"Waiting".as_ptr(),
        }
    }
    fn PerformApplicationPrelaunchCheck(&self, _: *const c_char) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("PerformApplicationPrelaunchCheck");
        vr::EVRApplicationError::UnknownApplication
    }
    fn GetSceneApplicationState(&self) -> vr::EVRSceneApplicationState {
        vr::EVRSceneApplicationState::Running
    }
    fn GetStartingApplication(&self, buf: *mut c_char, size: u32) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("GetStartingApplication");
        write_empty_string(buf, size);
        vr::EVRApplicationError::NoApplication
    }
    fn GetApplicationLaunchArguments(&self, _: u32, buf: *mut c_char, size: u32) -> u32 {
        crate::warn_unimplemented!("GetApplicationLaunchArguments");
        write_empty_string(buf, size);
        0
    }
    fn GetApplicationsThatSupportMimeType(
        &self,
        _: *const c_char,
        buf: *mut c_char,
        size: u32,
    ) -> u32 {
        crate::warn_unimplemented!("GetApplicationsThatSupportMimeType");
        write_empty_string(buf, size);
        0
    }
    fn GetApplicationSupportedMimeTypes(
        &self,
        _: *const c_char,
        buf: *mut c_char,
        size: u32,
    ) -> bool {
        crate::warn_unimplemented!("GetApplicationSupportedMimeTypes");
        write_empty_string(buf, size);
        false
    }
    fn GetDefaultApplicationForMimeType(
        &self,
        _: *const c_char,
        buf: *mut c_char,
        size: u32,
    ) -> bool {
        crate::warn_unimplemented!("GetDefaultApplicationForMimeType");
        write_empty_string(buf, size);
        false
    }
    fn SetDefaultApplicationForMimeType(
        &self,
        _: *const c_char,
        _: *const c_char,
    ) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("SetDefaultApplicationForMimeType");
        vr::EVRApplicationError::UnknownApplication
    }
    fn GetApplicationAutoLaunch(&self, _: *const c_char) -> bool {
        crate::warn_unimplemented!("GetApplicationAutoLaunch");
        false
    }
    fn SetApplicationAutoLaunch(&self, _: *const c_char, _: bool) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("SetApplicationAutoLaunch");
        vr::EVRApplicationError::UnknownApplication
    }
    fn GetApplicationPropertyUint64(
        &self,
        _: *const c_char,
        _: vr::EVRApplicationProperty,
        err: *mut vr::EVRApplicationError,
    ) -> u64 {
        crate::warn_unimplemented!("GetApplicationPropertyUint64");
        set_error(err, vr::EVRApplicationError::UnknownApplication);
        0
    }
    fn GetApplicationPropertyBool(
        &self,
        _: *const c_char,
        _: vr::EVRApplicationProperty,
        err: *mut vr::EVRApplicationError,
    ) -> bool {
        crate::warn_unimplemented!("GetApplicationPropertyBool");
        set_error(err, vr::EVRApplicationError::UnknownApplication);
        false
    }
    fn GetApplicationPropertyString(
        &self,
        _: *const c_char,
        _: vr::EVRApplicationProperty,
        buf: *mut c_char,
        size: u32,
        err: *mut vr::EVRApplicationError,
    ) -> u32 {
        crate::warn_unimplemented!("GetApplicationPropertyString");
        write_empty_string(buf, size);
        set_error(err, vr::EVRApplicationError::UnknownApplication);
        0
    }
    fn GetApplicationsErrorNameFromEnum(&self, e: vr::EVRApplicationError) -> *const c_char {
        let res: &'static CStr = match e {
            vr::EVRApplicationError::None => c"None",
            vr::EVRApplicationError::AppKeyAlreadyExists => c"AppKeyAlreadyExists",
            vr::EVRApplicationError::NoManifest => c"NoManifest",
            vr::EVRApplicationError::NoApplication => c"NoApplication",
            vr::EVRApplicationError::InvalidIndex => c"InvalidIndex",
            vr::EVRApplicationError::UnknownApplication => c"UnknownApplication",
            vr::EVRApplicationError::IPCFailed => c"IPCFailed",
            vr::EVRApplicationError::ApplicationAlreadyRunning => c"ApplicationAlreadyRunning",
            vr::EVRApplicationError::InvalidManifest => c"InvalidManifest",
            vr::EVRApplicationError::InvalidApplication => c"InvalidApplication",
            vr::EVRApplicationError::LaunchFailed => c"LaunchFailed",
            vr::EVRApplicationError::ApplicationAlreadyStarting => c"ApplicationAlreadyStarting",
            vr::EVRApplicationError::LaunchInProgress => c"LaunchInProgress",
            vr::EVRApplicationError::OldApplicationQuitting => c"OldApplicationQuitting",
            vr::EVRApplicationError::TransitionAborted => c"TransitionAborted",
            vr::EVRApplicationError::IsTemplate => c"IsTemplate",
            vr::EVRApplicationError::SteamVRIsExiting => c"SteamVRIsExiting",
            vr::EVRApplicationError::WaitingForChaperone => c"WaitingForChaperone",
            vr::EVRApplicationError::BufferTooSmall => c"BufferTooSmall",
            vr::EVRApplicationError::PropertyNotSet => c"PropertyNotSet",
            vr::EVRApplicationError::UnknownProperty => c"UnknownProperty",
            vr::EVRApplicationError::InvalidParameter => c"InvalidParameter",
            vr::EVRApplicationError::NotImplemented => c"NotImplemented",
        };
        res.as_ptr()
    }
    fn GetApplicationProcessId(&self, _: *const c_char) -> u32 {
        crate::warn_unimplemented!("GetApplicationProcessId");
        0
    }
    fn IdentifyApplication(&self, _: u32, _: *const c_char) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("IdentifyApplication");
        vr::EVRApplicationError::None
    }
    fn CancelApplicationLaunch(&self, _: *const c_char) -> bool {
        crate::warn_unimplemented!("CancelApplicationLaunch");
        false
    }
    // Nothing is launched by the launch functions below, so they must not return `None`: that
    // would tell the caller that the application was started.
    fn LaunchDashboardOverlay(&self, _: *const c_char) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("LaunchDashboardOverlay");
        vr::EVRApplicationError::UnknownApplication
    }
    fn LaunchApplicationFromMimeType(
        &self,
        _: *const c_char,
        _: *const c_char,
    ) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("LaunchApplicationFromMimeType");
        vr::EVRApplicationError::UnknownApplication
    }
    fn LaunchTemplateApplication(
        &self,
        _: *const c_char,
        _: *const c_char,
        _: *const vr::AppOverrideKeys_t,
        _: u32,
    ) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("LaunchTemplateApplication");
        vr::EVRApplicationError::UnknownApplication
    }
    fn LaunchApplication(&self, _: *const c_char) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("LaunchApplication");
        vr::EVRApplicationError::UnknownApplication
    }
    fn GetApplicationKeyByProcessId(
        &self,
        _: u32,
        buf: *mut c_char,
        size: u32,
    ) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("GetApplicationKeyByProcessId");
        write_empty_string(buf, size);
        vr::EVRApplicationError::UnknownApplication
    }
    fn GetApplicationKeyByIndex(
        &self,
        _: u32,
        buf: *mut c_char,
        size: u32,
    ) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("GetApplicationKeyByIndex");
        write_empty_string(buf, size);
        vr::EVRApplicationError::InvalidIndex
    }
    fn GetApplicationCount(&self) -> u32 {
        crate::warn_unimplemented!("GetApplicationCount");
        0
    }
    fn IsApplicationInstalled(&self, _: *const c_char) -> bool {
        crate::warn_unimplemented!("IsApplicationInstalled");
        false
    }
    fn RemoveApplicationManifest(&self, _: *const c_char) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("RemoveApplicationManifest");
        vr::EVRApplicationError::None
    }
    fn AddApplicationManifest(&self, _: *const c_char, _: bool) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("AddApplicationManifest");
        vr::EVRApplicationError::None
    }

    fn RegisterSubprocess(&self, _: u32) -> vr::EVRApplicationError {
        crate::warn_unimplemented!("RegisterSubprocess");
        vr::EVRApplicationError::None
    }
}

impl vr::IVRApplications006On007 for Applications {
    fn GetTransitionState(&self) -> vr::EVRApplicationTransitionState {
        crate::warn_unimplemented!("GetTransitionState");
        vr::EVRApplicationTransitionState::None
    }
    fn GetApplicationsTransitionStateNameFromEnum(
        &self,
        state: vr::EVRApplicationTransitionState,
    ) -> *const ::std::os::raw::c_char {
        crate::warn_unimplemented!("GetApplicationsTransitionStateNameFromEnum");
        match state {
            vr::EVRApplicationTransitionState::None => c"None".as_ptr(),
            vr::EVRApplicationTransitionState::OldAppQuitSent => c"OldAppQuitSent".as_ptr(),
            vr::EVRApplicationTransitionState::WaitingForExternalLaunch => {
                c"WaitingForExternalLaunch".as_ptr()
            }
            vr::EVRApplicationTransitionState::NewAppLaunched => c"NewAppLaunched".as_ptr(),
        }
    }
    fn IsQuitUserPromptRequested(&self) -> bool {
        crate::warn_unimplemented!("IsQuitUserPromptRequested");
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use std::ptr::{null, null_mut};
    use vr::EVRApplicationError as AE;
    use vr::EVRApplicationProperty as P;
    use vr::EVRSceneApplicationState as S;
    use vr::IVRApplications008_Interface;

    const UNKNOWN: AE = AE::UnknownApplication;
    const KEY: *const c_char = c"app.key".as_ptr();
    const MIME: *const c_char = c"text/plain".as_ptr();

    fn check_name(value: impl Debug, name: *const c_char) {
        let name = unsafe { CStr::from_ptr(name) };
        assert_eq!(name.to_str().unwrap(), format!("{value:?}"));
    }

    /// Calls `f` with a 4-byte buffer filled with 0x7f and checks that it returns `expected` and
    /// clears only the first byte. Also calls it with size 0 (must not write) and a null buffer.
    fn check_buf<R: PartialEq + Debug>(expected: R, mut f: impl FnMut(*mut c_char, u32) -> R) {
        let mut buf = [0x7f; 4];
        assert_eq!(f(buf.as_mut_ptr(), 4), expected);
        assert_eq!(buf, [0, 0x7f, 0x7f, 0x7f]);
        buf = [0x7f; 4];
        f(buf.as_mut_ptr(), 0);
        assert_eq!(buf, [0x7f; 4]);
        f(null_mut(), 4);
    }

    #[test]
    fn scene_state_and_names() {
        let a = Applications::default();
        assert_eq!(a.GetCurrentSceneProcessId(), std::process::id());
        assert_eq!(a.GetSceneApplicationState(), S::Running);
        for s in [S::None, S::Starting, S::Quitting, S::Running, S::Waiting] {
            check_name(s, a.GetSceneApplicationStateNameFromEnum(s));
        }
        for e in [AE::None, UNKNOWN, AE::IPCFailed, AE::NotImplemented] {
            check_name(e, a.GetApplicationsErrorNameFromEnum(e));
        }
    }

    #[test]
    fn out_params_are_cleared_and_errors_set() {
        let a = Applications::default();
        let (p, mut e) = (P::Name_String, [AE::None; 3]);
        check_buf(AE::NoApplication, |b, s| a.GetStartingApplication(b, s));
        check_buf(UNKNOWN, |b, s| a.GetApplicationKeyByProcessId(1, b, s));
        check_buf(AE::InvalidIndex, |b, s| a.GetApplicationKeyByIndex(0, b, s));
        check_buf(0, |b, s| a.GetApplicationLaunchArguments(0, b, s));
        check_buf(0, |b, s| a.GetApplicationsThatSupportMimeType(MIME, b, s));
        check_buf(false, |b, s| a.GetApplicationSupportedMimeTypes(MIME, b, s));
        check_buf(false, |b, s| a.GetDefaultApplicationForMimeType(MIME, b, s));
        check_buf(0, |b, s| {
            a.GetApplicationPropertyString(KEY, p, b, s, &mut e[0])
        });
        assert_eq!(a.GetApplicationPropertyUint64(KEY, p, &mut e[1]), 0);
        assert!(!a.GetApplicationPropertyBool(KEY, p, &mut e[2]));
        assert_eq!(e, [UNKNOWN; 3]);
        // Null error pointers are tolerated.
        assert_eq!(a.GetApplicationPropertyUint64(KEY, p, null_mut()), 0);
        assert!(!a.GetApplicationPropertyBool(KEY, p, null_mut()));
        let n = a.GetApplicationPropertyString(KEY, p, null_mut(), 0, null_mut());
        assert_eq!(n, 0);
    }

    #[test]
    fn launching_stubs_do_not_report_success() {
        // `None` would tell the caller that the application was started.
        let a = Applications::default();
        assert_eq!(a.PerformApplicationPrelaunchCheck(KEY), UNKNOWN);
        assert_eq!(a.LaunchApplication(KEY), UNKNOWN);
        assert_eq!(a.LaunchDashboardOverlay(KEY), UNKNOWN);
        assert_eq!(a.LaunchApplicationFromMimeType(MIME, KEY), UNKNOWN);
        assert_eq!(a.LaunchTemplateApplication(KEY, KEY, null(), 0), UNKNOWN);
        assert_eq!(a.SetDefaultApplicationForMimeType(KEY, MIME), UNKNOWN);
        assert_eq!(a.SetApplicationAutoLaunch(KEY, true), UNKNOWN);
        assert!(!a.GetApplicationAutoLaunch(KEY));
        assert_eq!(a.GetApplicationProcessId(KEY), 0);
        assert!(!a.CancelApplicationLaunch(KEY));
    }
}
