//! v987-dash: SCS Telemetry SDK plugin for Euro Truck Simulator 2 and
//! American Truck Simulator. Streams truck state as one flat JSON object per
//! UDP datagram to 127.0.0.1:47822.

mod sdk;

use sdk::*;
use std::ffi::{c_void, CStr};
use std::fmt::Write as _;
use std::net::UdpSocket;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

const TARGET: &str = "127.0.0.1:47822";
const SOURCE_VERSION: &str = "1.0";
const SEND_INTERVAL_ACTIVE: Duration = Duration::from_millis(50); // ~20 Hz
const SEND_INTERVAL_PAUSED: Duration = Duration::from_millis(500);

// Float channel slots.
const F_SPEED: usize = 0;
const F_RPM: usize = 1;
const F_FUEL: usize = 2;
const F_FUEL_RANGE: usize = 3;
const F_CRUISE: usize = 4;
const F_SPEED_LIMIT: usize = 5;
const F_NAV_DISTANCE: usize = 6;
const F_NAV_TIME: usize = 7;
const F_ODOMETER: usize = 8;
const F_COUNT: usize = 9;

// Bool channel slots.
const B_ENGINE: usize = 0;
const B_ELECTRIC: usize = 1;
const B_PARKING_BRAKE: usize = 2;
const B_LIGHTS_PARKING: usize = 3;
const B_LIGHTS_LOW: usize = 4;
const B_LIGHTS_HIGH: usize = 5;
const B_BEACON: usize = 6;
const B_BLINKER_LEFT: usize = 7;
const B_BLINKER_RIGHT: usize = 8;
const B_HAZARD: usize = 9;
const B_WIPERS: usize = 10;
const B_COUNT: usize = 11;

#[derive(Clone, Copy)]
enum Slot {
    F(usize),
    B(usize),
    /// Blinker lamp (left = 0, right = 1). Not part of the change detection.
    Lamp(usize),
    Gear,
    Retarder,
}

const CHANNELS: &[(&[u8], Slot)] = &[
    (CH_SPEED, Slot::F(F_SPEED)),
    (CH_ENGINE_RPM, Slot::F(F_RPM)),
    (CH_DISPLAYED_GEAR, Slot::Gear),
    (CH_ENGINE_ENABLED, Slot::B(B_ENGINE)),
    (CH_ELECTRIC_ENABLED, Slot::B(B_ELECTRIC)),
    (CH_FUEL, Slot::F(F_FUEL)),
    (CH_FUEL_RANGE, Slot::F(F_FUEL_RANGE)),
    (CH_CRUISE_CONTROL, Slot::F(F_CRUISE)),
    (CH_NAV_SPEED_LIMIT, Slot::F(F_SPEED_LIMIT)),
    (CH_NAV_DISTANCE, Slot::F(F_NAV_DISTANCE)),
    (CH_NAV_TIME, Slot::F(F_NAV_TIME)),
    (CH_ODOMETER, Slot::F(F_ODOMETER)),
    (CH_PARKING_BRAKE, Slot::B(B_PARKING_BRAKE)),
    (CH_LIGHT_PARKING, Slot::B(B_LIGHTS_PARKING)),
    (CH_LIGHT_LOW_BEAM, Slot::B(B_LIGHTS_LOW)),
    (CH_LIGHT_HIGH_BEAM, Slot::B(B_LIGHTS_HIGH)),
    (CH_LIGHT_BEACON, Slot::B(B_BEACON)),
    (CH_LBLINKER, Slot::B(B_BLINKER_LEFT)),
    (CH_RBLINKER, Slot::B(B_BLINKER_RIGHT)),
    (CH_LIGHT_LBLINKER, Slot::Lamp(0)),
    (CH_LIGHT_RBLINKER, Slot::Lamp(1)),
    (CH_HAZARD_WARNING, Slot::B(B_HAZARD)),
    (CH_WIPERS, Slot::B(B_WIPERS)),
    (CH_RETARDER_LEVEL, Slot::Retarder),
];

struct State {
    socket: Option<UdpSocket>,
    ats: bool,
    active: bool,
    hazard_supported: bool,

