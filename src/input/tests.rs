use super::{
    ActionData, Input, InteractionProfile,
    profiles::{
        knuckles::Knuckles, oculus_touch::OculusTouch, simple_controller::SimpleController,
        vive_controller::ViveWands,
    },
};
use crate::{
    input::ActionKey,
    openxr_data::{FakeCompositor, Hand, OpenXrData},
    vr::{self, IVRInput010_Interface},
};
use fakexr::UserPath::*;
use glam::{Mat4, Quat};
use openxr as xr;
use slotmap::KeyData;
use std::collections::HashSet;
use std::f32::consts::FRAC_PI_4;
use std::ffi::CStr;
use std::sync::{Arc, Barrier};

static ACTIONS_JSONS_DIR: &CStr = unsafe {
    CStr::from_bytes_with_nul_unchecked(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/input_data/\0").as_bytes(),
    )
};

impl std::fmt::Debug for ActionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionData::Bool(_) => f.write_str("InputAction::Bool"),
            ActionData::Vector1 { .. } => f.write_str("InputAction::Float"),
            ActionData::Vector2 { .. } => f.write_str("InputAction::Vector2"),
            ActionData::Pose => f.write_str("InputAction::Pose"),
            ActionData::Skeleton { .. } => f.write_str("InputAction::Skeleton"),
            ActionData::Haptic(_) => f.write_str("InputAction::Haptic"),
        }
    }
}

pub(super) struct Fixture {
    pub input: Arc<Input<FakeCompositor>>,
    pending_profile_change: bool,
    _comp: Arc<FakeCompositor>,
}

pub(super) trait ActionType: xr::ActionTy {
    fn get_xr_action(data: &ActionData) -> Result<xr::sys::Action, String>;
}

macro_rules! impl_action_type {
    ($ty:ty, $err_ty:literal, $pattern:pat => $extract_action:expr) => {
        impl ActionType for $ty {
            fn get_xr_action(data: &ActionData) -> Result<xr::sys::Action, String> {
                match data {
                    $pattern => Ok($extract_action),
                    other => Err(format!("Expected {} action, got {other:?}", $err_ty)),
                }
            }
        }
    };
}

impl_action_type!(bool, "boolean", ActionData::Bool(a) => a.as_raw());
impl_action_type!(f32, "vector1", ActionData::Vector1 { action, .. } => action.as_raw());
impl_action_type!(xr::Vector2f, "vector2", ActionData::Vector2{ action, .. } => action.as_raw());
impl_action_type!(xr::Haptic, "haptic", ActionData::Haptic(a) => a.as_raw());
//impl_action_type!(xr::Posef, "pose", ActionData::Pose { action, .. } => action.as_raw());

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum ExtraActionType {
    Analog,
    GrabTouch,
    GrabForce,
    DpadDirection,
    ToggleAction,
    Double,
}

impl Fixture {
    pub fn new() -> Self {
        crate::init_logging();
        let xr = Arc::new(OpenXrData::new(&crate::clientcore::Injector::default()).unwrap());
        let comp = Arc::new(FakeCompositor::new(&xr));
        xr.compositor.set(Arc::downgrade(&comp));
        let ret = Self {
            input: Input::new(xr.clone()).into(),
            pending_profile_change: false,
            _comp: comp,
        };
        xr.input.set(Arc::downgrade(&ret.input));

        ret
    }

    pub fn load_actions(&self, file: &CStr) {
        let path = &[ACTIONS_JSONS_DIR.to_bytes(), file.to_bytes_with_nul()].concat();
        assert_eq!(
            self.input.SetActionManifestPath(path.as_ptr() as _),
            vr::EVRInputError::None,
            "check manifest path: {}",
            std::str::from_utf8(path).expect("Non utf8 path!")
        );
    }

    fn verify_bindings_core(
        &self,
        action: xr::sys::Action,
        interaction_profile: &str,
        action_name: &CStr,
        action_type: &str,
        mut expected_bindings: HashSet<String>,
    ) {
        let profile = self
            .input
            .openxr
            .instance
            .string_to_path(interaction_profile)
            .unwrap();

        let bindings = fakexr::get_suggested_bindings(action, profile);

        let mut found_bindings = Vec::new();

        for binding in bindings {
            assert!(
                expected_bindings.remove(binding.as_str()) || found_bindings.contains(&binding),
                concat!(
                    "Unexpected binding {} for {} action {:?}\n",
                    "found bindings: {:#?}\n",
                    "remaining bindings: {:#?}"
                ),
                binding,
                action_type,
                action_name,
                found_bindings,
                expected_bindings,
            );

            found_bindings.push(binding);
        }

        assert!(
            expected_bindings.is_empty(),
            "Missing expected bindings for {action_type} action {action_name:?}: {expected_bindings:#?}",
        );
    }

    #[track_caller]
    pub fn verify_bindings<T: ActionType>(
        &self,
        interaction_profile: &str,
        action_name: &CStr,
        expected_bindings: impl Into<HashSet<String>>,
    ) {
        let handle = self.get_action_handle(action_name);
        let action = self.get_action::<T>(handle);

        self.verify_bindings_core(
            action,
            interaction_profile,
            action_name,
            std::any::type_name::<T>(),
            expected_bindings.into(),
        )
    }

    #[track_caller]
    pub fn verify_extra_bindings(
        &self,
        interaction_profile: &str,
        action_name: &CStr,
        extra_action_type: ExtraActionType,
        expected_bindings: impl Into<HashSet<String>>,
    ) {
        let handle = self.get_action_handle(action_name);
        let action = self
            .get_extra_action(handle, extra_action_type)
            .unwrap_or_else(|| panic!("No extra action {extra_action_type:?} for {action_name:?}"));

        self.verify_bindings_core(
            action,
            interaction_profile,
            action_name,
            &format!("{extra_action_type:?}"),
            expected_bindings.into(),
        );
    }

    #[track_caller]
    pub fn verify_no_extra_bindings(
        &self,
        interaction_profile: &str,
        action_name: &CStr,
        extra_action_type: ExtraActionType,
    ) {
        let handle = self.get_action_handle(action_name);
        if let Some(action) = self.get_extra_action(handle, extra_action_type) {
            self.verify_bindings_core(
                action,
                interaction_profile,
                action_name,
                &format!("{extra_action_type:?}"),
                [].into(),
            );
        }
    }
}

impl Fixture {
    pub fn get_action_handle(&self, name: &CStr) -> vr::VRActionHandle_t {
        let mut handle = 0;
        assert_eq!(
            self.input.GetActionHandle(name.as_ptr(), &mut handle),
            vr::EVRInputError::None
        );
        assert_ne!(handle, 0);
        handle
    }

    pub fn get_action_set_handle(&self, name: &CStr) -> vr::VRActionSetHandle_t {
        let mut handle = 0;
        assert_eq!(
            self.input.GetActionSetHandle(name.as_ptr(), &mut handle),
            vr::EVRInputError::None
        );
        assert_ne!(handle, 0);
        handle
    }

