use crate::openxr_data::RealOpenXrData;
use openvr as vr;
use std::sync::Arc;

#[derive(macros::InterfaceImpl)]
#[interface = "IVRChaperone"]
#[versions(004, 003)]
pub struct Chaperone {
    vtables: Vtables,
    openxr: Arc<RealOpenXrData>,
}

impl Chaperone {
    pub fn new(openxr: Arc<RealOpenXrData>) -> Self {
        Self {
            vtables: Default::default(),
            openxr,
        }
    }
}

impl vr::IVRChaperone004_Interface for Chaperone {
    fn ResetZeroPose(&self, origin: vr::ETrackingUniverseOrigin) {
        self.openxr.reset_tracking_space(origin);
    }

    fn ForceBoundsVisible(&self, _: bool) {
        crate::warn_unimplemented!("ForceBoundsVisible");
    }
    fn AreBoundsVisible(&self) -> bool {
        crate::warn_unimplemented!("AreBoundsVisible");
        false
    }
    fn GetBoundsColor(
        &self,
        color_array: *mut vr::HmdColor_t,
        count: std::ffi::c_int,
        _collision_bounds_fade_distance: f32,
        camera_color: *mut vr::HmdColor_t,
    ) {
        crate::warn_unimplemented!("GetBoundsColor");
        if color_array.is_null() || camera_color.is_null() || count <= 0 {
            return;
        }
        let color_array = unsafe { std::slice::from_raw_parts_mut(color_array, count as usize) };
        color_array.fill(vr::HmdColor_t::default());
        unsafe {
            camera_color.write(vr::HmdColor_t::default());
        }
    }
    fn SetSceneColor(&self, _: vr::HmdColor_t) {
        crate::warn_unimplemented!("SetSceneColor");
    }
    fn ReloadInfo(&self) {
        crate::warn_unimplemented!("ReloadInfo");
    }
    fn GetPlayAreaRect(&self, rect: *mut vr::HmdQuad_t) -> bool {
        crate::warn_unimplemented!("GetPlayAreaRect");
        if let Some(rect) = unsafe { rect.as_mut() } {
            *rect = Default::default();
        }
        false
    }
    fn GetPlayAreaSize(&self, size_x: *mut f32, size_z: *mut f32) -> bool {
        crate::warn_unimplemented!("GetPlayAreaSize");
        if let Some(size_x) = unsafe { size_x.as_mut() } {
            *size_x = 1.0;
        }
        if let Some(size_z) = unsafe { size_z.as_mut() } {
            *size_z = 1.0;
        }
        true
    }
    fn GetCalibrationState(&self) -> vr::ChaperoneCalibrationState {
        vr::ChaperoneCalibrationState::OK
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{clientcore::Injector, openxr_data::OpenXrData};
    use std::ptr::null_mut;
    use vr::IVRChaperone004_Interface;

    fn chaperone() -> Chaperone {
        Chaperone::new(Arc::new(OpenXrData::new(&Injector::default()).unwrap()))
    }

    #[test]
    fn play_area_rect() {
        let c = chaperone();
        let mut rect = vr::HmdQuad_t::default();
        rect.vCorners[0].v = [1.0; 3];
        assert!(!c.GetPlayAreaRect(&mut rect));
        assert_eq!(rect.vCorners[0].v, [0.0; 3]);
        // The output is required, but the call must survive without it.
        assert!(!c.GetPlayAreaRect(null_mut()));
    }

    #[test]
    fn play_area_size() {
        let c = chaperone();
        let (mut x, mut z) = (0.0, 0.0);
        assert!(c.GetPlayAreaSize(&mut x, &mut z));
        assert_eq!((x, z), (1.0, 1.0));

        // Each output is optional.
        (x, z) = (0.0, 0.0);
        assert!(c.GetPlayAreaSize(&mut x, null_mut()));
        assert_eq!((x, z), (1.0, 0.0));
        assert!(c.GetPlayAreaSize(null_mut(), &mut z));
        assert_eq!(z, 1.0);
        assert!(c.GetPlayAreaSize(null_mut(), null_mut()));
    }
}
