use openvr as vr;

#[derive(Default, macros::InterfaceImpl)]
#[interface = "IVROverlayView"]
#[versions(003)]
pub struct OverlayView {
    vtables: Vtables,
}

impl vr::IVROverlayView003_Interface for OverlayView {
    fn IsViewingPermitted(&self, _: vr::VROverlayHandle_t) -> bool {
        crate::warn_unimplemented!("IsViewingPermitted");
        false
    }
    fn PostOverlayEvent(&self, _: vr::VROverlayHandle_t, _: *const vr::VREvent_t) {
        crate::warn_unimplemented!("PostOverlayEvent");
    }
    fn ReleaseOverlayView(&self, _: *mut vr::VROverlayView_t) -> vr::EVROverlayError {
        vr::EVROverlayError::InvalidHandle
    }
    fn AcquireOverlayView(
        &self,
        _: vr::VROverlayHandle_t,
        _: *mut vr::VRNativeDevice_t,
        _: *mut vr::VROverlayView_t,
        _: u32,
    ) -> vr::EVROverlayError {
        crate::warn_unimplemented!("AcquireOverlayView");
        vr::EVROverlayError::PermissionDenied
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vr::IVROverlayView003_Interface;

    #[test]
    fn is_viewing_permitted_is_false() {
        let view = OverlayView::default();
        assert!(!view.IsViewingPermitted(0));
        assert!(!view.IsViewingPermitted(vr::k_ulOverlayHandleInvalid));
        assert!(!view.IsViewingPermitted(0x1234));
    }

    #[test]
    fn post_overlay_event_is_noop() {
        let view = OverlayView::default();
        view.PostOverlayEvent(0x1234, std::ptr::null());

        let event = vr::VREvent_t::default();
        view.PostOverlayEvent(0x1234, &event);
    }

    #[test]
    fn acquire_overlay_view_is_permission_denied() {
        let view = OverlayView::default();

        // nothing is written to the out parameter
        let mut overlay_view = vr::VROverlayView_t::default();
        let mut device = vr::VRNativeDevice_t::default();
        assert_eq!(
            view.AcquireOverlayView(
                0x1234,
                &mut device,
                &mut overlay_view,
                std::mem::size_of::<vr::VROverlayView_t>() as u32
            ),
            vr::EVROverlayError::PermissionDenied
        );
        assert_eq!(overlay_view.overlayHandle, 0);
        assert!(device.handle.is_null());

        // null pointers are accepted
        assert_eq!(
            view.AcquireOverlayView(
                vr::k_ulOverlayHandleInvalid,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0
            ),
            vr::EVROverlayError::PermissionDenied
        );
    }
}