    pub fn get_input_source_handle(&self, name: &CStr) -> vr::VRInputValueHandle_t {
        let mut src = 0;
        assert_eq!(
            self.input.GetInputSourceHandle(name.as_ptr(), &mut src),
            vr::EVRInputError::None
        );
        src
    }

    pub fn sync(&mut self, mut active: vr::VRActiveActionSet_t) {
        assert_eq!(
            self.input.UpdateActionState(
                &mut active,
                std::mem::size_of::<vr::VRActiveActionSet_t>() as u32,
                1
            ),
            vr::EVRInputError::None
        );
        if self.pending_profile_change {
            self.input.openxr.poll_events();
            self.pending_profile_change = false;
        }
    }

    #[track_caller]
    pub fn get_action<T: ActionType>(&self, handle: vr::VRActionHandle_t) -> xr::sys::Action {
        let data = self.input.openxr.session_data.get();
        let actions = data
            .input_data
            .get_loaded_actions()
            .expect("Actions aren't loaded");
        let action = actions.try_get_action(handle).unwrap_or_else(|_| {
            let key = ActionKey::from(KeyData::from_ffi(handle));
            panic!(
                "Couldn't find action ({}) for handle ({handle})",
                self.input.action_map.read().unwrap()[key].path
            );
        });

        T::get_xr_action(action).expect("Couldn't get OpenXR handle for action")
    }

    #[track_caller]
    pub fn get_extra_action(
        &self,
        handle: vr::VRActionHandle_t,
        extra_action_type: ExtraActionType,
    ) -> Option<xr::sys::Action> {
        let data = self.input.openxr.session_data.get();
        let actions = data
            .input_data
            .get_loaded_actions()
            .expect("Actions aren't loaded");
        let extras = actions.try_get_extra(handle).ok()?;

        Some(match extra_action_type {
            ExtraActionType::Analog => extras.analog_action.as_ref()?.as_raw(),
            ExtraActionType::GrabTouch => extras.grab_actions.as_ref()?.value_action.as_raw(),
            ExtraActionType::GrabForce => extras.grab_actions.as_ref()?.force_action.as_raw(),
            ExtraActionType::DpadDirection => extras.vector2_action.as_ref()?.as_raw(),
            ExtraActionType::ToggleAction => extras.toggle_action.as_ref()?.as_raw(),
            ExtraActionType::Double => extras.double_action.as_ref()?.as_raw(),
        })
    }

    pub fn get_pose(
        &self,
        handle: vr::VRActionHandle_t,
        restrict: vr::VRInputValueHandle_t,
    ) -> Result<vr::InputPoseActionData_t, vr::EVRInputError> {
        self.input.openxr.poll_events();
        let mut state = Default::default();
        let err = self.input.GetPoseActionDataForNextFrame(
            handle,
            vr::ETrackingUniverseOrigin::Seated,
            &mut state,
            std::mem::size_of_val(&state) as u32,
            restrict,
        );

        if err != vr::EVRInputError::None {
            Err(err)
        } else {
            Ok(state)
        }
    }

    pub fn get_bool_state(
        &self,
        handle: vr::VRActionHandle_t,
    ) -> Result<vr::InputDigitalActionData_t, vr::EVRInputError> {
        self.get_bool_state_hand(handle, 0)
    }

    pub fn get_bool_state_hand(
        &self,
        handle: vr::VRActionHandle_t,
        restrict: vr::VRInputValueHandle_t,
    ) -> Result<vr::InputDigitalActionData_t, vr::EVRInputError> {
        let mut state = Default::default();
        let err = self.input.GetDigitalActionData(
            handle,
            &mut state,
            std::mem::size_of::<vr::InputDigitalActionData_t>() as u32,
            restrict,
        );

        if err != vr::EVRInputError::None {
            Err(err)
        } else {
            Ok(state)
        }
    }

    pub fn set_interaction_profile<P: InteractionProfile>(&mut self, hand: fakexr::UserPath) {
        fakexr::set_interaction_profile(
            self.raw_session(),
            hand,
            self.input
                .openxr
                .instance
                .string_to_path(P::profile_path())
                .unwrap(),
        );
        self.pending_profile_change = true;
    }

    pub fn raw_session(&self) -> xr::sys::Session {
        self.input.openxr.session_data.get().session.as_raw()
    }
}

#[test]
fn unknown_handles() {
    let f = Fixture::new();
    f.load_actions(c"actions.json");

    let handle = f.get_action_handle(c"/actions/set1/in/fakeaction");
    let mut state = Default::default();
    assert_ne!(
        f.input.GetDigitalActionData(
            handle,
            &mut state,
            std::mem::size_of::<vr::InputDigitalActionData_t>() as u32,
            0
        ),
        vr::EVRInputError::None
    );
}

#[test]
fn handles_dont_change_after_load() {
    let f = Fixture::new();

    let set1 = f.get_action_set_handle(c"/actions/set1");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");

    f.load_actions(c"actions.json");

    let set_load = f.get_action_set_handle(c"/actions/set1");
    assert_eq!(set_load, set1);
    let act_load = f.get_action_handle(c"/actions/set1/in/boolact");
    assert_eq!(act_load, boolact);
}

#[test]
fn input_state_flow() {
    let mut f = Fixture::new();

    let set1 = f.get_action_set_handle(c"/actions/set1");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");

    f.load_actions(c"actions.json");

    assert!(
        f.input
            .openxr
            .session_data
            .get()
            .input_data
            .get_legacy_actions()
            .is_none()
    );

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let state = f.get_bool_state(boolact).unwrap();
    assert!(!state.bState);
    assert!(!state.bActive);
    assert!(!state.bChanged);

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let state = f.get_bool_state(boolact).unwrap();
    assert!(!state.bState);
    assert!(!state.bActive);
    assert!(!state.bChanged);

    fakexr::set_action_state(
        f.get_action::<bool>(boolact),
        fakexr::ActionState::Bool(true),
        LeftHand,
    );

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let state = f.get_bool_state(boolact).unwrap();
    assert!(state.bState);
    assert!(state.bActive);
    assert!(state.bChanged);
}

#[test]
fn reload_manifest_on_session_restart() {
    let mut f = Fixture::new();

    let set1 = f.get_action_set_handle(c"/actions/set1");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");

    f.load_actions(c"actions.json");
    f.input.openxr.restart_session();

    fakexr::set_action_state(
        f.get_action::<bool>(boolact),
        fakexr::ActionState::Bool(true),
        LeftHand,
    );
    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let state = f.get_bool_state(boolact).unwrap();
    assert!(state.bState);
    assert!(state.bActive);
}

