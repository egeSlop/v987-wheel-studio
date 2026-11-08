//! Key presses for the tablet's button panel.
//!
//! Linux: a virtual keyboard through /dev/uinput (Fedora and most distros give the logged-in
//! user access to it; Steam's udev rules do too). Windows: SendInput with scan codes, which
//! DirectInput games read. The game must have focus, as with a real keyboard.
//!
//! Keys are named in lowercase ("f", "kp1", "lctrl", "pageup"); a combo is "lctrl+f".
//! FS22's binding names ("KEY_KP_1", "KEY_lctrl KEY_f") are accepted too.

use std::sync::Mutex;
use std::time::Duration;

/// (name, Linux evdev code). Codes 1..=88 equal PC set-1 scan codes; the rest are mapped in
/// `win_scan`.
const KEYS: &[(&str, u16)] = &[
    ("esc", 1), ("1", 2), ("2", 3), ("3", 4), ("4", 5), ("5", 6), ("6", 7), ("7", 8), ("8", 9),
    ("9", 10), ("0", 11), ("minus", 12), ("equal", 13), ("backspace", 14), ("tab", 15),
    ("q", 16), ("w", 17), ("e", 18), ("r", 19), ("t", 20), ("y", 21), ("u", 22), ("i", 23),
    ("o", 24), ("p", 25), ("bracketleft", 26), ("bracketright", 27), ("enter", 28),
    ("lctrl", 29), ("a", 30), ("s", 31), ("d", 32), ("f", 33), ("g", 34), ("h", 35), ("j", 36),
    ("k", 37), ("l", 38), ("semicolon", 39), ("apostrophe", 40), ("grave", 41), ("lshift", 42),
    ("backslash", 43), ("z", 44), ("x", 45), ("c", 46), ("v", 47), ("b", 48), ("n", 49),
    ("m", 50), ("comma", 51), ("period", 52), ("slash", 53), ("rshift", 54), ("kpmultiply", 55),
    ("lalt", 56), ("space", 57), ("capslock", 58), ("f1", 59), ("f2", 60), ("f3", 61),
    ("f4", 62), ("f5", 63), ("f6", 64), ("f7", 65), ("f8", 66), ("f9", 67), ("f10", 68),
    ("numlock", 69), ("scrolllock", 70), ("kp7", 71), ("kp8", 72), ("kp9", 73),
    ("kpminus", 74), ("kp4", 75), ("kp5", 76), ("kp6", 77), ("kpplus", 78), ("kp1", 79),
    ("kp2", 80), ("kp3", 81), ("kp0", 82), ("kpperiod", 83), ("less", 86), ("f11", 87),
    ("f12", 88), ("kpenter", 96), ("rctrl", 97), ("kpdivide", 98), ("print", 99), ("ralt", 100),
    ("home", 102), ("up", 103), ("pageup", 104), ("left", 105), ("right", 106), ("end", 107),
    ("down", 108), ("pagedown", 109), ("insert", 110), ("delete", 111), ("pause", 119),
];

const ALIASES: &[(&str, &str)] = &[
    ("return", "enter"), ("escape", "esc"), ("ctrl", "lctrl"), ("control", "lctrl"),
    ("shift", "lshift"), ("alt", "lalt"), ("del", "delete"), ("ins", "insert"),
    ("pgup", "pageup"), ("pgdn", "pagedown"), ("pagedn", "pagedown"), ("kpadd", "kpplus"),
    ("kpsubtract", "kpminus"), ("kpslash", "kpdivide"), ("kpasterisk", "kpmultiply"),
    ("kpdecimal", "kpperiod"), ("kpdelete", "kpperiod"), ("leftbracket", "bracketleft"),
    ("rightbracket", "bracketright"), ("[", "bracketleft"), ("]", "bracketright"),
    (";", "semicolon"), ("'", "apostrophe"), (",", "comma"), (".", "period"), ("/", "slash"),
    ("\\", "backslash"), ("-", "minus"), ("=", "equal"), ("`", "grave"), ("<", "less"),
    ("lessgreater", "less"), ("printscreen", "print"), ("sysrq", "print"), ("caps", "capslock"),
    // SCS (ETS2/ATS) controls.sii names
    ("lbracket", "bracketleft"), ("rbracket", "bracketright"), ("scrollock", "scrolllock"),
    ("uarrow", "up"), ("darrow", "down"), ("larrow", "left"), ("rarrow", "right"),
    ("numenter", "kpenter"), ("numslash", "kpdivide"), ("nummultiply", "kpmultiply"),
    ("numplus", "kpplus"), ("numminus", "kpminus"), ("numperiod", "kpperiod"), ("numlck", "numlock"),
];

