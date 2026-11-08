//! Hand-written Rust declarations of the SCS Telemetry SDK C ABI
//! (scssdk.h, scssdk_value.h, scssdk_telemetry*.h, common/*.h).
//!
//! SCSAPIFUNC is `__stdcall` with MSVC and empty with GCC. On x86_64 both
//! collapse to the platform C ABI, so `extern "C"` is correct for the 64-bit
//! Windows and Linux builds. 32-bit Windows is not supported by this crate.

#![allow(non_camel_case_types, dead_code)]

use std::ffi::{c_char, c_void};

#[cfg(all(windows, target_arch = "x86"))]
compile_error!("32-bit Windows needs __stdcall (extern \"system\"); not supported here");

pub type scs_u8_t = u8;
pub type scs_s32_t = i32;
pub type scs_u32_t = u32;
pub type scs_u64_t = u64;
pub type scs_s64_t = i64;
pub type scs_float_t = f32;
pub type scs_double_t = f64;
pub type scs_string_t = *const c_char;
pub type scs_context_t = *mut c_void;
pub type scs_result_t = scs_s32_t;
pub type scs_log_type_t = scs_s32_t;
pub type scs_value_type_t = scs_u32_t;
pub type scs_event_t = scs_u32_t;

// Result codes.
pub const SCS_RESULT_OK: scs_result_t = 0;
pub const SCS_RESULT_UNSUPPORTED: scs_result_t = -1;
pub const SCS_RESULT_INVALID_PARAMETER: scs_result_t = -2;
pub const SCS_RESULT_ALREADY_REGISTERED: scs_result_t = -3;
pub const SCS_RESULT_NOT_FOUND: scs_result_t = -4;
pub const SCS_RESULT_UNSUPPORTED_TYPE: scs_result_t = -5;
pub const SCS_RESULT_NOT_NOW: scs_result_t = -6;
pub const SCS_RESULT_GENERIC_ERROR: scs_result_t = -7;

// Log types.
pub const SCS_LOG_TYPE_MESSAGE: scs_log_type_t = 0;
pub const SCS_LOG_TYPE_WARNING: scs_log_type_t = 1;
pub const SCS_LOG_TYPE_ERROR: scs_log_type_t = 2;

pub const fn scs_make_version(major: u32, minor: u32) -> u32 {
    (major << 16) | minor
}
pub const SCS_TELEMETRY_VERSION_1_00: u32 = scs_make_version(1, 0);
pub const SCS_TELEMETRY_VERSION_1_01: u32 = scs_make_version(1, 1);

// Value types.
pub const SCS_VALUE_TYPE_INVALID: scs_value_type_t = 0;
pub const SCS_VALUE_TYPE_BOOL: scs_value_type_t = 1;
pub const SCS_VALUE_TYPE_S32: scs_value_type_t = 2;
pub const SCS_VALUE_TYPE_U32: scs_value_type_t = 3;
pub const SCS_VALUE_TYPE_U64: scs_value_type_t = 4;
pub const SCS_VALUE_TYPE_FLOAT: scs_value_type_t = 5;
pub const SCS_VALUE_TYPE_DOUBLE: scs_value_type_t = 6;
pub const SCS_VALUE_TYPE_STRING: scs_value_type_t = 12;
pub const SCS_VALUE_TYPE_S64: scs_value_type_t = 13;

// Events.
pub const SCS_TELEMETRY_EVENT_FRAME_START: scs_event_t = 1;
pub const SCS_TELEMETRY_EVENT_FRAME_END: scs_event_t = 2;
pub const SCS_TELEMETRY_EVENT_PAUSED: scs_event_t = 3;
pub const SCS_TELEMETRY_EVENT_STARTED: scs_event_t = 4;
pub const SCS_TELEMETRY_EVENT_CONFIGURATION: scs_event_t = 5;
pub const SCS_TELEMETRY_EVENT_GAMEPLAY: scs_event_t = 6;