#[track_caller]
pub fn compare_pose(expected: xr::Posef, actual: xr::Posef) {
    fn float_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < f32::EPSILON
    }
    let epos = expected.position;
    let apos = actual.position;
    assert!(
        float_eq(apos.x, epos.x) && float_eq(apos.y, epos.y) && float_eq(apos.z, epos.z),
        "expected position: {epos:?}\nactual position: {apos:?}"
    );

    let erot = expected.orientation;
    let arot = actual.orientation;
    assert!(
        float_eq(arot.x, erot.x)
            && float_eq(arot.y, erot.y)
            && float_eq(arot.z, erot.z)
            && float_eq(arot.w, erot.w),
        "expected orientation: {erot:?}\nactual orientation: {arot:?}",
    );
}

#[test]
fn raw_pose_waitgetposes_and_skeletal_pose_identical() {
    let mut f = Fixture::new();
    let left_hand = f.get_input_source_handle(c"/user/hand/left");
    let pose_handle = f.get_action_handle(c"/actions/set1/in/pose");
    let skel_handle = f.get_action_handle(c"/actions/set1/in/skellyl");
    f.load_actions(c"actions.json");
    f.set_interaction_profile::<Knuckles>(LeftHand);

    let frame = || {
        f.input.openxr.poll_events();
        f.input.frame_start_update();
    };

    // we need to wait two frames for the controller to be connected.
    frame();
    assert!(
        f.input
            .get_controller_device_index(super::Hand::Left)
            .is_none()
    );
    frame();
    assert!(
        f.input
            .get_controller_device_index(super::Hand::Left)
            .is_some()
    );

    let rot = Quat::from_rotation_x(-FRAC_PI_4);
    let pose = xr::Posef {
        position: xr::Vector3f {
            x: 0.5,
            y: 0.5,
            z: 0.5,
        },
        orientation: xr::Quaternionf {
            x: rot.x,
            y: rot.y,
            z: rot.z,
            w: rot.w,
        },
    };
    fakexr::set_grip(f.raw_session(), LeftHand, pose);
    fakexr::set_aim(f.raw_session(), LeftHand, pose);

    let seated_origin = vr::ETrackingUniverseOrigin::Seated;
    let waitgetposes_pose = f
        .input
        .get_controller_pose(super::Hand::Left, Some(seated_origin));

    let mut raw_pose = vr::InputPoseActionData_t {
        pose: vr::TrackedDevicePose_t {
            eTrackingResult: vr::ETrackingResult::Running_OutOfRange,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut skel_pose = raw_pose;

    let ret = f.input.GetPoseActionDataForNextFrame(
        pose_handle,
        seated_origin,
        &mut raw_pose,
        std::mem::size_of_val(&raw_pose) as u32,
        left_hand,
    );
    assert_eq!(ret, vr::EVRInputError::None);
    compare_pose(
        waitgetposes_pose.unwrap().mDeviceToAbsoluteTracking.into(),
        raw_pose.pose.mDeviceToAbsoluteTracking.into(),
    );

    let ret = f.input.GetPoseActionDataForNextFrame(
        skel_handle,
        seated_origin,
        &mut skel_pose,
        std::mem::size_of_val(&skel_pose) as u32,
        0,
    );
    assert_eq!(ret, vr::EVRInputError::None);

    compare_pose(
        waitgetposes_pose.unwrap().mDeviceToAbsoluteTracking.into(),
        skel_pose.pose.mDeviceToAbsoluteTracking.into(),
    );
}

#[test]
fn actions_with_bad_paths() {
    let mut f = Fixture::new();
    let spaces = f.get_action_handle(c"/actions/set1/in/action with spaces");
    let commas = f.get_action_handle(c"/actions/set1/in/action,with,commas");
    let mixed = f.get_action_handle(c"/actions/set1/in/mixed, action");
    let paren = f.get_action_handle(c"/actions/set1/in/(action)(with)(parenthesis)");
    let brackets = f.get_action_handle(c"/actions/set1/in/[action][with][brackets]");
    let long_bad1 = f.get_action_handle(c"/actions/set1/in/ThisActionHasAReallyLongNameThatIsMostCertainlyLongerThanTheOpenXRLimit,However,ItWillBeGivenASimpleLocalizedName");
    let long_bad2 = f.get_action_handle(c"/actions/set1/in/ThisActionWillAlsoHaveAReallyLongNameAndAShortLocalizedName,MuchLikeThePreviousAction");
    let long_exact = f.get_action_handle(
        c"/actions/set1/in/this right here is an action that is exactly 64 characters long!",
    );

    let set1 = f.get_action_set_handle(c"/actions/set1");
    f.load_actions(c"actions_malformed_paths.json");

    fakexr::set_action_state(
        f.get_action::<bool>(spaces),
        fakexr::ActionState::Bool(true),
        LeftHand,
    );
    fakexr::set_action_state(
        f.get_action::<f32>(commas),
        fakexr::ActionState::Float(0.5),
        LeftHand,
    );
    fakexr::set_action_state(
        f.get_action::<bool>(mixed),
        fakexr::ActionState::Bool(true),
        LeftHand,
    );
    fakexr::set_action_state(
        f.get_action::<bool>(long_bad1),
        fakexr::ActionState::Bool(false),
        LeftHand,
    );
    fakexr::set_action_state(
        f.get_action::<bool>(long_bad2),
        fakexr::ActionState::Bool(false),
        LeftHand,
    );
    fakexr::set_action_state(
        f.get_action::<bool>(long_exact),
        fakexr::ActionState::Bool(false),
        LeftHand,
    );
    fakexr::set_action_state(
        f.get_action::<bool>(paren),
        fakexr::ActionState::Bool(false),
        LeftHand,
    );
    fakexr::set_action_state(
        f.get_action::<bool>(brackets),
        fakexr::ActionState::Bool(false),
        LeftHand,
    );
    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let s = f.get_bool_state(spaces).unwrap();
    assert!(s.bActive);
    assert!(s.bState);
    assert!(s.bChanged);

    let s = f.get_bool_state(mixed).unwrap();
    assert!(s.bActive);
    assert!(s.bState);
    assert!(s.bChanged);

    let s = f.get_bool_state(paren).unwrap();
    assert!(s.bActive);
    assert!(!s.bState);
    assert!(!s.bChanged);

    let s = f.get_bool_state(brackets).unwrap();
    assert!(s.bActive);
    assert!(!s.bState);
    assert!(!s.bChanged);

    let s = f.get_bool_state(long_bad1).unwrap();
    assert!(s.bActive);
    assert!(!s.bState);
    assert!(!s.bChanged);

    let s = f.get_bool_state(long_bad2).unwrap();
    assert!(s.bActive);
    assert!(!s.bState);
    assert!(!s.bChanged);

    let s = f.get_bool_state(long_exact).unwrap();
    assert!(s.bActive);
    assert!(!s.bState);
    assert!(!s.bChanged);

    let mut s = vr::InputAnalogActionData_t::default();
    let ret = f
        .input
        .GetAnalogActionData(commas, &mut s, std::mem::size_of_val(&s) as u32, 0);
    assert_eq!(ret, vr::EVRInputError::None);

    assert!(s.bActive);
    assert_eq!(s.x, 0.5);
}

#[test]
fn bindings_with_mismatched_action_type_are_skipped() {
    let f = Fixture::new();
    // The binding file binds actions to inputs of a different type than the action:
    // a boolean action to a trigger's pull (float), a vector1 action to a joystick's position
    // (vector2) and a vector2 action to a joystick's click (boolean).
    // These must be skipped without preventing the valid bindings from being suggested.
    f.load_actions(c"actions_type_mismatch.json");

    let path = Knuckles::profile_path();
    f.verify_bindings::<bool>(
        path,
        c"/actions/default/in/use",
        [
            "/user/hand/left/input/trigger/touch".into(),
            "/user/hand/left/input/thumbstick/touch".into(),
        ],
    );
    f.verify_bindings::<f32>(
        path,
        c"/actions/default/in/grabaxis",
        ["/user/hand/right/input/trigger/value".into()],
    );
    f.verify_bindings::<xr::Vector2f>(
        path,
        c"/actions/default/in/move",
        ["/user/hand/right/input/thumbstick".into()],
    );
}

#[test]
fn pose_action_no_restrict() {
    let mut f = Fixture::new();

    let set1 = f.get_action_set_handle(c"/actions/set1");
    let posel = f.get_action_handle(c"/actions/set1/in/posel");
    let poser = f.get_action_handle(c"/actions/set1/in/poser");

    f.load_actions(c"actions.json");
    f.set_interaction_profile::<SimpleController>(LeftHand);
    f.set_interaction_profile::<SimpleController>(RightHand);
    let session = f.input.openxr.session_data.get().session.as_raw();
    let pose_left = xr::Posef {
        position: xr::Vector3f {
            x: 0.5,
            y: 0.5,
            z: 0.5,
        },
        orientation: xr::Quaternionf::IDENTITY,
    };
    fakexr::set_grip(session, LeftHand, pose_left);

    let pose_right = xr::Posef {
        position: xr::Vector3f {
            x: 0.6,
            y: 0.6,
            z: 0.6,
        },
        orientation: xr::Quaternionf::IDENTITY,
    };
    fakexr::set_grip(session, RightHand, pose_right);

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    for (handle, expected) in [(posel, pose_left), (poser, pose_right)] {
        let actual = f.get_pose(handle, 0).unwrap();
        assert!(actual.bActive);
        let p = actual.pose;
        assert!(p.bPoseIsValid);
        compare_pose(expected, p.mDeviceToAbsoluteTracking.into());
    }
}

#[test]
fn raw_pose_switch_profile() {
    let mut f = Fixture::new();

    let set1 = f.get_action_set_handle(c"/actions/set1");
    let posel = f.get_action_handle(c"/actions/set1/in/posel");
    let poser = f.get_action_handle(c"/actions/set1/in/poser");

    f.load_actions(c"actions.json");
    f.set_interaction_profile::<SimpleController>(LeftHand);
    f.set_interaction_profile::<SimpleController>(RightHand);
    let session = f.input.openxr.session_data.get().session.as_raw();
    let pose_left = xr::Posef {
        position: xr::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        orientation: xr::Quaternionf::IDENTITY,
    };
    fakexr::set_grip(session, LeftHand, pose_left);

    let pose_right = xr::Posef {
        position: xr::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        orientation: xr::Quaternionf::IDENTITY,
    };
    fakexr::set_grip(session, RightHand, pose_right);

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    fn offset_to_pose(offset: &Mat4) -> xr::Posef {
        let translation = offset.w_axis.truncate();
        let rotation = Quat::from_mat4(offset);

        xr::Posef {
            orientation: xr::Quaternionf {
                x: rotation.x,
                y: rotation.y,
                z: rotation.z,
                w: rotation.w,
            },
            position: xr::Vector3f {
                x: translation.x,
                y: translation.y,
                z: translation.z,
            },
        }
    }

    for (handle, hand) in [(posel, Hand::Left), (poser, Hand::Right)] {
        let expected = &SimpleController::offset_grip_pose(hand);
        let actual = f.get_pose(handle, 0).unwrap();
        assert!(actual.bActive, "{hand:?} not active");
        let p = actual.pose;
        assert!(p.bPoseIsValid, "{hand:?} invalid pose");
        compare_pose(offset_to_pose(expected), p.mDeviceToAbsoluteTracking.into());
    }

    f.set_interaction_profile::<OculusTouch>(LeftHand);
    f.set_interaction_profile::<OculusTouch>(RightHand);

    // Cached poses don't reset until next frame
    f.input.openxr.poll_events();
    f.input.frame_start_update();

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    for (handle, expected) in [
        (posel, &OculusTouch::offset_grip_pose(Hand::Left)),
        (poser, &OculusTouch::offset_grip_pose(Hand::Right)),
    ] {
        let actual = f.get_pose(handle, 0).unwrap();
        assert!(actual.bActive);
        let p = actual.pose;
        assert!(p.bPoseIsValid);
        compare_pose(offset_to_pose(expected), p.mDeviceToAbsoluteTracking.into());
    }
}

#[test]
fn cased_actions() {
    let mut f = Fixture::new();
    let set1 = f.get_action_set_handle(c"/actions/set1");
    f.load_actions(c"actions_cased.json");

    let path = ViveWands::profile_path();
    f.verify_bindings::<bool>(
        path,
        c"/actions/set1/in/BoolAct",
        ["/user/hand/left/input/squeeze/click".into()],
    );
    f.verify_bindings::<f32>(
        path,
        c"/actions/set1/in/Vec1Act",
        ["/user/hand/left/input/trigger/value".into()],
    );
    f.verify_bindings::<xr::Vector2f>(
        path,
        c"/actions/set1/in/Vec2Act",
        ["/user/hand/left/input/trackpad".into()],
    );
    f.verify_bindings::<xr::Haptic>(
        path,
        c"/actions/set1/in/VibAct",
        ["/user/hand/left/output/haptic".into()],
    );

    f.set_interaction_profile::<ViveWands>(LeftHand);
    let session = f.input.openxr.session_data.get().session.as_raw();
    fakexr::set_grip(session, LeftHand, xr::Posef::IDENTITY);
    fakexr::set_aim(session, LeftHand, xr::Posef::IDENTITY);
    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let poseact = f.get_action_handle(c"/actions/set1/in/PoseAct");
    let pose = f.get_pose(poseact, 0).unwrap();
    assert!(pose.bActive);
    assert!(pose.pose.bPoseIsValid);

    let skelact = f.get_action_handle(c"/actions/set1/in/SkelAct");
    let pose = f.get_pose(skelact, 0).unwrap();
    assert!(pose.bActive);
    assert!(pose.pose.bPoseIsValid);
}

#[test]
fn digital_action_initalize_on_failure() {
    let f = Fixture::new();
    f.load_actions(c"actions.json");
    let bad_state = vr::InputDigitalActionData_t {
        bActive: true,
        bState: true,
        activeOrigin: 12345,
        bChanged: true,
        fUpdateTime: -999.0,
    };

    let mut state = bad_state;
    let bad_handle = 3434343343;
    assert_ne!(
        f.input.GetDigitalActionData(
            bad_handle,
            &mut state,
            std::mem::size_of::<vr::InputDigitalActionData_t>() as u32,
            0
        ),
        vr::EVRInputError::None
    );
    assert!(!state.bActive);
    assert!(!state.bState);
    assert_eq!(state.activeOrigin, 0);
    assert!(!state.bChanged);
    assert_eq!(state.fUpdateTime, 0.0);

    let mut state = bad_state;
    let vecact = f.get_action_handle(c"/actions/set1/in/vec1act");
    assert_ne!(
        f.input.GetDigitalActionData(
            vecact,
            &mut state,
            std::mem::size_of::<vr::InputDigitalActionData_t>() as u32,
            0
        ),
        vr::EVRInputError::None
    );

    assert!(!state.bActive);
    assert!(!state.bState);
    assert_eq!(state.activeOrigin, 0);
    assert!(!state.bChanged);
    assert_eq!(state.fUpdateTime, 0.0);
}

#[test]
fn analog_action_initialize_on_failure() {
    let f = Fixture::new();
    f.load_actions(c"actions.json");

    let bad_state = vr::InputAnalogActionData_t {
        bActive: true,
        activeOrigin: 12345,
        x: 300.0,
        y: 600.0,
        z: 900.0,
        deltaX: 1000.0,
        deltaY: 3000.0,
        deltaZ: -999.999,
        fUpdateTime: 395.0,
    };

    let check_state = |state: vr::InputAnalogActionData_t, desc: &str| {
        assert!(!state.bActive, "{desc}");
        assert_eq!(state.activeOrigin, 0, "{desc}");
        assert_eq!(state.x, 0.0, "{desc}");
        assert_eq!(state.deltaX, 0.0, "{desc}");
        assert_eq!(state.y, 0.0, "{desc}");
        assert_eq!(state.deltaY, 0.0, "{desc}");
        assert_eq!(state.z, 0.0, "{desc}");
        assert_eq!(state.deltaZ, 0.0, "{desc}");
        assert_eq!(state.fUpdateTime, 0.0, "{desc}");
    };

    let mut state = bad_state;
    let bad_handle = 3434343343;
    assert_ne!(
        f.input.GetAnalogActionData(
            bad_handle,
            &mut state,
            std::mem::size_of::<vr::InputAnalogActionData_t>() as u32,
            0
        ),
        vr::EVRInputError::None
    );
    check_state(state, "bad handle");

    let mut state = bad_state;
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");
    assert_ne!(
        f.input.GetAnalogActionData(
            boolact,
            &mut state,
            std::mem::size_of::<vr::InputAnalogActionData_t>() as u32,
            0
        ),
        vr::EVRInputError::None
    );
    check_state(state, "wrong type");
}

#[test]
fn implicit_action_sets() {
    let mut f = Fixture::new();
    let set1 = f.get_action_set_handle(c"/actions/set1");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");
    f.load_actions(c"actions_missing_sets.json");

    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let res = f.get_bool_state(boolact);
    assert!(res.is_ok(), "{res:?}");
}

#[test]
fn detect_controller_after_manifest_load() {
    let mut f = Fixture::new();
    f.load_actions(c"actions.json");

    let input = f.input.clone();
    let frame = || {
        input.openxr.poll_events();
        input.frame_start_update();
    };

    frame();
    assert!(f.input.get_controller_device_index(Hand::Left).is_none());

    f.set_interaction_profile::<Knuckles>(fakexr::UserPath::LeftHand);
    frame();
    // Profile won't be set for this frame - we call sync after events have already been polled
    assert!(f.input.get_controller_device_index(Hand::Left).is_none());

    frame();
    let index = f.input.get_controller_device_index(Hand::Left);
    assert!(index.is_some_and(|i| f.input.is_device_connected(i)));
}

#[test]
fn empty_manifest() {
    let f = Fixture::new();
    f.input
        .SetActionManifestPath(c"empty_manifest.json".as_ptr() as _);

    f.input.openxr.restart_session();
    assert!(f.input.action_map.read().unwrap().is_empty());
}

#[test]
fn load_actions_race() {
    let mut f = Fixture::new();
    f.input.openxr.restart_session(); // get to real session

    f.set_interaction_profile::<OculusTouch>(LeftHand);
    f.set_interaction_profile::<OculusTouch>(RightHand);

    let mut f = Arc::new(f);
    f.input.frame_start_update(); // load legacy
    f.input.openxr.poll_events();
    let got_input = f.input.get_legacy_controller_state(
        1,
        &mut vr::VRControllerState_t::default(),
        std::mem::size_of::<vr::VRControllerState_t>() as _,
    );
    assert!(got_input);

    std::thread::scope(|scope| {
        let barrier = Arc::new(Barrier::new(2));
        {
            let input = f.input.clone();
            let barrier = barrier.clone();
            scope.spawn(move || {
                barrier.wait();
                // arbitrary delay, so we get the frame start update right after restart
                std::thread::sleep(std::time::Duration::from_micros(500));
                input.frame_start_update();
            });
        }
        {
            let f = f.clone();
            scope.spawn(move || {
                barrier.wait();
                f.load_actions(c"actions.json");
            });
        }
    });

    let got_input = f.input.get_legacy_controller_state(
        0,
        &mut vr::VRControllerState_t::default(),
        std::mem::size_of::<vr::VRControllerState_t>() as _,
    );
    assert!(!got_input);

    let set1 = f.get_action_set_handle(c"/actions/set1");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");
    Arc::get_mut(&mut f).unwrap().sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });

    let res = f.get_bool_state(boolact);
    assert!(res.is_ok(), "{res:?}");
}