    floats: [f32; F_COUNT],
    bools: [bool; B_COUNT],
    lamps: [bool; 2],
    gear: i32,
    retarder: u32,

    brand: String,
    truck_name: String,
    fuel_capacity: f32,
    rpm_limit: f32,
    gears_forward: u32,
    gears_reverse: u32,

    last_send: Option<Instant>,
    last_bools: [bool; B_COUNT],
    last_active: bool,
    buf: String,
}

impl State {
    fn new(ats: bool) -> Self {
        let socket = UdpSocket::bind("127.0.0.1:0").ok().and_then(|s| {
            let _ = s.set_nonblocking(true);
            Some(s)
        });
        State {
            socket,
            ats,
            active: false, // the game starts in paused state
            hazard_supported: false,
            floats: [0.0; F_COUNT],
            bools: [false; B_COUNT],
            lamps: [false; 2],
            gear: 0,
            retarder: 0,
            brand: String::new(),
            truck_name: String::new(),
            fuel_capacity: 0.0,
            rpm_limit: 0.0,
            gears_forward: 0,
            gears_reverse: 0,
            last_send: None,
            last_bools: [false; B_COUNT],
            last_active: false,
            buf: String::with_capacity(1024),
        }
    }

    fn vehicle(&self) -> String {
        match (self.brand.is_empty(), self.truck_name.is_empty()) {
            (true, true) => String::new(),
            (false, true) => self.brand.clone(),
            (true, false) => self.truck_name.clone(),
            (false, false) => {
                if self.truck_name.starts_with(&self.brand) {
                    self.truck_name.clone()
                } else {
                    format!("{} {}", self.brand, self.truck_name)
                }
            }
        }
    }

    fn build_json(&mut self, active: bool) {
        let mut s = std::mem::take(&mut self.buf);
        s.clear();
        let f = &self.floats;
        let b = &self.bools;
        let (game, name) = if self.ats {
            ("ats", "American Truck Simulator")
        } else {
            ("ets2", "Euro Truck Simulator 2")
        };
        let gear = match self.gear {
            0 => "N".to_string(),
            g if g < 0 => format!("R{}", g.unsigned_abs()),
            g => g.to_string(),
        };
        let _ = write!(
            s,
            "{{\"type\":\"dash\",\"game\":\"{game}\",\"name\":\"{name}\",\"source\":\"v987-dash-{game} {SOURCE_VERSION}\",\"active\":{active}"
        );
        let _ = write!(s, ",\"speedKmh\":{}", num(f[F_SPEED].abs() * 3.6, 1));
        let _ = write!(s, ",\"rpm\":{}", num(f[F_RPM].max(0.0), 0));
        if self.rpm_limit > 0.0 {
            let _ = write!(s, ",\"maxRpm\":{}", num(self.rpm_limit, 0));
        }
        let _ = write!(s, ",\"gear\":\"{gear}\"");
        let _ = write!(s, ",\"gearsForward\":{},\"gearsReverse\":{}", self.gears_forward, self.gears_reverse);
        let _ = write!(s, ",\"engine\":{},\"electric\":{}", b[B_ENGINE], b[B_ELECTRIC]);
        if self.fuel_capacity > 0.0 {
            let frac = (f[F_FUEL] / self.fuel_capacity).clamp(0.0, 1.0);
            let _ = write!(s, ",\"fuel\":{}", num(frac, 3));
        }
        let _ = write!(s, ",\"fuelLiters\":{}", num(f[F_FUEL].max(0.0), 1));
        let _ = write!(s, ",\"fuelRange\":{}", num(f[F_FUEL_RANGE].max(0.0), 0));
        let _ = write!(s, ",\"cruise\":{}", num(f[F_CRUISE].max(0.0) * 3.6, 0));
        let _ = write!(s, ",\"speedLimit\":{}", num(f[F_SPEED_LIMIT].max(0.0) * 3.6, 0));
        let _ = write!(s, ",\"navDistance\":{}", num(f[F_NAV_DISTANCE].max(0.0) / 1000.0, 1));
        let _ = write!(s, ",\"navTime\":{}", num(f[F_NAV_TIME].max(0.0), 0));
        let _ = write!(s, ",\"odometer\":{}", num(f[F_ODOMETER].max(0.0), 1));
        let _ = write!(
            s,
            ",\"parkingBrake\":{},\"lightsParking\":{},\"lightsLow\":{},\"lightsHigh\":{},\"beacon\":{},\"blinkerLeft\":{},\"blinkerRight\":{}",
            b[B_PARKING_BRAKE],
            b[B_LIGHTS_PARKING],
            b[B_LIGHTS_LOW],
            b[B_LIGHTS_HIGH],
            b[B_BEACON],
            b[B_BLINKER_LEFT],
            b[B_BLINKER_RIGHT]
        );
        let _ = write!(
            s,
            ",\"blinkerLampLeft\":{},\"blinkerLampRight\":{}",
            self.lamps[0], self.lamps[1]
        );
        if self.hazard_supported {
            let _ = write!(s, ",\"hazard\":{}", b[B_HAZARD]);
        }
        let _ = write!(s, ",\"wipers\":{},\"retarder\":{}", b[B_WIPERS], self.retarder);
        let vehicle = self.vehicle();
        if !vehicle.is_empty() {
            s.push_str(",\"vehicle\":");
            push_json_str(&mut s, &vehicle);
        }
        s.push('}');
        self.buf = s;
    }