// Channel flags.
pub const SCS_TELEMETRY_CHANNEL_FLAG_NONE: scs_u32_t = 0;
pub const SCS_TELEMETRY_CHANNEL_FLAG_EACH_FRAME: scs_u32_t = 1;
pub const SCS_TELEMETRY_CHANNEL_FLAG_NO_VALUE: scs_u32_t = 2;

pub const SCS_U32_NIL: scs_u32_t = u32::MAX;

pub const SCS_GAME_ID_EUT2: &str = "eut2";
pub const SCS_GAME_ID_ATS: &str = "ats";

/// Payload of `scs_value_t`. Only the scalar members are read; `_size`
/// pads the union to the size of scs_value_dplacement_t (40 bytes, align 8).
#[repr(C)]
#[derive(Clone, Copy)]
pub union scs_value_union {
    pub value_bool: scs_u8_t,
    pub value_s32: scs_s32_t,
    pub value_u32: scs_u32_t,
    pub value_u64: scs_u64_t,
    pub value_s64: scs_s64_t,
    pub value_float: scs_float_t,
    pub value_double: scs_double_t,
    pub value_string: scs_string_t,
    _size: [u64; 5],
}

#[repr(C)]
pub struct scs_value_t {
    pub type_: scs_value_type_t,
    pub _padding: scs_u32_t,
    pub value: scs_value_union,
}

#[repr(C)]
pub struct scs_named_value_t {
    pub name: scs_string_t,
    pub index: scs_u32_t,
    // On x64 the C header has an explicit _padding u32 here; repr(C) inserts
    // the same padding automatically before the 8-aligned scs_value_t.
    pub value: scs_value_t,
}

#[repr(C)]
pub struct scs_telemetry_configuration_t {
    pub id: scs_string_t,
    pub attributes: *const scs_named_value_t,
}

pub type scs_log_t = Option<unsafe extern "C" fn(type_: scs_log_type_t, message: scs_string_t)>;

pub type scs_telemetry_event_callback_t =
    Option<unsafe extern "C" fn(event: scs_event_t, event_info: *const c_void, context: scs_context_t)>;

pub type scs_telemetry_register_for_event_t = Option<
    unsafe extern "C" fn(
        event: scs_event_t,
        callback: scs_telemetry_event_callback_t,
        context: scs_context_t,
    ) -> scs_result_t,
>;

pub type scs_telemetry_unregister_from_event_t =
    Option<unsafe extern "C" fn(event: scs_event_t) -> scs_result_t>;

pub type scs_telemetry_channel_callback_t = Option<
    unsafe extern "C" fn(
        name: scs_string_t,
        index: scs_u32_t,
        value: *const scs_value_t,
        context: scs_context_t,
    ),
>;

pub type scs_telemetry_register_for_channel_t = Option<
    unsafe extern "C" fn(
        name: scs_string_t,
        index: scs_u32_t,
        type_: scs_value_type_t,
        flags: scs_u32_t,
        callback: scs_telemetry_channel_callback_t,
        context: scs_context_t,
    ) -> scs_result_t,
>;

pub type scs_telemetry_unregister_from_channel_t = Option<
    unsafe extern "C" fn(name: scs_string_t, index: scs_u32_t, type_: scs_value_type_t) -> scs_result_t,
>;

#[repr(C)]
pub struct scs_sdk_init_params_v100_t {
    pub game_name: scs_string_t,
    pub game_id: scs_string_t,
    pub game_version: scs_u32_t,
    // x64: explicit _padding u32 in the header; repr(C) pads identically.
    pub log: scs_log_t,
}

/// `scs_telemetry_init_params_v100_t` (v101 is a typedef of it). In C++ it
/// derives from an empty struct, which adds no bytes (empty base optimization),
/// so `common` sits at offset 0.
#[repr(C)]
pub struct scs_telemetry_init_params_v100_t {
    pub common: scs_sdk_init_params_v100_t,
    pub register_for_event: scs_telemetry_register_for_event_t,
    pub unregister_from_event: scs_telemetry_unregister_from_event_t,
    pub register_for_channel: scs_telemetry_register_for_channel_t,
    pub unregister_from_channel: scs_telemetry_unregister_from_channel_t,
}
pub type scs_telemetry_init_params_v101_t = scs_telemetry_init_params_v100_t;

