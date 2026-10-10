use super::super::{
    DynInputPath, InputToXrPath, InteractionProfile, ProfileProperties, SkeletalInputBindings,
};
use super::ms_motion_controller::HolographicController;
use crate::input::legacy::LegacyBindings;
use crate::input::profiles::{MainAxisType, Property, wmr};
use crate::openxr_data::Hand;
use glam::Mat4;
use glam::Vec3;

pub struct SamsungOdysseyController;

impl InteractionProfile for SamsungOdysseyController {
    // Identical inputs to the original Windows Mixed Reality controllers.
    type LegalPaths = <HolographicController as InteractionProfile>::LegalPaths;

    fn properties() -> &'static ProfileProperties {
        static DEVICE_PROPERTIES: ProfileProperties = ProfileProperties {
            model: Property::PerHand {
                left:c"WindowsMR: 0x04E8/0x065D/0/1",
                right: c"WindowsMR: 0x04E8/0x065D/0/2",
            },
            openvr_controller_type: wmr::OG_OPENVR_CONTROLLER_TYPE,
            render_model_name: Property::PerHand {
                left: c"C:\\Users\\steamuser\\AppData\\Local\\Microsoft/Windows/OpenVR\\controller_1629_1256_1\\controller.obj",
                right: c"C:\\Users\\steamuser\\AppData\\Local\\Microsoft/Windows/OpenVR\\controller_1629_1256_2\\controller.obj"
            },
            main_axis: MainAxisType::Thumbstick,
            // The official driver doesn't seem return any thing for this?
            registered_device_type: Property::PerHand {
                left: c"WindowsMR/MRSOURCE0",
                right: c"WindowsMR/MRSOURCE1",
            },
            serial_number: wmr::SERIAL_NUMBER,
            tracking_system_name: wmr::TRACKING_SYSTEM_NAME,
            manufacturer_name: c"WindowsMR: 0x04E8",
            legacy_buttons_mask: wmr::OG_LEGACY_BUTTONS_MASK,
        };
        &DEVICE_PROPERTIES
    }
    fn profile_path() -> &'static str {
        "/interaction_profiles/samsung/odyssey_controller"
    }
    fn has_required_extensions(enabled_extensions: &openxr::ExtensionSet) -> bool {
        enabled_extensions.ext_samsung_odyssey_controller
    }
    fn translate_path(path: DynInputPath) -> Option<DynInputPath> {
        HolographicController::translate_path(path)
    }

    fn legacy_bindings(c: &InputToXrPath<Self>) -> LegacyBindings {
        HolographicController::legacy_bindings(&InputToXrPath::new(c.instance))
    }

    fn skeletal_input_bindings(c: &InputToXrPath<Self>) -> SkeletalInputBindings {
        HolographicController::skeletal_input_bindings(&InputToXrPath::new(c.instance))
    }

    fn offset_grip_pose(_hand: Hand) -> Mat4 {
        Mat4::from_translation(Vec3::new(
            // From the models found here https://www.microsoft.com/en-us/download/details.aspx?id=56414
            0.0, 0.079738, -0.035449,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{InteractionProfile, SamsungOdysseyController};
    use crate::input::profiles::wmr::{
        ms_motion_controller, tests::verify_left_controller_properties,
    };

    #[test]
    fn verify_bindings() {
        ms_motion_controller::tests::base_verify_bindings(SamsungOdysseyController::profile_path());
    }

    #[test]
    fn controller_properties() {
        // Games only know the Odyssey controllers as holographic controllers
        verify_left_controller_properties::<SamsungOdysseyController>(
            c"holographic_controller",
            c"WindowsMR: 0x04E8/0x065D/0/1",
        );
    }

    #[test]
    fn requires_extension() {
        let mut exts = openxr::ExtensionSet::default();
        assert!(!SamsungOdysseyController::has_required_extensions(&exts));
        exts.ext_samsung_odyssey_controller = true;
        assert!(SamsungOdysseyController::has_required_extensions(&exts));
    }
}
