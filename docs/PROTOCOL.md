# LiteStar "FFB WHEEL" BLE configuration protocol

Reverse-engineered from LiteStar's `RacingWheel` app (`com.star.v10`, v1.0.25) and
the bundled PXN V12 SDK (`com.pxn.v12`), then verified against a real
**Rampage Pace V987** (BLE name `FFB WHEEL PACE V987`, firmware 70 / `0x46`).

Everything marked ✅ was observed on the V987. ❌ means the V987 ignores it.
❔ means the payload is known from the V12 SDK but the V987 has not been tested.

## Transport

| | |
|---|---|
| Advertised name | starts with `FFB WHEEL` (app matches case-insensitively) |
| Service | `0000ffe0-0000-1000-8000-00805f9b34fb` |
| Write (no response) | `ffe1` |
| Notify | `ffe2` (V10 family). V12/DD bases use `ffe4`, which the V987 lacks |

* Requests are raw byte arrays, no framing, no checksum. Replies are 20-byte
  notifications.
* Reply id = request id `| 0x80` (`0x50`→`0xD0` … `0x54`→`0xD4`).
* The vendor app keeps **exactly one request in flight**: send, wait for the
  reply, wait 100 ms, send the next one. It times out at 500 ms, retries 3×
  (2 s apart) and then disconnects.
* **Unsupported commands get no reply at all** and change nothing. That makes
  feature detection safe: a supported command echoes, an unsupported one is silent.
* Reading a profile page that does not exist returns 20 zero bytes (no header).

## Report ids

| Req | Resp | Meaning |
|---|---|---|
| `0x50` | `0xD0` | read device state |
| `0x51` | `0xD1` | read profile page |
| `0x52` | `0xD2` | write profile page |
| `0x53` | `0xD3` | edit a single parameter (sub-command in byte 1) |
| `0x54` | `0xD4` | test mode (live input stream) |
| `0xC2` | ?    | "reset" used by the vendor's *test* device class only. Not used for FFB wheels |

## Device state — `50 01` → `D0 …`

| Byte | Field |
|---|---|
| 2 | mode (see Modes) |
| 3–4 | firmware version, u16 LE (vendor masks with `& 0xFF`). V987 = 70 |
| 5 | pedal state (2 = pedals detected on V987) |
| 6 | shifter/lever state (2 = shifter detected on V987) |
| 7 | angle mode (1 on V987) |

## Modes

| Value | Mode | Switchable from app |
|---|---|---|
| 1 | PC D-Input | ✅ |
| 2 | PC X-Input | ✅ |
| 3 / 4 | PS3 HID / PS3 Sony | |
| 5 | PS4 wheel | |
| 6 | PS4 gamepad | |
| 7 | Xbox 360 | |
| 8 | Xbox One wheel | |
| 9 | Xbox One gamepad | |
| 10 | Android | |
| 11 | Switch Pro | |

Key maps and parameters are stored **per mode**.

## Profile pages — `51 <mode> <page>` → `D1 <mode> <page> …payload`

Page payloads begin at byte 3.

| Page | Payload |
|---|---|
| 0 | key map: replacement code for each of `[0..13, 17, 16]` (16 bytes) |
| 1 | key map: `[14, 15, 47, 20..33]` (17 bytes) |
| 2 | key map `[34, 35]`, then **angle u16 LE**, then `ffb, spring, damping, friction, inertia` (u8 each), then `reverse` (0/1). Confirmed on the wheel: a `09` forces write changes exactly bytes 5 to 8 of this body |
| 3 | throttle curve: 4 × (x u16 LE, y u16 LE) |
| 4 | brake curve, same format |
| 5 | clutch curve, same format |

A key map entry equal to its own code means "unchanged". Curve x is raw pedal
travel 0–4095 and y is output 0–65535. All zeros means no curve, i.e. linear.

Write a page with `52 <mode> <page> …payload` (20 bytes, same layout as the
read payload).

V987 factory-ish values seen: angle 1080, FFB 100, spring/damping/friction/inertia 80, reverse off.

## Edit parameter — `53 <sub> …` → `D3 <sub> …echo`

