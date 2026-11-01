//! Byte-level codec for the LiteStar "FFB WHEEL" BLE protocol. See docs/PROTOCOL.md.

use serde::{Deserialize, Serialize};

pub const READ_STATE: u8 = 0x50;
pub const READ_PROFILE: u8 = 0x51;
pub const WRITE_PROFILE: u8 = 0x52;
pub const EDIT: u8 = 0x53;
pub const TEST: u8 = 0x54;

pub const SUB_MODE: u8 = 0x01;
pub const SUB_KEYMAP: u8 = 0x02;
pub const SUB_FFB: u8 = 0x03;
pub const SUB_ANGLE: u8 = 0x04;
pub const SUB_RESET: u8 = 0x08;
pub const SUB_FORCES: u8 = 0x09;

/// Codes listed in profile pages 0, 1 and 2, in on-wire order.
pub const KEY_SEQUENCE: [&[u8]; 3] = [
    &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 17, 16],
    &[14, 15, 47, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33],
    &[34, 35],
];

pub fn response_id(request_id: u8) -> u8 {
    request_id | 0x80
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceState {
    pub mode: u8,
    pub firmware: u16,
    pub pedal_state: u8,
    pub lever_state: u8,
    pub angle_mode: u8,
}

pub fn decode_state(d: &[u8]) -> Option<DeviceState> {
    if d.len() < 8 || d[0] != response_id(READ_STATE) {
        return None;
    }
    Some(DeviceState {
        mode: d[2],
        firmware: u16::from_le_bytes([d[3], d[4]]),
        pedal_state: d[5],
        lever_state: d[6],
        angle_mode: d[7],
    })
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    pub angle: u16,
    pub ffb: u8,
    pub spring: u8,
    pub damping: u8,
    pub friction: u8,
    pub inertia: u8,
    pub reverse: bool,
}

/// One pedal response curve: 4 points of (raw travel 0-4095, output 0-65535).
pub type Curve = [[u16; 2]; 4];

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub mode: u8,
    /// (source code, replacement code) for every mappable control.
    pub keymap: Vec<(u8, u8)>,
    pub params: Params,
    /// throttle, brake, clutch
    pub curves: [Curve; 3],
    /// Which pages answered with a valid header.
    pub pages_ok: [bool; 6],
}

pub fn read_page_request(mode: u8, page: u8) -> Vec<u8> {
    vec![READ_PROFILE, mode, page]
}

/// Applies a 0xD1 page reply to `p`. Returns false if the reply isn't a valid page.
pub fn apply_page(p: &mut Profile, d: &[u8]) -> bool {
    if d.len() < 20 || d[0] != response_id(READ_PROFILE) {
        return false;
    }
    let page = d[2] as usize;
    let body = &d[3..];
    match page {
        0 | 1 => {
            for (i, code) in KEY_SEQUENCE[page].iter().enumerate() {
                set_key(p, *code, body[i]);
            }
        }
        2 => {
            set_key(p, 34, body[0]);
            set_key(p, 35, body[1]);
            // Layout confirmed on the real wheel: a `09` forces write lands in body[5..9].
            p.params.angle = u16::from_le_bytes([body[2], body[3]]);
            p.params.ffb = body[4];
            p.params.spring = body[5];
            p.params.damping = body[6];
            p.params.friction = body[7];
            p.params.inertia = body[8];
            p.params.reverse = body[9] == 1;
        }
        3..=5 => {
            let c = &mut p.curves[page - 3];
            for (i, pt) in c.iter_mut().enumerate() {
                let o = i * 4;
                pt[0] = u16::from_le_bytes([body[o], body[o + 1]]);
                pt[1] = u16::from_le_bytes([body[o + 2], body[o + 3]]);
            }
        }
        _ => return false,
    }
    if page < 6 {
        p.pages_ok[page] = true;
    }
    true
}

fn set_key(p: &mut Profile, code: u8, replacement: u8) {
    if let Some(e) = p.keymap.iter_mut().find(|(c, _)| *c == code) {
        e.1 = replacement;
    } else {
        p.keymap.push((code, replacement));
    }
}

pub fn write_curve_request(mode: u8, pedal: u8, curve: &Curve) -> Vec<u8> {
    let mut b = vec![0u8; 20];
    b[0] = WRITE_PROFILE;
    b[1] = mode;
    b[2] = 3 + pedal;
    for (i, pt) in curve.iter().enumerate() {
        let o = 3 + i * 4;
        b[o..o + 2].copy_from_slice(&pt[0].to_le_bytes());
        b[o + 2..o + 4].copy_from_slice(&pt[1].to_le_bytes());
    }
    b
}

pub fn angle_request(angle: u16) -> Vec<u8> {
    let [lo, hi] = angle.to_le_bytes();
    vec![EDIT, SUB_ANGLE, 1, 1, lo, hi]
}

pub fn ffb_request(mode: u8, ffb: u8, reverse: bool) -> Vec<u8> {
    vec![EDIT, SUB_FFB, mode, ffb, reverse as u8]
}

pub fn forces_request(spring: u8, damping: u8, friction: u8, inertia: u8) -> Vec<u8> {
    vec![EDIT, SUB_FORCES, 1, spring, damping, friction, inertia]
}