#[test]
fn dominant_hand_defaults_to_right() {
    let f = Fixture::new();

    let mut hand = vr::ETrackedControllerRole::Invalid;
    assert_eq!(
        vr::IVRInput011_Interface::GetDominantHand(&*f.input, &mut hand),
        vr::EVRInputError::None
    );
    assert_eq!(hand, vr::ETrackedControllerRole::RightHand);

    assert_eq!(
        vr::IVRInput011_Interface::GetDominantHand(&*f.input, std::ptr::null_mut()),
        vr::EVRInputError::InvalidParam
    );
}

#[test]
fn show_action_origins_is_a_noop() {
    // Vinyl Reality calls this when selecting a menu item (#331); it used to panic.
    let f = Fixture::new();
    f.load_actions(c"actions.json");
    let set = f.get_action_set_handle(c"/actions/set1");
    let action = f.get_action_handle(c"/actions/set1/in/boolact");

    assert_eq!(
        vr::IVRInput011_Interface::ShowActionOrigins(&*f.input, set, action),
        vr::EVRInputError::None
    );
    assert_eq!(
        vr::IVRInput011_Interface::ShowActionOrigins(&*f.input, 0, 0),
        vr::EVRInputError::None
    );
}

#[test]
fn show_bindings_for_action_set_is_a_noop() {
    let f = Fixture::new();
    f.load_actions(c"actions.json");
    let set = f.get_action_set_handle(c"/actions/set1");
    let mut active = vr::VRActiveActionSet_t {
        ulActionSet: set,
        ..Default::default()
    };

    assert_eq!(
        vr::IVRInput011_Interface::ShowBindingsForActionSet(
            &*f.input,
            &mut active,
            std::mem::size_of::<vr::VRActiveActionSet_t>() as u32,
            1,
            0
        ),
        vr::EVRInputError::None
    );
    assert_eq!(
        vr::IVRInput011_Interface::ShowBindingsForActionSet(
            &*f.input,
            std::ptr::null_mut(),
            0,
            0,
            0
        ),
        vr::EVRInputError::None
    );
}