    fn send(&mut self, active: bool) {
        self.build_json(active);
        if let Some(sock) = &self.socket {
            let _ = sock.send_to(self.buf.as_bytes(), TARGET);
        }
        self.last_send = Some(Instant::now());
        self.last_bools = self.bools;
        self.last_active = active;
    }

    fn frame_end(&mut self) {
        let changed = self.bools != self.last_bools || self.active != self.last_active;
        let interval = if self.active { SEND_INTERVAL_ACTIVE } else { SEND_INTERVAL_PAUSED };
        let due = self.last_send.map_or(true, |t| t.elapsed() >= interval);
        if changed || due {
            self.send(self.active);
        }
    }
}

/// Finite number with fixed decimals; NaN/inf become 0 so the JSON stays valid.
fn num(v: f32, decimals: usize) -> String {
    let v = if v.is_finite() { v } else { 0.0 };
    let out = format!("{:.*}", decimals, v);
    if out == "-0" || out.starts_with("-0.") && out[3..].bytes().all(|c| c == b'0') {
        out[1..].to_string()
    } else {
        out
    }
}

fn push_json_str(s: &mut String, v: &str) {
    s.push('"');
    for c in v.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(s, "\\u{:04x}", c as u32);
            }
            c => s.push(c),
        }
    }
    s.push('"');
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn lock() -> MutexGuard<'static, Option<State>> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Runs `f` and swallows any panic so nothing unwinds into the game.
/// Every exported function and SDK callback goes through this; the crate is
/// built with panic=unwind so the catch actually works in release builds.
fn guard<R>(default: R, f: impl FnOnce() -> R) -> R {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(default)
}

unsafe fn cstr(p: scs_string_t) -> Option<&'static str> {
    if p.is_null() {
        None
    } else {
        CStr::from_ptr(p).to_str().ok()
    }
}

fn log_msg(log: scs_log_t, kind: scs_log_type_t, msg: &str) {
    if let Some(log) = log {
        let mut bytes: Vec<u8> = msg.bytes().filter(|&b| b != 0).collect();
        bytes.push(0);
        unsafe { log(kind, bytes.as_ptr().cast()) };
    }
}

// ---- callbacks -----------------------------------------------------------

unsafe extern "C" fn on_float(_n: scs_string_t, _i: scs_u32_t, v: *const scs_value_t, ctx: scs_context_t) {
    guard((), || {
        if v.is_null() || (*v).type_ != SCS_VALUE_TYPE_FLOAT {
            return;
        }
        let idx = ctx as usize;
        if let Some(st) = lock().as_mut() {
            if idx < F_COUNT {
                st.floats[idx] = (*v).value.value_float;
            }
        }
    })
}

unsafe extern "C" fn on_bool(_n: scs_string_t, _i: scs_u32_t, v: *const scs_value_t, ctx: scs_context_t) {
    guard((), || {
        if v.is_null() || (*v).type_ != SCS_VALUE_TYPE_BOOL {
            return;
        }
        let idx = ctx as usize;
        if let Some(st) = lock().as_mut() {
            if idx < B_COUNT {
                st.bools[idx] = (*v).value.value_bool != 0;
            }
        }
    })
}