pub fn keymap_request(mode: u8, target: u8, replacement: u8) -> Vec<u8> {
    vec![EDIT, SUB_KEYMAP, mode, target, replacement]
}

pub fn mode_request(mode: u8) -> Vec<u8> {
    vec![EDIT, SUB_MODE, mode]
}

pub fn reset_request(mode: u8) -> Vec<u8> {
    vec![EDIT, SUB_RESET, mode]
}

pub fn test_request(on: bool) -> Vec<u8> {
    vec![TEST, if on { 1 } else { 2 }]
}

/// Live input frame from test mode (`D4 03 …`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveInput {
    /// Codes that are currently pressed.
    pub pressed: Vec<u8>,
    pub angle: u8,
    pub throttle: u8,
    pub brake: u8,
    pub clutch: u8,
    /// 0 = neutral, 1-6, 255 = reverse.
    pub gear: u8,
}

pub fn is_live_frame(d: &[u8]) -> bool {
    d.len() >= 14 && d[0] == response_id(TEST) && d[1] == 3
}

pub fn decode_live(d: &[u8]) -> LiveInput {
    let mut pressed = Vec::new();
    let mut bits = |byte: u8, codes: &[u8]| {
        for (i, c) in codes.iter().enumerate() {
            if byte >> i & 1 == 1 {
                pressed.push(*c);
            }
        }
    };
    bits(d[2], &[0, 1, 2, 3, 4, 5, 6, 7]);
    bits(d[3], &[8, 9, 10, 11, 12, 13, 14, 15]);
    bits(d[4], &[16, 17]);
    bits(d[12], &[25, 26, 23, 24]);
    let gear = d[13];
    match gear {
        1..=6 => pressed.push(26 + gear),
        255 => pressed.push(33),
        _ => {}
    }
    LiveInput {
        pressed,
        angle: d[6],
        throttle: d[8],
        brake: d[9],
        clutch: d[10],
        gear,
    }
}

pub fn hex(d: &[u8]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(s: &str) -> Vec<u8> {
        s.split_whitespace().map(|x| u8::from_str_radix(x, 16).unwrap()).collect()
    }

    #[test]
    fn decodes_real_v987_replies() {
        let st = decode_state(&h("d0 01 01 46 00 02 02 01 00 00 00 00 00 00 00 00 00 00 00 00")).unwrap();
        assert_eq!((st.mode, st.firmware, st.pedal_state, st.lever_state), (1, 70, 2, 2));

        let mut p = Profile::default();
        assert!(apply_page(&mut p, &h("d1 01 02 22 23 38 04 64 50 50 50 50 00 00 00 00 00 00 00 00")));
        assert_eq!(
            p.params,
            Params { angle: 1080, ffb: 100, spring: 80, damping: 80, friction: 80, inertia: 80, reverse: false }
        );
        // Captured from a V987 (firmware 70) after LiteStar's phone app applied its ETS2 preset.
        // The app writes V2 presets two bytes off, so the wheel really holds angle 0x2322 and
        // FFB 132. Writing forces 30/80/10/0 afterwards moved exactly body[5..9], which
        // confirms the layout.
        let mut q = Profile::default();
        assert!(apply_page(&mut q, &h("d1 01 02 22 23 22 23 84 03 64 1e 50 00 00 00 00 00 00 00 00")));
        assert_eq!(
            q.params,
            Params { angle: 0x2322, ffb: 132, spring: 3, damping: 100, friction: 30, inertia: 80, reverse: false }
        );
        assert!(apply_page(&mut q, &h("d1 01 02 22 23 22 23 84 1e 50 0a 00 00 00 00 00 00 00 00 00")));
        assert_eq!((q.params.spring, q.params.damping, q.params.friction, q.params.inertia), (30, 80, 10, 0));
        assert!(apply_page(&mut p, &h("d1 01 00 00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 11 10 00")));
        assert!(p.keymap.iter().filter(|(c, _)| *c <= 17 && *c != 14 && *c != 15).all(|(c, r)| c == r));
        // Unsupported page: all zeros, no header.
        assert!(!apply_page(&mut p, &[0u8; 20]));
    }

    #[test]
    fn encodes_requests_like_vendor_app() {
        assert_eq!(angle_request(900), vec![0x53, 4, 1, 1, 0x84, 0x03]);
        assert_eq!(forces_request(80, 80, 80, 80), h("53 09 01 50 50 50 50"));
        let c: Curve = [[0, 0], [1365, 21845], [2730, 43690], [4095, 65535]];
        let w = write_curve_request(1, 0, &c);
        assert_eq!(&w[..3], &[0x52, 1, 3]);
        assert_eq!(&w[15..19], &[0xff, 0x0f, 0xff, 0xff]);
    }

    #[test]
    fn decodes_live_frame() {
        let mut d = vec![0u8; 20];
        d[0] = 0xd4;
        d[1] = 3;
        d[2] = 0b0000_0001; // code 0
        d[3] = 0b0000_0100; // code 10 (A)
        d[12] = 0b0000_0100; // code 23 (gear up)
        d[13] = 255; // reverse
        d[8] = 200;
        let l = decode_live(&d);
        assert_eq!(l.pressed, vec![0, 10, 23, 33]);
        assert_eq!(l.throttle, 200);
    }
}
