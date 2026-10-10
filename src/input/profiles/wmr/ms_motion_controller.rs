use super::super::{
    DynInputPath, InputToXrPath, InteractionProfile, MainAxisType, ProfileProperties, Property,
    SkeletalInputBindings, legal_paths, paths::*,
};
use crate::input::legacy::{self, LegacyBindings};
use crate::input::profiles::wmr;
use crate::openxr_data::Hand;
use glam::Mat4;
use glam::Vec3;

pub struct HolographicController;

impl InteractionProfile for HolographicController {
    type LegalPaths = legal_paths![
        Both::<
            (Menu, Click),
            (Squeeze, Click),
            (Trigger, Value),
            (Thumbstick, ()),
            (Thumbstick, Click),
            (Trackpad, ()),
            (Trackpad, Click),
            (Trackpad, Touch),
        >
    ];

    fn properties() -> &'static ProfileProperties {
        static DEVICE_PROPERTIES: ProfileProperties = ProfileProperties {
            model: Property::PerHand {
                left:c"WindowsMR: 0x045E/0x065B/0/1",
                right: c"WindowsMR: 0x045E/0x065B/0/2",
            },
            openvr_controller_type: wmr::OG_OPENVR_CONTROLLER_TYPE,
            render_model_name: Property::PerHand {
                left: c"C:\\Users\\steamuser\\AppData\\Local\\Microsoft\\Windows\\OpenVR\\controller_1627_1118_1\\controller.obj",
                right: c"C:\\Users\\steamuser\\AppData\\Local\\Microsoft\\Windows\\OpenVR\\controller_1627_1118_2\\controller.obj"
            },
            main_axis: MainAxisType::Thumbstick,
            // The official driver doesn't seem return any thing for this?
            registered_device_type: Property::PerHand {
                left: c"WindowsMR: 0x045E/0x065B/0/1",
                right: c"WindowsMR: 0x045E/0x065B/0/2",
            },
            serial_number: wmr::SERIAL_NUMBER,
            tracking_system_name: wmr::TRACKING_SYSTEM_NAME,
            manufacturer_name: c"WindowsMR: 0x045E",
            legacy_buttons_mask: wmr::OG_LEGACY_BUTTONS_MASK,
        };
        &DEVICE_PROPERTIES
    }
    fn profile_path() -> &'static str {
        "/interaction_profiles/microsoft/motion_controller"
    }
    fn has_required_extensions(_: &openxr::ExtensionSet) -> bool {
        true
    }
    fn translate_path(path: DynInputPath) -> Option<DynInputPath> {
        match path {
            path @ DynInputPath {
                subpath: DynSubpath::Trigger,
                component: Some(DynComponent::Click),
                ..
            } => Some(path.with_component(DynComponent::Value)),
            path @ DynInputPath {
                subpath: DynSubpath::Squeeze,
                component: Some(DynComponent::Value),
                ..
            } => Some(path.with_component(DynComponent::Click)),
            _ => None,
        }
    }

    fn legacy_bindings(c: &InputToXrPath<Self>) -> LegacyBindings {
        // Bindings mostly from OpenComposite
        // Games that use legacy bindings will typically just not use the thumbstick,
        // but most users would probably prefer to use it, so let's use it.
        // And also, games that use the face button can instead use the trackpad as one big button
        LegacyBindings {
            extra: legacy::Bindings {
                grip_pose: c.pose(),
            },
            trigger: c.leftright::<Trigger, Value, _, _>(),
            trigger_click: c.leftright::<Trigger, Value, _, _>(),
            app_menu: c.leftright::<Menu, Click, _, _>(),
            a: c.leftright::<Trackpad, Click, _, _>(),
            squeeze: c.leftright::<Squeeze, Click, _, _>(),
            squeeze_click: c.leftright::<Squeeze, Click, _, _>(),
            main_xy: c.leftright::<Thumbstick, (), _, _>(),
            main_xy_click: c.leftright::<Thumbstick, Click, _, _>(),
            main_xy_touch: vec![],
            haptic: c.haptics(),
        }
    }

    fn skeletal_input_bindings(c: &InputToXrPath<Self>) -> SkeletalInputBindings {
        SkeletalInputBindings {
            thumb_touch: c.leftright::<Trackpad, Touch, _, _>(),
            index_touch: c.leftright::<Trigger, Value, _, _>(),
            index_curl: c.leftright::<Trigger, Value, _, _>(),
            rest_curl: c.leftright::<Squeeze, Click, _, _>(),
        }
    }

    fn offset_grip_pose(_hand: Hand) -> Mat4 {
        Mat4::from_translation(Vec3::new(
            // From the models found here https://www.microsoft.com/en-us/download/details.aspx?id=56414
            0.0, 0.026310, -0.078693,
        ))
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::{HolographicController, InteractionProfile};
    use crate::input::profiles::wmr::tests::verify_left_controller_properties;
    use crate::input::tests::Fixture;
    use openxr as xr;

    #[test]
    fn verify_bindings() {
        base_verify_bindings(HolographicController::profile_path());
    }

    #[test]
    fn controller_properties() {
        verify_left_controller_properties::<HolographicController>(
            c"holographic_controller",
            c"WindowsMR: 0x045E/0x065B/0/1",
        );
    }

    #[test]
    fn needs_no_extension() {
        assert!(HolographicController::has_required_extensions(
            &openxr::ExtensionSet::default()
        ));
    }

    // Separate so the tests can be reused for the Samsung Odyssey controllers
    pub(crate) fn base_verify_bindings(path: &'static str) {
        let f = Fixture::new();
        f.load_actions(c"actions.json");

        f.verify_bindings::<bool>(
            path,
            c"/actions/set1/in/boolact",
            [
                "/user/hand/left/input/thumbstick/click".into(),
                "/user/hand/right/input/thumbstick/click".into(),
                "/user/hand/left/input/squeeze/click".into(),
                "/user/hand/right/input/squeeze/click".into(),
                "/user/hand/left/input/menu/click".into(),
                "/user/hand/right/input/menu/click".into(),
                "/user/hand/left/input/trackpad/click".into(),
                "/user/hand/right/input/trackpad/click".into(),
                "/user/hand/left/input/trackpad/touch".into(),
                "/user/hand/right/input/trackpad/touch".into(),
            ],
        );

        f.verify_bindings::<f32>(
            path,
            c"/actions/set1/boolact_asfloat",
            [
                "/user/hand/left/input/trigger/value".into(),
                "/user/hand/right/input/trigger/value".into(),
            ],
        );

        f.verify_bindings::<f32>(
            path,
            c"/actions/set1/in/vec1act",
            [
                "/user/hand/left/input/trigger/value".into(),
                "/user/hand/right/input/trigger/value".into(),
            ],
        );

        f.verify_bindings::<xr::Vector2f>(
            path,
            c"/actions/set1/in/vec2act",
            [
                "/user/hand/left/input/trackpad".into(),
                "/user/hand/right/input/trackpad".into(),
                "/user/hand/left/input/thumbstick".into(),
                "/user/hand/right/input/thumbstick".into(),
            ],
        );

        f.verify_bindings::<xr::Haptic>(
            path,
            c"/actions/set1/in/vib",
            [
                "/user/hand/left/output/haptic".into(),
                "/user/hand/right/output/haptic".into(),
            ],
        );
    }
}
