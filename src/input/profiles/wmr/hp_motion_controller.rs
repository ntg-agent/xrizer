use super::super::{
    DynInputPath, InputToXrPath, InteractionProfile, Left, MainAxisType, ProfileProperties,
    Property, Right, SkeletalInputBindings, legal_paths, paths::*,
};
use crate::button_mask_from_ids;
use crate::input::legacy::{self, LegacyBindings, button_mask_from_id};
use crate::input::profiles::wmr;
use crate::openxr_data::Hand;
use glam::Mat4;
use glam::Quat;
use glam::Vec3;
use openvr::EVRButtonId as btn;

pub struct ReverbG2Controller;

impl InteractionProfile for ReverbG2Controller {
    type LegalPaths = legal_paths![
        Both::<
            (Menu, Click),
            (Squeeze, Value),
            (Trigger, Value),
            (Thumbstick, ()),
            (Thumbstick, Click),
        >,
        Left::<(X, Click), (Y, Click)>,
        Right::<(A, Click), (B, Click)>
    ];

    fn properties() -> &'static ProfileProperties {
        static DEVICE_PROPERTIES: ProfileProperties = ProfileProperties {
            model: Property::PerHand {
                left:c"WindowsMR: 0x045E/0x066A/0/1",
                right: c"WindowsMR: 0x045E/0x066A/0/2",
            },
            openvr_controller_type: c"hpmotioncontroller",
            render_model_name: Property::PerHand {
                left: c"C:\\Users\\steamuser\\AppData\\Local\\Microsoft/Windows/OpenVR\\controller_1642_1118_1\\controller.obj",
                right: c"C:\\Users\\steamuser\\AppData\\Local\\Microsoft/Windows/OpenVR\\controller_1642_1118_2\\controller.obj"
            },
            main_axis: MainAxisType::Thumbstick,
            // The official drivers don't seem to return anything for this:
            registered_device_type: Property::PerHand {
                left: c"WindowsMR/MRSOURCE0",
                right: c"WindowsMR/MRSOURCE1",
            },
            serial_number: wmr::SERIAL_NUMBER,
            tracking_system_name: wmr::TRACKING_SYSTEM_NAME,
            manufacturer_name: c"WindowsMR: 0x045E",
            legacy_buttons_mask: button_mask_from_ids!(
                btn::System,
                btn::ApplicationMenu,
                btn::Grip,
                btn::Axis0,
                btn::Axis1,
                btn::Axis2,
                btn::A,
            ),
        };
        &DEVICE_PROPERTIES
    }
    fn profile_path() -> &'static str {
        "/interaction_profiles/hp/mixed_reality_controller"
    }
    fn has_required_extensions(enabled_extensions: &openxr::ExtensionSet) -> bool {
        enabled_extensions.ext_hp_mixed_reality_controller
    }
    fn translate_path(path: DynInputPath) -> Option<DynInputPath> {
        match path {
            path @ DynInputPath {
                subpath: DynSubpath::Trigger | DynSubpath::Squeeze,
                component: Some(DynComponent::Click),
                ..
            } => Some(path.with_component(DynComponent::Value)),
            _ => None,
        }
    }

    fn legacy_bindings(c: &InputToXrPath<Self>) -> LegacyBindings {
        // Bindings mostly from OpenComposite
        LegacyBindings {
            extra: legacy::Bindings {
                grip_pose: c.pose(),
            },
            trigger: c.leftright::<Trigger, Value, _, _>(),
            trigger_click: c.leftright::<Trigger, Value, _, _>(),
            app_menu: c.leftright::<Menu, Click, _, _>(),
            a: [
                c.into::<Left<X, Click>, _>(),
                c.into::<Right<A, Click>, _>(),
            ]
            .concat(),
            squeeze: c.leftright::<Squeeze, Value, _, _>(),
            squeeze_click: c.leftright::<Squeeze, Value, _, _>(),
            main_xy: c.leftright::<Thumbstick, (), _, _>(),
            main_xy_click: c.leftright::<Thumbstick, Click, _, _>(),
            main_xy_touch: vec![],
            haptic: c.haptics(),
        }
    }

    fn skeletal_input_bindings(c: &InputToXrPath<Self>) -> SkeletalInputBindings {
        SkeletalInputBindings {
            thumb_touch: [
                c.leftright::<Thumbstick, Click, _, _>(),
                c.into::<Left<X, Click>, _>(),
                c.into::<Left<Y, Click>, _>(),
                c.into::<Right<A, Click>, _>(),
                c.into::<Right<B, Click>, _>(),
            ]
            .concat(),
            index_touch: c.leftright::<Trigger, Value, _, _>(),
            index_curl: c.leftright::<Trigger, Value, _, _>(),
            rest_curl: c.leftright::<Squeeze, Value, _, _>(),
        }
    }

    fn offset_grip_pose(_: Hand) -> Mat4 {
        // SteamVR's transform of the G2 controller's grip relative to its raw pose, as carried in
        // OpenComposite's Reverb G2 interaction profile. Not verified on hardware.
        Mat4::from_rotation_translation(
            Quat::from_rotation_x((-5.04_f32).to_radians()),
            Vec3::new(0.0, -0.00553, 0.09689),
        )
        .inverse()
    }
}

#[cfg(test)]
mod tests {
    use super::{InteractionProfile, ReverbG2Controller};
    use crate::input::profiles::wmr::tests::verify_left_controller_properties;
    use crate::input::tests::Fixture;
    use openxr as xr;

    #[test]
    fn verify_bindings() {
        let path = ReverbG2Controller::profile_path();
        let f = Fixture::new();
        f.load_actions(c"actions.json");

        f.verify_bindings::<bool>(
            path,
            c"/actions/set1/in/boolact",
            [
                "/user/hand/left/input/thumbstick/click".into(),
                "/user/hand/right/input/thumbstick/click".into(),
                "/user/hand/left/input/menu/click".into(),
                "/user/hand/right/input/menu/click".into(),
                "/user/hand/left/input/x/click".into(),
                "/user/hand/left/input/y/click".into(),
                "/user/hand/right/input/a/click".into(),
                "/user/hand/right/input/b/click".into(),
            ],
        );

        f.verify_bindings::<f32>(
            path,
            c"/actions/set1/boolact_asfloat",
            [
                "/user/hand/left/input/squeeze/value".into(),
                "/user/hand/right/input/squeeze/value".into(),
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

    #[test]
    fn controller_properties() {
        verify_left_controller_properties::<ReverbG2Controller>(
            c"hpmotioncontroller",
            c"WindowsMR: 0x045E/0x066A/0/1",
        );
    }

    #[test]
    fn requires_extension() {
        let mut exts = openxr::ExtensionSet::default();
        assert!(!ReverbG2Controller::has_required_extensions(&exts));
        exts.ext_hp_mixed_reality_controller = true;
        assert!(ReverbG2Controller::has_required_extensions(&exts));
    }
}