pub fn code_of(name: &str) -> Option<u16> {
    let mut n = name.trim().to_ascii_lowercase();
    if let Some(s) = n.strip_prefix("key_") {
        n = s.to_string();
    }
    n = n.replace('_', "");
    // SCS: key1..key0 are the number row, num0..num9 the keypad.
    if let Some(d) = n.strip_prefix("key").filter(|d| d.len() == 1 && d.chars().all(|c| c.is_ascii_digit())) {
        n = d.to_string();
    } else if let Some(d) = n.strip_prefix("num").filter(|d| d.len() == 1 && d.chars().all(|c| c.is_ascii_digit())) {
        n = format!("kp{d}");
    }
    if let Some((_, to)) = ALIASES.iter().find(|(a, _)| *a == n) {
        n = to.to_string();
    }
    KEYS.iter().find(|(k, _)| *k == n).map(|(_, c)| *c)
}

/// "lctrl+f", "KEY_lctrl KEY_f", "ctrl + F" -> [29, 33]. Modifiers first, as typed.
pub fn parse_combo(s: &str) -> Result<Vec<u16>, String> {
    let parts: Vec<&str> = s.split(|c: char| c == '+' || c.is_whitespace()).filter(|p| !p.is_empty()).collect();
    // "kp+" style: a trailing '+' means the plus key itself.
    if parts.is_empty() {
        return Err(format!("empty key \"{s}\""));
    }
    parts.iter().map(|p| code_of(p).ok_or_else(|| format!("unknown key \"{p}\""))).collect()
}


// ---------------------------------------------------------------- injection

pub struct Keyboard {
    inner: Mutex<Option<imp::Device>>,
    error: Mutex<Option<String>>,
}

impl Keyboard {
    pub const fn new() -> Self {
        Self { inner: Mutex::new(None), error: Mutex::new(None) }
    }

    fn with<R>(&self, f: impl FnOnce(&mut imp::Device) -> std::io::Result<R>) -> Result<R, String> {
        let mut g = self.inner.lock().unwrap();
        if g.is_none() {
            match imp::Device::open() {
                Ok(d) => {
                    *g = Some(d);
                    *self.error.lock().unwrap() = None;
                }
                Err(e) => {
                    let msg = imp::explain(&e);
                    *self.error.lock().unwrap() = Some(msg.clone());
                    return Err(msg);
                }
            }
        }
        let r = f(g.as_mut().unwrap());
        if let Err(e) = &r {
            *g = None; // reopen next time
            *self.error.lock().unwrap() = Some(e.to_string());
        }
        r.map_err(|e| e.to_string())
    }

    /// Creates the virtual keyboard now (so the compositor knows it before the first press).
    pub fn warm_up(&self) -> Result<(), String> {
        self.with(|_| Ok(()))
    }

    pub fn error(&self) -> Option<String> {
        self.error.lock().unwrap().clone()
    }

    pub fn ready(&self) -> bool {
        self.inner.lock().unwrap().is_some()
    }

    pub fn down(&self, keys: &[u16]) -> Result<(), String> {
        self.with(|d| {
            for &k in keys {
                d.key(k, true)?;
                d.sync()?;
                // Some games poll the keyboard once a frame; a modifier must be seen before
                // the key it modifies.
                if keys.len() > 1 {
                    std::thread::sleep(Duration::from_millis(12));
                }
            }
            Ok(())
        })
    }

    pub fn up(&self, keys: &[u16]) -> Result<(), String> {
        self.with(|d| {
            for &k in keys.iter().rev() {
                d.key(k, false)?;
                d.sync()?;
            }
            Ok(())
        })
    }

