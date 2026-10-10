use std::ffi::CStr;

use crate::input::legacy::button_mask_from_id;
use crate::{button_mask_from_ids, input::profiles::Property};
use openvr::EVRButtonId::{A, ApplicationMenu, Axis0, Axis1, Axis2, Grip, System};

pub mod hp_motion_controller;
pub mod ms_motion_controller;
pub mod samsung_odyssey_controller;

const TRACKING_SYSTEM_NAME: &CStr = c"holographic";
const SERIAL_NUMBER: Property<&CStr> = Property::PerHand {
    left: c"MRSOURCE0",
    right: c"MRSOURCE1",
};
const OG_OPENVR_CONTROLLER_TYPE: &CStr = c"holographic_controller";
const OG_LEGACY_BUTTONS_MASK: u64 =
    button_mask_from_ids!(System, ApplicationMenu, Grip, Axis0, Axis1, Axis2, A);

#[cfg(test)]
mod tests {
    use crate::input::{profiles::InteractionProfile, tests::Fixture};
    use crate::openxr_data::Hand;
    use openvr as vr;
    use std::ffi::CStr;

    /// Makes the runtime report `P` for the left hand and checks what games see for that controller.
    pub(super) fn verify_left_controller_properties<P: InteractionProfile>(
        controller_type: &CStr,
        model_number: &CStr,
    ) {
        let mut f = Fixture::new();
        let set1 = f.get_action_set_handle(c"/actions/set1");
        f.load_actions(c"actions.json");
        f.set_interaction_profile::<P>(fakexr::UserPath::LeftHand);
        // The runtime only reports the new profile after the next sync
        f.sync(vr::VRActiveActionSet_t {
            ulActionSet: set1,
            ..Default::default()
        });

        let index = f
            .input
            .get_controller_device_index(Hand::Left)
            .expect("no controller was created for the interaction profile");
        let get = |property| {
            f.input
                .get_device_string_tracked_property(index, property)
                .unwrap()
        };

        assert_eq!(
            get(vr::ETrackedDeviceProperty::ControllerType_String).as_c_str(),
            controller_type
        );
        assert_eq!(
            get(vr::ETrackedDeviceProperty::ModelNumber_String).as_c_str(),
            model_number
        );
        assert_eq!(
            get(vr::ETrackedDeviceProperty::TrackingSystemName_String).as_c_str(),
            super::TRACKING_SYSTEM_NAME
        );
    }
}