| Sub | Name | Payload | V987 |
|---|---|---|---|
| `01` | switch mode | `mode` | ✅ (USB re-enumerates) |
| `02` | key map | `mode target replacement` | ✅ reply echoes target/replacement in bytes 3–4 |
| `03` | force feedback | `mode ffb reverse` | ✅ |
| `04` | angle | `01 01 lo hi` on V10 (`mode 00 lo hi` on V12) | ✅ |
| `08` | reset profile | `mode` | ✅ |
| `09` | forces | `01 spring damping friction inertia` | ✅ echoes the 4 values |
| `0B` / `0C` | LED colours (5 RGB slots each) | `mode r g b ×5` | ❔ |
| `0D` / `0E` | pedal curve points 0–3 / 4–7 (V12 8-point form) | `mode pedal (x y)×4` | ❔ (V10 uses pages 3–5) |
| `0F` | centre calibration | `01` start, `02` finish | ❔ |
| `10` | road sensitivity | `mode value` | ❔ |
| `11` | fan level | `level` | ❔ |
| `12` | double clutch | `0/1` | ❌ |
| `13` | virtual key press | 3 bitmask bytes | ❔ |
| `15` | dashboard telemetry | `0/1` | ❌ |
| `06` `07` `0A` | pedal / shifter / panel update | unknown payloads | ❔ |

All values in the vendor UI are multiples of 10. Angle runs 90–1080 on
"large angle" models (the V987 is one) and 90–900 otherwise. Forces and FFB run 0–100.

## Test mode — `54 01` enter, `54 02` exit → stream of `D4 03 …`

| Byte | Content |
|---|---|
| 2 | bits 0–7 → codes 0–7 |
| 3 | bits 0–7 → codes 8–15 |
| 4 | bits 0–1 → codes 16, 17 |
| 6 | steering angle (u8, scaled) |
| 8 / 9 / 10 | throttle / brake / clutch (u8) |
| 12 | bits 0–3 → codes 25, 26, 23, 24 (handbrake, LO-HI, gear-up, gear-down) |
| 13 | gear: 1–6, 255 = reverse, 0 = neutral |

## Key codes

| Code | Control | Code | Control |
|---|---|---|---|
| 0 | View / Back | 16 | Home |
| 1 | L3 (SL) | 17 | Share |
| 2 | R3 (SR) | 20 | Throttle |
| 3 | Menu / Start | 21 | Brake |
| 4–7 | D-pad up / right / down / left | 22 | Clutch |
| 8 | Y | 23 / 24 | gear up / gear down (paddles) |
| 9 | B | 25 | shifter left button (P / handbrake) |
| 10 | A | 26 | shifter right button (LO⇌HI) |
| 11 | X | 27–32 | gears 1–6 |
| 12 / 13 | L1 (LB) / R1 (RB) | 33 | reverse gear |
| 14 / 15 | L2 (LT) / R2 (RT) | 34 / 35 | steering axis − / + |
| 47 | reserved | 254 | disabled / unbound |

Remap targets also include keyboard keys 64–156 (`A`=64 … `Z`=89, `1`=90 … `0`=99,
`F1`=100 … `F12`=111, `Esc`=112, …, numpad 141–156) and mouse buttons 48–50 (left/right/middle).
That is what the wheel's second USB interface (keyboard + mouse) is for.

## Vendor presets (online)

`GET https://api.e-pxn.com.cn/mobile/v1/keyboard/getKeyboard?pageNum=1&pageSize=50&status=1&type=1&driveType=<20|21|22>`
(20 = PC, 21 = PS4, 22 = Xbox). Each entry's `keboardValue` [sic] is hex:
`mode(1)` + key map + `angle u16 LE` + `ffb spring damping friction inertia` + optional `reverse` + optional pedal curves (3 × 4 points × 2 u16).
The key map order comes from the vendor app (`DetailActivity.KEYMAP_SEQUENCE`): mode byte 0 uses
`0–13, 17, 16, 47, 14, 15, 20–35` (35 bytes); modes 3/5/8 repeat code 47 after 15 (36 bytes).
PS4/Xbox blobs are 44 bytes (no reverse); Xbox ones store 0 for every force, meaning "not set".
Wheel Studio stores these as TOML presets (see PRESETS.md).

**Vendor app bug.** When LiteStar's phone app applies a mode 3/5/8 preset, it writes the 36-byte V2
key map into pages that hold 35 keys, so page 2 ends up two bytes late. After applying the ETS2
preset a V987 reported `d1 01 02 22 23 22 23 84 03 64 1e 50 00 00`: angle `0x2322` (8994°),
FFB 132, spring 3, damping 100, friction 30, inertia 80. The intended values were 900°, FFB 100,
spring 30, damping 80, friction 80, inertia 80. Wheel Studio shows a warning with a repair button
when it reads values like these, and it applies presets with the individual `03`, `04` and `09`
commands, so it can't hit this bug.