unsafe extern "C" fn on_lamp(_n: scs_string_t, _i: scs_u32_t, v: *const scs_value_t, ctx: scs_context_t) {
    guard((), || {
        if v.is_null() || (*v).type_ != SCS_VALUE_TYPE_BOOL {
            return;
        }
        let idx = ctx as usize;
        if let Some(st) = lock().as_mut() {
            if idx < 2 {
                st.lamps[idx] = (*v).value.value_bool != 0;
            }
        }
    })
}

unsafe extern "C" fn on_gear(_n: scs_string_t, _i: scs_u32_t, v: *const scs_value_t, _ctx: scs_context_t) {
    guard((), || {
        if v.is_null() || (*v).type_ != SCS_VALUE_TYPE_S32 {
            return;
        }
        if let Some(st) = lock().as_mut() {
            st.gear = (*v).value.value_s32;
        }
    })
}

unsafe extern "C" fn on_retarder(_n: scs_string_t, _i: scs_u32_t, v: *const scs_value_t, _ctx: scs_context_t) {
    guard((), || {
        if v.is_null() || (*v).type_ != SCS_VALUE_TYPE_U32 {
            return;
        }
        if let Some(st) = lock().as_mut() {
            st.retarder = (*v).value.value_u32;
        }
    })
}

unsafe extern "C" fn on_frame_end(_e: scs_event_t, _info: *const c_void, _ctx: scs_context_t) {
    guard((), || {
        if let Some(st) = lock().as_mut() {
            st.frame_end();
        }
    })
}

unsafe extern "C" fn on_pause(event: scs_event_t, _info: *const c_void, _ctx: scs_context_t) {
    guard((), || {
        if let Some(st) = lock().as_mut() {
            st.active = event == SCS_TELEMETRY_EVENT_STARTED;
            st.send(st.active);
        }
    })
}

unsafe extern "C" fn on_configuration(_e: scs_event_t, info: *const c_void, _ctx: scs_context_t) {
    guard((), || {
        if info.is_null() {
            return;
        }
        let cfg = &*(info as *const scs_telemetry_configuration_t);
        if cstr(cfg.id) != Some(CONFIG_TRUCK) {
            return;
        }
        let mut guard = lock();
        let Some(st) = guard.as_mut() else { return };
        // Each truck configuration event carries the full set; an empty set
        // means no truck is configured.
        st.brand.clear();
        st.truck_name.clear();
        st.fuel_capacity = 0.0;
        st.rpm_limit = 0.0;
        st.gears_forward = 0;
        st.gears_reverse = 0;
        let mut attr = cfg.attributes;
        if attr.is_null() {
            return;
        }
        while !(*attr).name.is_null() {
            let a = &*attr;
            let val = &a.value;
            match (cstr(a.name), val.type_) {
                (Some(ATTR_BRAND), SCS_VALUE_TYPE_STRING) => {
                    st.brand = cstr(val.value.value_string).unwrap_or("").to_string()
                }
                (Some(ATTR_NAME), SCS_VALUE_TYPE_STRING) => {
                    st.truck_name = cstr(val.value.value_string).unwrap_or("").to_string()
                }
                (Some(ATTR_FUEL_CAPACITY), SCS_VALUE_TYPE_FLOAT) => st.fuel_capacity = val.value.value_float,
                (Some(ATTR_RPM_LIMIT), SCS_VALUE_TYPE_FLOAT) => st.rpm_limit = val.value.value_float,
                (Some(ATTR_GEARS_FORWARD), SCS_VALUE_TYPE_U32) => st.gears_forward = val.value.value_u32,
                (Some(ATTR_GEARS_REVERSE), SCS_VALUE_TYPE_U32) => st.gears_reverse = val.value.value_u32,
                _ => {}
            }
            attr = attr.add(1);
        }
    })
}

// ---- exported entry points -------------------------------------------------