#[test]
fn component_state_for_binding_has_no_data() {
    let f = Fixture::new();
    let binding = vr::InputBindingInfo_t::default();
    let mut state = vr::RenderModel_ComponentState_t {
        uProperties: 0x55,
        ..Default::default()
    };

    assert_eq!(
        vr::IVRInput011_Interface::GetComponentStateForBinding(
            &*f.input,
            c"renderModel".as_ptr(),
            c"component".as_ptr(),
            &binding,
            std::mem::size_of::<vr::InputBindingInfo_t>() as u32,
            1,
            &mut state
        ),
        vr::EVRInputError::NoData
    );
    // The output must not be fabricated.
    assert_eq!(state.uProperties, 0x55);
    assert_eq!(state.mTrackingToComponentLocal.m, [[0.0; 4]; 3]);

    // null arguments are not dereferenced
    assert_eq!(
        vr::IVRInput011_Interface::GetComponentStateForBinding(
            &*f.input,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            0,
            std::ptr::null_mut()
        ),
        vr::EVRInputError::NoData
    );
}

// Parent of each bone in SteamVR's reference hand skeleton
// (SteamVR/resources/skeletons/vr_glove_{left,right}_skeleton.glb).
const STEAMVR_BONE_PARENTS: [i32; 31] = [
    -1, 0, 1, 2, 3, 4, 1, 6, 7, 8, 9, 1, 11, 12, 13, 14, 1, 16, 17, 18, 19, 1, 21, 22, 23, 24, 0,
    0, 0, 0, 0,
];