// Same size checks the SDK headers make (scs_check_size(..., 32bit, 64bit)).
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(std::mem::size_of::<scs_value_t>() == 48);
    assert!(std::mem::size_of::<scs_named_value_t>() == 64);
    assert!(std::mem::size_of::<scs_telemetry_configuration_t>() == 16);
    assert!(std::mem::size_of::<scs_sdk_init_params_v100_t>() == 32);
    assert!(std::mem::size_of::<scs_telemetry_init_params_v100_t>() == 64);
    assert!(std::mem::offset_of!(scs_named_value_t, value) == 16);
    assert!(std::mem::offset_of!(scs_sdk_init_params_v100_t, log) == 24);
};

// Truck channels (common/scssdk_telemetry_truck_common_channels.h).
pub const CH_SPEED: &[u8] = b"truck.speed\0"; // float m/s, negative in reverse
pub const CH_ENGINE_RPM: &[u8] = b"truck.engine.rpm\0"; // float
pub const CH_DISPLAYED_GEAR: &[u8] = b"truck.displayed.gear\0"; // s32
pub const CH_ENGINE_ENABLED: &[u8] = b"truck.engine.enabled\0"; // bool
pub const CH_ELECTRIC_ENABLED: &[u8] = b"truck.electric.enabled\0"; // bool
pub const CH_FUEL: &[u8] = b"truck.fuel.amount\0"; // float litres
pub const CH_FUEL_RANGE: &[u8] = b"truck.fuel.range\0"; // float km
pub const CH_CRUISE_CONTROL: &[u8] = b"truck.cruise_control\0"; // float m/s, 0 = off
pub const CH_NAV_SPEED_LIMIT: &[u8] = b"truck.navigation.speed.limit\0"; // float m/s
pub const CH_NAV_DISTANCE: &[u8] = b"truck.navigation.distance\0"; // float metres
pub const CH_NAV_TIME: &[u8] = b"truck.navigation.time\0"; // float seconds
pub const CH_ODOMETER: &[u8] = b"truck.odometer\0"; // float km
pub const CH_PARKING_BRAKE: &[u8] = b"truck.brake.parking\0"; // bool
pub const CH_RETARDER_LEVEL: &[u8] = b"truck.brake.retarder\0"; // u32
pub const CH_LIGHT_PARKING: &[u8] = b"truck.light.parking\0"; // bool
pub const CH_LIGHT_LOW_BEAM: &[u8] = b"truck.light.beam.low\0"; // bool
pub const CH_LIGHT_HIGH_BEAM: &[u8] = b"truck.light.beam.high\0"; // bool
pub const CH_LIGHT_BEACON: &[u8] = b"truck.light.beacon\0"; // bool
pub const CH_LBLINKER: &[u8] = b"truck.lblinker\0"; // bool, switch state (does not blink)
pub const CH_RBLINKER: &[u8] = b"truck.rblinker\0"; // bool
pub const CH_LIGHT_LBLINKER: &[u8] = b"truck.light.lblinker\0"; // bool, lamp state (blinks)
pub const CH_LIGHT_RBLINKER: &[u8] = b"truck.light.rblinker\0"; // bool
pub const CH_HAZARD_WARNING: &[u8] = b"truck.hazard.warning\0"; // bool, newer SDK only
pub const CH_WIPERS: &[u8] = b"truck.wipers\0"; // bool

// Configuration ids and attributes (common/scssdk_telemetry_common_configs.h).
pub const CONFIG_TRUCK: &str = "truck";
pub const ATTR_BRAND: &str = "brand"; // string
pub const ATTR_NAME: &str = "name"; // string
pub const ATTR_FUEL_CAPACITY: &str = "fuel.capacity"; // float litres
pub const ATTR_RPM_LIMIT: &str = "rpm.limit"; // float
pub const ATTR_GEARS_FORWARD: &str = "gears.forward"; // u32
pub const ATTR_GEARS_REVERSE: &str = "gears.reverse"; // u32