    /// Press and release, held long enough for a 30 fps game to notice.
    pub fn tap(&self, keys: &[u16]) -> Result<(), String> {
        self.down(keys)?;
        std::thread::sleep(Duration::from_millis(70));
        self.up(keys)
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use std::fs::{File, OpenOptions};
    use std::io::Write;
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;

    const UI_SET_EVBIT: libc::c_ulong = 0x4004_5564;
    const UI_SET_KEYBIT: libc::c_ulong = 0x4004_5565;
    const UI_DEV_SETUP: libc::c_ulong = 0x405c_5503;
    const UI_DEV_CREATE: libc::c_ulong = 0x5501;
    const UI_DEV_DESTROY: libc::c_ulong = 0x5502;
    const EV_SYN: u16 = 0;
    const EV_KEY: u16 = 1;

    #[repr(C)]
    struct UinputSetup {
        bustype: u16,
        vendor: u16,
        product: u16,
        version: u16,
        name: [u8; 80],
        ff_effects_max: u32,
    }

    pub struct Device {
        file: File,
    }

    impl Device {
        pub fn open() -> std::io::Result<Self> {
            let file = OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open("/dev/uinput")?;
            let fd = file.as_raw_fd();
            let chk = |r: libc::c_int| if r < 0 { Err(std::io::Error::last_os_error()) } else { Ok(()) };
            unsafe {
                chk(libc::ioctl(fd, UI_SET_EVBIT as _, EV_KEY as libc::c_int))?;
                for (_, code) in super::KEYS {
                    chk(libc::ioctl(fd, UI_SET_KEYBIT as _, *code as libc::c_int))?;
                }
                let mut s = UinputSetup { bustype: 0x06, vendor: 0x1209, product: 0x0987, version: 1, name: [0; 80], ff_effects_max: 0 };
                let n = b"V987 Wheel Studio tablet keys";
                s.name[..n.len()].copy_from_slice(n);
                chk(libc::ioctl(fd, UI_DEV_SETUP as _, &s as *const UinputSetup))?;
                chk(libc::ioctl(fd, UI_DEV_CREATE as _))?;
            }
            // Give the compositor a moment to pick the new keyboard up.
            std::thread::sleep(std::time::Duration::from_millis(300));
            Ok(Self { file })
        }

        fn emit(&mut self, ty: u16, code: u16, value: i32) -> std::io::Result<()> {
            let mut ev = [0u8; 24];
            ev[16..18].copy_from_slice(&ty.to_ne_bytes());
            ev[18..20].copy_from_slice(&code.to_ne_bytes());
            ev[20..24].copy_from_slice(&value.to_ne_bytes());
            self.file.write_all(&ev)
        }

        pub fn key(&mut self, code: u16, down: bool) -> std::io::Result<()> {
            self.emit(EV_KEY, code, down as i32)
        }

        pub fn sync(&mut self) -> std::io::Result<()> {
            self.emit(EV_SYN, 0, 0)
        }
    }

    impl Drop for Device {
        fn drop(&mut self) {
            unsafe { libc::ioctl(self.file.as_raw_fd(), UI_DEV_DESTROY as _) };
        }
    }

    pub fn explain(e: &std::io::Error) -> String {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            "No access to /dev/uinput, so the tablet can't press keys. Install your distro's steam-devices (or game-devices-udev) package, or add a udev rule: KERNEL==\"uinput\", TAG+=\"uaccess\".".into()
        } else if e.kind() == std::io::ErrorKind::NotFound {
            "/dev/uinput is missing. Load the module: sudo modprobe uinput".into()
        } else {
            format!("virtual keyboard: {e}")
        }
    }
}

#[cfg(windows)]
mod imp {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
        KEYEVENTF_SCANCODE, SendInput, VIRTUAL_KEY,
    };

    pub struct Device;

    /// Linux code -> (set-1 scan code, extended).
    fn win_scan(code: u16) -> (u16, bool) {
        match code {
            1..=88 => (code, false),
            96 => (0x1C, true),
            97 => (0x1D, true),
            98 => (0x35, true),
            99 => (0x37, true),
            100 => (0x38, true),
            102 => (0x47, true),
            103 => (0x48, true),
            104 => (0x49, true),
            105 => (0x4B, true),
            106 => (0x4D, true),
            107 => (0x4F, true),
            108 => (0x50, true),
            109 => (0x51, true),
            110 => (0x52, true),
            111 => (0x53, true),
            119 => (0x45, false), // Pause: close enough for games, which read it as a scan code
            _ => (0, false),
        }
    }

    impl Device {
        pub fn open() -> std::io::Result<Self> {
            Ok(Device)
        }

        pub fn key(&mut self, code: u16, down: bool) -> std::io::Result<()> {
            let (scan, ext) = win_scan(code);
            let mut flags: KEYBD_EVENT_FLAGS = KEYEVENTF_SCANCODE;
            if ext {
                flags |= KEYEVENTF_EXTENDEDKEY;
            }
            if !down {
                flags |= KEYEVENTF_KEYUP;
            }
            let input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VIRTUAL_KEY(0), wScan: scan, dwFlags: flags, time: 0, dwExtraInfo: 0 } },
            };
            let n = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
            if n == 1 { Ok(()) } else { Err(std::io::Error::last_os_error()) }
        }

        pub fn sync(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    pub fn explain(e: &std::io::Error) -> String {
        format!("SendInput: {e}")
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod imp {
    pub struct Device;
    impl Device {
        pub fn open() -> std::io::Result<Self> {
            Err(std::io::Error::other("key presses are not supported on this system"))
        }
        pub fn key(&mut self, _: u16, _: bool) -> std::io::Result<()> {
            Ok(())
        }
        pub fn sync(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    pub fn explain(e: &std::io::Error) -> String {
        e.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_names_and_fs22_bindings() {
        assert_eq!(parse_combo("lctrl+f").unwrap(), vec![29, 33]);
        assert_eq!(parse_combo("KEY_lctrl KEY_f").unwrap(), vec![29, 33]);
        assert_eq!(parse_combo("KEY_KP_1").unwrap(), vec![79]);
        assert_eq!(parse_combo("KEY_return").unwrap(), vec![28]);
        assert_eq!(parse_combo("KEY_KP_plus").unwrap(), vec![78]);
        assert_eq!(parse_combo("Ctrl + A").unwrap(), vec![29, 30]);
        assert_eq!(parse_combo("[").unwrap(), vec![26]);
        assert!(parse_combo("MOUSE_BUTTON_LEFT").is_err());
    }
}