const STEAMVR_BONE_NAMES_LEFT: [&str; 31] = [
    "Root",
    "wrist_l",
    "finger_thumb_0_l",
    "finger_thumb_1_l",
    "finger_thumb_2_l",
    "finger_thumb_l_end",
    "finger_index_meta_l",
    "finger_index_0_l",
    "finger_index_1_l",
    "finger_index_2_l",
    "finger_index_l_end",
    "finger_middle_meta_l",
    "finger_middle_0_l",
    "finger_middle_1_l",
    "finger_middle_2_l",
    "finger_middle_l_end",
    "finger_ring_meta_l",
    "finger_ring_0_l",
    "finger_ring_1_l",
    "finger_ring_2_l",
    "finger_ring_l_end",
    "finger_pinky_meta_l",
    "finger_pinky_0_l",
    "finger_pinky_1_l",
    "finger_pinky_2_l",
    "finger_pinky_l_end",
    "finger_thumb_l_aux",
    "finger_index_l_aux",
    "finger_middle_l_aux",
    "finger_ring_l_aux",
    "finger_pinky_l_aux",
];

const STEAMVR_BONE_NAMES_RIGHT: [&str; 31] = [
    "Root",
    "wrist_r",
    "finger_thumb_0_r",
    "finger_thumb_1_r",
    "finger_thumb_2_r",
    "finger_thumb_r_end",
    "finger_index_meta_r",
    "finger_index_0_r",
    "finger_index_1_r",
    "finger_index_2_r",
    "finger_index_r_end",
    "finger_middle_meta_r",
    "finger_middle_0_r",
    "finger_middle_1_r",
    "finger_middle_2_r",
    "finger_middle_r_end",
    "finger_ring_meta_r",
    "finger_ring_0_r",
    "finger_ring_1_r",
    "finger_ring_2_r",
    "finger_ring_r_end",
    "finger_pinky_meta_r",
    "finger_pinky_0_r",
    "finger_pinky_1_r",
    "finger_pinky_2_r",
    "finger_pinky_r_end",
    "finger_thumb_r_aux",
    "finger_index_r_aux",
    "finger_middle_r_aux",
    "finger_ring_r_aux",
    "finger_pinky_r_aux",
];

#[test]
fn bone_hierarchy() {
    let f = Fixture::new();
    f.load_actions(c"actions.json");

    for name in [c"/actions/set1/in/SkellyL", c"/actions/set1/in/SkellyR"] {
        let skeleton = f.get_action_handle(name);

        let mut count = 0;
        assert_eq!(
            vr::IVRInput011_Interface::GetBoneCount(&*f.input, skeleton, &mut count),
            vr::EVRInputError::None
        );
        assert_eq!(count as usize, STEAMVR_BONE_PARENTS.len());

        let mut parents = [i32::MIN; 31];
        assert_eq!(
            vr::IVRInput011_Interface::GetBoneHierarchy(
                &*f.input,
                skeleton,
                parents.as_mut_ptr(),
                parents.len() as u32
            ),
            vr::EVRInputError::None
        );
        assert_eq!(parents, STEAMVR_BONE_PARENTS);

        // Every bone, except the root and the aux bones, is a child of an earlier bone.
        for (bone, parent) in parents.iter().enumerate() {
            assert!(*parent < bone as i32, "bone {bone} parent {parent}");
        }
    }
}