/// # Safety
/// Called by the game with a valid `scs_telemetry_init_params_v100_t`.
#[no_mangle]
pub unsafe extern "C" fn scs_telemetry_init(version: scs_u32_t, params: *const c_void) -> scs_result_t {
    guard(SCS_RESULT_GENERIC_ERROR, || init(version, params))
}

unsafe fn init(version: u32, params: *const c_void) -> scs_result_t {
    if version != SCS_TELEMETRY_VERSION_1_00 && version != SCS_TELEMETRY_VERSION_1_01 {
        return SCS_RESULT_UNSUPPORTED;
    }
    if params.is_null() {
        return SCS_RESULT_INVALID_PARAMETER;
    }
    let p = &*(params as *const scs_telemetry_init_params_v101_t);
    let log = p.common.log;

    let ats = match cstr(p.common.game_id) {
        Some(SCS_GAME_ID_EUT2) => false,
        Some(SCS_GAME_ID_ATS) => true,
        other => {
            log_msg(
                log,
                SCS_LOG_TYPE_WARNING,
                &format!("v987-dash: unsupported game id {:?}", other.unwrap_or("?")),
            );
            return SCS_RESULT_UNSUPPORTED;
        }
    };
    let (Some(reg_event), Some(reg_channel)) = (p.register_for_event, p.register_for_channel) else {
        return SCS_RESULT_INVALID_PARAMETER;
    };

    *lock() = Some(State::new(ats));

    let null = std::ptr::null_mut();
    let ok_frame = reg_event(SCS_TELEMETRY_EVENT_FRAME_END, Some(on_frame_end), null) == SCS_RESULT_OK;
    let ok_pause = reg_event(SCS_TELEMETRY_EVENT_PAUSED, Some(on_pause), null) == SCS_RESULT_OK;
    let ok_start = reg_event(SCS_TELEMETRY_EVENT_STARTED, Some(on_pause), null) == SCS_RESULT_OK;
    if !(ok_frame && ok_pause && ok_start) {
        log_msg(log, SCS_LOG_TYPE_ERROR, "v987-dash: could not register frame/pause events");
        *lock() = None;
        return SCS_RESULT_GENERIC_ERROR;
    }
    if reg_event(SCS_TELEMETRY_EVENT_CONFIGURATION, Some(on_configuration), null) != SCS_RESULT_OK {
        log_msg(log, SCS_LOG_TYPE_WARNING, "v987-dash: configuration event unavailable");
    }

    let mut hazard_ok = false;
    for &(name, slot) in CHANNELS {
        let (ty, cb, ctx): (scs_value_type_t, scs_telemetry_channel_callback_t, usize) = match slot {
            Slot::F(i) => (SCS_VALUE_TYPE_FLOAT, Some(on_float), i),
            Slot::B(i) => (SCS_VALUE_TYPE_BOOL, Some(on_bool), i),
            Slot::Lamp(i) => (SCS_VALUE_TYPE_BOOL, Some(on_lamp), i),
            Slot::Gear => (SCS_VALUE_TYPE_S32, Some(on_gear), 0),
            Slot::Retarder => (SCS_VALUE_TYPE_U32, Some(on_retarder), 0),
        };
        let r = reg_channel(
            name.as_ptr().cast(),
            SCS_U32_NIL,
            ty,
            SCS_TELEMETRY_CHANNEL_FLAG_NONE,
            cb,
            ctx as scs_context_t,
        );
        let is_hazard = name == CH_HAZARD_WARNING;
        if r == SCS_RESULT_OK {
            hazard_ok |= is_hazard;
        } else if !is_hazard {
            let n = String::from_utf8_lossy(&name[..name.len() - 1]);
            log_msg(log, SCS_LOG_TYPE_WARNING, &format!("v987-dash: channel {n} not available ({r})"));
        }
    }
    if let Some(st) = lock().as_mut() {
        st.hazard_supported = hazard_ok;
    }

    log_msg(log, SCS_LOG_TYPE_MESSAGE, &format!("v987-dash: streaming to udp {TARGET}"));
    SCS_RESULT_OK
}

#[no_mangle]
pub extern "C" fn scs_telemetry_shutdown() {
    guard((), || {
        let mut g = lock();
        if let Some(mut st) = g.take() {
            st.send(false);
        }
    })
}