#[test]
fn bone_hierarchy_bad_params() {
    let f = Fixture::new();
    f.load_actions(c"actions.json");
    let skeleton = f.get_action_handle(c"/actions/set1/in/SkellyR");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");

    let mut parents = [i32::MIN; 31];
    let get = |action, ptr, count| {
        vr::IVRInput011_Interface::GetBoneHierarchy(&*f.input, action, ptr, count)
    };

    // too few entries: nothing is written
    assert_eq!(
        get(skeleton, parents.as_mut_ptr(), 30),
        vr::EVRInputError::BufferTooSmall
    );
    assert_eq!(
        get(skeleton, parents.as_mut_ptr(), 0),
        vr::EVRInputError::BufferTooSmall
    );
    assert_eq!(parents, [i32::MIN; 31]);

    assert_eq!(
        get(skeleton, std::ptr::null_mut(), 31),
        vr::EVRInputError::InvalidParam
    );
    assert_eq!(
        get(boolact, parents.as_mut_ptr(), 31),
        vr::EVRInputError::WrongType
    );
    assert_eq!(
        get(0xdead_beef, parents.as_mut_ptr(), 31),
        vr::EVRInputError::InvalidHandle
    );
    assert_eq!(parents, [i32::MIN; 31]);

    // a larger array is fine, but only the skeleton's bones are written
    let mut big = [i32::MIN; 40];
    assert_eq!(get(skeleton, big.as_mut_ptr(), 40), vr::EVRInputError::None);
    assert_eq!(big[..31], STEAMVR_BONE_PARENTS);
    assert_eq!(big[31..], [i32::MIN; 9]);
}

#[test]
fn bone_hierarchy_without_loaded_actions() {
    let f = Fixture::new();
    let mut parents = [i32::MIN; 31];
    assert_eq!(
        vr::IVRInput011_Interface::GetBoneHierarchy(&*f.input, 1, parents.as_mut_ptr(), 31),
        vr::EVRInputError::InvalidHandle
    );
    assert_eq!(parents, [i32::MIN; 31]);
}

fn get_bone_name(
    f: &Fixture,
    skeleton: vr::VRActionHandle_t,
    bone: i32,
    buffer: &mut [std::ffi::c_char],
) -> vr::EVRInputError {
    vr::IVRInput011_Interface::GetBoneName(
        &*f.input,
        skeleton,
        bone,
        buffer.as_mut_ptr(),
        buffer.len() as u32,
    )
}

#[test]
fn bone_names() {
    let f = Fixture::new();
    f.load_actions(c"actions.json");

    for (action, expected) in [
        (c"/actions/set1/in/SkellyL", STEAMVR_BONE_NAMES_LEFT),
        (c"/actions/set1/in/SkellyR", STEAMVR_BONE_NAMES_RIGHT),
    ] {
        let skeleton = f.get_action_handle(action);

        for (bone, expected) in expected.iter().enumerate() {
            // OpenVR's k_unMaxBoneNameLength
            let mut buffer = [0x55; 32];
            assert_eq!(
                get_bone_name(&f, skeleton, bone as i32, &mut buffer),
                vr::EVRInputError::None
            );
            let name = unsafe { CStr::from_ptr(buffer.as_ptr()) };
            assert_eq!(name.to_str().unwrap(), *expected);

            // a buffer that is exactly large enough for the name and its terminator
            let mut exact = vec![0x55; expected.len() + 1];
            assert_eq!(
                get_bone_name(&f, skeleton, bone as i32, &mut exact),
                vr::EVRInputError::None
            );
            assert_eq!(
                unsafe { CStr::from_ptr(exact.as_ptr()) }.to_str().unwrap(),
                *expected
            );

            // one byte short: error, nothing written
            let mut short = vec![0x55; expected.len()];
            assert_eq!(
                get_bone_name(&f, skeleton, bone as i32, &mut short),
                vr::EVRInputError::BufferTooSmall
            );
            assert!(short.iter().all(|c| *c == 0x55));
        }
    }
}

#[test]
fn bone_name_bad_params() {
    let f = Fixture::new();
    f.load_actions(c"actions.json");
    let skeleton = f.get_action_handle(c"/actions/set1/in/SkellyL");
    let boolact = f.get_action_handle(c"/actions/set1/in/boolact");

    let mut buffer = [0x55; 32];
    for bone in [-1, 31, 32, i32::MAX, i32::MIN] {
        assert_eq!(
            get_bone_name(&f, skeleton, bone, &mut buffer),
            vr::EVRInputError::InvalidBoneIndex,
            "bone {bone}"
        );
    }
    assert_eq!(
        get_bone_name(&f, boolact, 0, &mut buffer),
        vr::EVRInputError::WrongType
    );
    assert_eq!(
        get_bone_name(&f, 0xdead_beef, 0, &mut buffer),
        vr::EVRInputError::InvalidHandle
    );
    assert_eq!(
        get_bone_name(&f, skeleton, 0, &mut []),
        vr::EVRInputError::BufferTooSmall
    );
    assert!(buffer.iter().all(|c| *c == 0x55));

    assert_eq!(
        vr::IVRInput011_Interface::GetBoneName(&*f.input, skeleton, 0, std::ptr::null_mut(), 32),
        vr::EVRInputError::InvalidParam
    );
}

#[test]
fn compressed_skeletal_data_is_unsupported() {
    let f = Fixture::new();
    f.load_actions(c"actions.json");
    let skeleton = f.get_action_handle(c"/actions/set1/in/SkellyR");

    let mut data = [0x55u8; 64];
    let mut required = 0xdead_beef;
    assert_eq!(
        vr::IVRInput011_Interface::GetSkeletalBoneDataCompressed(
            &*f.input,
            skeleton,
            vr::EVRSkeletalMotionRange::WithController,
            data.as_mut_ptr().cast(),
            data.len() as u32,
            &mut required,
        ),
        vr::EVRInputError::NoData
    );
    // no compressed data is produced
    assert_eq!(required, 0);
    assert_eq!(data, [0x55u8; 64]);

    // null output pointers are not dereferenced
    assert_eq!(
        vr::IVRInput011_Interface::GetSkeletalBoneDataCompressed(
            &*f.input,
            skeleton,
            vr::EVRSkeletalMotionRange::WithController,
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
        ),
        vr::EVRInputError::NoData
    );

    let compressed = [0x55u8; 16];
    let mut bones = [vr::VRBoneTransform_t::default(); 31];
    assert_eq!(
        vr::IVRInput011_Interface::DecompressSkeletalBoneData(
            &*f.input,
            compressed.as_ptr().cast(),
            compressed.len() as u32,
            vr::EVRSkeletalTransformSpace::Parent,
            bones.as_mut_ptr(),
            bones.len() as u32,
        ),
        vr::EVRInputError::InvalidCompressedData
    );
    // the bones must not be fabricated
    assert!(bones.iter().all(|b| b.position.v == [0.0; 4]));
    assert!(bones.iter().all(|b| b.orientation.w == 0.0));

    assert_eq!(
        vr::IVRInput011_Interface::DecompressSkeletalBoneData(
            &*f.input,
            std::ptr::null(),
            0,
            vr::EVRSkeletalTransformSpace::Parent,
            std::ptr::null_mut(),
            0,
        ),
        vr::EVRInputError::InvalidCompressedData
    );
}

#[test]
fn legacy_004_compressed_skeletal_data_is_unsupported() {
    let f = Fixture::new();
    f.load_actions(c"actions.json");
    let skeleton = f.get_action_handle(c"/actions/set1/in/SkellyR");

    let mut data = [0x55u8; 64];
    let mut required = 0xdead_beef;
    assert_eq!(
        vr::IVRInput004On005::GetSkeletalBoneDataCompressed(
            &*f.input,
            skeleton,
            vr::EVRSkeletalTransformSpace::Parent,
            vr::EVRSkeletalMotionRange::WithController,
            data.as_mut_ptr().cast(),
            data.len() as u32,
            &mut required,
            vr::k_ulInvalidInputValueHandle,
        ),
        vr::EVRInputError::NoData
    );
    assert_eq!(required, 0);
    assert_eq!(data, [0x55u8; 64]);

    assert_eq!(
        vr::IVRInput004On005::GetSkeletalBoneDataCompressed(
            &*f.input,
            skeleton,
            vr::EVRSkeletalTransformSpace::Parent,
            vr::EVRSkeletalMotionRange::WithController,
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            vr::k_ulInvalidInputValueHandle,
        ),
        vr::EVRInputError::NoData
    );

    let mut compressed = [0x55u8; 16];
    let mut space = vr::EVRSkeletalTransformSpace::Model;
    let mut bones = [vr::VRBoneTransform_t::default(); 31];
    assert_eq!(
        vr::IVRInput004On005::DecompressSkeletalBoneData(
            &*f.input,
            compressed.as_mut_ptr().cast(),
            compressed.len() as u32,
            &mut space,
            bones.as_mut_ptr(),
            bones.len() as u32,
        ),
        vr::EVRInputError::InvalidCompressedData
    );
    assert_eq!(space, vr::EVRSkeletalTransformSpace::Model);
    assert!(bones.iter().all(|b| b.position.v == [0.0; 4]));
    assert!(bones.iter().all(|b| b.orientation.w == 0.0));

    assert_eq!(
        vr::IVRInput004On005::DecompressSkeletalBoneData(
            &*f.input,
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        ),
        vr::EVRInputError::InvalidCompressedData
    );
}

#[test]
fn head_proximity_action_follows_user_presence() {
    let mut f = Fixture::new();
    f.load_actions(c"actions_proximity.json");

    let set1 = f.get_action_set_handle(c"/actions/set1");
    let worn = f.get_action_handle(c"/actions/set1/in/headworn");
    let head = f.get_input_source_handle(c"/user/head");
    let left = f.get_input_source_handle(c"/user/hand/left");
    let sync = |f: &mut Fixture| {
        f.sync(vr::VRActiveActionSet_t {
            ulActionSet: set1,
            ..Default::default()
        });
    };

    // The user is present until the runtime says otherwise.
    sync(&mut f);
    let state = f.get_bool_state(worn).unwrap();
    assert!(state.bActive);
    assert!(state.bState);
    assert!(!state.bChanged);
    assert!(f.get_bool_state_hand(worn, head).unwrap().bState);

    // The very first presence event is a removal.
    fakexr::set_user_presence(f.raw_session(), false);
    f.input.openxr.poll_events();
    // The OpenVR state only updates at the next UpdateActionState.
    assert!(f.get_bool_state(worn).unwrap().bState);
    sync(&mut f);
    let state = f.get_bool_state(worn).unwrap();
    assert!(state.bActive);
    assert!(!state.bState);
    assert!(state.bChanged);
    assert_eq!(state.activeOrigin, head);
    let state = f.get_bool_state_hand(worn, head).unwrap();
    assert!(state.bActive);
    assert!(!state.bState);

    sync(&mut f);
    let state = f.get_bool_state(worn).unwrap();
    assert!(state.bActive);
    assert!(!state.bState);
    assert!(!state.bChanged);

    fakexr::set_user_presence(f.raw_session(), true);
    f.input.openxr.poll_events();
    sync(&mut f);
    let state = f.get_bool_state(worn).unwrap();
    assert!(state.bActive);
    assert!(state.bState);
    assert!(state.bChanged);

    // The head is not a hand.
    assert!(!f.get_bool_state_hand(worn, left).unwrap().bActive);
}

#[test]
fn head_proximity_action_without_runtime_support() {
    fakexr::set_user_presence_supported(false);
    let mut f = Fixture::new();
    fakexr::set_user_presence_supported(true);
    f.load_actions(c"actions_proximity.json");

    let set1 = f.get_action_set_handle(c"/actions/set1");
    let worn = f.get_action_handle(c"/actions/set1/in/headworn");

    // The runtime has no user presence sensing, so we have nothing to report - even if the runtime
    // sends events anyway.
    fakexr::set_user_presence(f.raw_session(), false);
    f.input.openxr.poll_events();
    f.sync(vr::VRActiveActionSet_t {
        ulActionSet: set1,
        ..Default::default()
    });
    let state = f.get_bool_state(worn).unwrap();
    assert!(!state.bActive);
    assert!(!state.bState);
    assert!(!state.bChanged);
}

#[test]
fn unimplemented_getters_write_neutral_outputs() {
    use std::ffi::c_char;
    use vr::IVRInput011_Interface as I;

    let f = Fixture::new();
    let input = &*f.input;
    let hand = f.get_input_source_handle(c"/user/hand/left");
    let ok = vr::EVRInputError::None;

    // Strings: "" in a non-null buffer of size > 0; size 0 and null buffers are left alone.
    type StringGetter<'a> = &'a dyn Fn(*mut c_char, u32) -> vr::EVRInputError;
    let string_getters: [StringGetter; 2] = [
        &|buf, size| I::GetBindingVariant(input, hand, buf, size),
        &|buf, size| I::GetOriginLocalizedName(input, hand, buf, size, 0),
    ];
    for get in string_getters {
        let mut buf = [0x7f as c_char; 4];
        assert_eq!(get(buf.as_mut_ptr(), 4), ok);
        assert_eq!(buf, [0, 0x7f, 0x7f, 0x7f]);
        buf = [0x7f; 4];
        assert_eq!(get(buf.as_mut_ptr(), 0), ok);
        assert_eq!(buf, [0x7f; 4]);
        assert_eq!(get(std::ptr::null_mut(), 4), ok);
    }

    // Origins: every provided entry is overwritten with the invalid handle, nothing past it is.
    let mut origins = [0xdead_beef; 4];
    let err = I::GetActionOrigins(input, 0, 0, origins.as_mut_ptr(), 3);
    assert_eq!(err, ok);
    let invalid = vr::k_ulInvalidInputValueHandle;
    assert_eq!(origins, [invalid, invalid, invalid, 0xdead_beef]);
    assert_eq!(
        I::GetActionOrigins(input, 0, 0, std::ptr::null_mut(), 3),
        ok
    );
    assert_eq!(
        I::GetActionOrigins(input, 0, 0, origins.as_mut_ptr(), 0),
        ok
    );
}
