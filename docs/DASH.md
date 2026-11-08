# Tablet dashboard

Wheel Studio runs a small web server on your network. Open it on a tablet or phone and you get
a car-radio screen with live data from the game, plus a panel of buttons that press the game's
keys on the PC. Every game uses the same page; a profile (one TOML file per game) decides what
the radio does, which readouts appear and which buttons there are.

This page is for people who want to add a game: send data from a mod or plugin, and write a
profile for it. Nothing here needs changes to Wheel Studio itself.

- [Ports](#ports)
- [Sending data](#sending-data)
- [Fields](#fields)
- [Profiles](#profiles)
- [Tablet protocol](#tablet-protocol)
- [Built-in games](#built-in-games)

## Ports

| Port | Protocol | What |
|---|---|---|
| 47821 | HTTP + WebSocket, all interfaces | The tablet page (`/`), tablets (`/ws`), mods (`/feed`) |
| 47822 | UDP, localhost only | JSON datagrams from game plugins |
| 47820 | WebSocket, localhost only | The FFB server. Its telemetry reaches the dashboard too |
| 20777 | UDP, localhost | DiRT Rally 2.0's own telemetry output |
| 9996 | UDP | Assetto Corsa's remote telemetry (Wheel Studio asks the game for it) |

The settings live in `~/.config/v987-wheel-studio/dash.toml` (Windows:
`%APPDATA%\v987-wheel-studio\dash.toml`): port, PIN, whether a PIN is required, autostart.

## Sending data

Pick whichever transport your game's modding allows. All three take the same JSON object.

**UDP datagram** to `127.0.0.1:47822`, one JSON object per datagram. Easiest from a native
plugin. This is what the ETS2/ATS plugin does.

```json
{"type":"dash","game":"mygame","name":"My Game","speedKmh":64.2,"gear":"4","rpm":3100,"maxRpm":6500,"lightsLow":true}
```

**WebSocket** to `ws://127.0.0.1:47821/feed`. Send the same objects as text messages. You get
`{"type":"welcome",...}` on connect and `{"type":"error","message":"..."}` if a message is
rejected.

**Through the FFB server.** If your mod already sends telemetry to the FFB server
(`ws://127.0.0.1:47820/telemetry`, see
[FFB-SERVER.md](https://github.com/egeSlop/ffb-server/blob/main/docs/FFB-SERVER.md)), add the
dashboard fields to the same telemetry message. The FFB server ignores fields it doesn't know
and passes them on, and the dashboard picks them up. `speed` (m/s) becomes `speedKmh`
automatically. FS22_WheelFFB works this way.

Rules:

- `game` is required: a short id that a profile can match (`"mygame"`). `name` is what the
  tablet shows.
- Send 10 to 30 times a second while driving. Each message replaces the previous one, so always
  send every field you have.
- Send `"active": false` in menus and pauses (the tablet shows "Paused"). A game that stops
  sending disappears from the tablet after a minute.
- Unknown fields are kept, so a profile can show anything you send.

## Fields

All optional. Use these names when they fit so the built-in readouts and indicator lights work.

| Field | Type | Meaning |
|---|---|---|
| `speedKmh` | number | Speed, km/h, always positive (`speed` in m/s also works) |
| `gear` | string or number | `"N"`, `"R"`, `"R2"`, `"5"`. Numbers: 0 = N, negative = reverse |
| `gearGroup` | string | Range/group, shown next to the gear (`"L"`, `"H"`) |
| `rpm`, `maxRpm` | number | Engine speed and redline |
| `fuel` | number | Tank level 0..1 |
| `engine` | bool | Engine running |
| `cruise` | number | Cruise control speed, km/h; 0 = off (`cruiseOn` is filled in) |
| `speedLimit` | number | km/h, 0 = none |
| `lightsLow`, `lightsHigh`, `lightsParking` | bool | Headlights |
| `lightsWorkFront`, `lightsWorkBack` | bool | Work lights |
| `beacon`, `hazard` | bool | Beacon, hazard lights |
| `blinkerLeft`, `blinkerRight` | bool | Indicator switch (not the blinking lamp) |
| `parkingBrake`, `wipers` | bool | |
| `vehicle` | string | Vehicle name, shown on the radio display |
| `lap`, `lapTime`, `lastLap`, `bestLap` | number | Lap count; times in seconds |
| `progress` | number | Stage or race progress 0..1 |
| `active` | bool | false = menu or paused |

## Profiles

A profile is a TOML file. Built-in ones are in
[`src-tauri/dash-profiles`](../src-tauri/dash-profiles). Put yours in
`~/.config/v987-wheel-studio/dash/` (Windows: `%APPDATA%\v987-wheel-studio\dash\`). Wheel Studio
has a button that opens the folder. Files are reloaded within a few seconds of saving. A file
whose `id` matches a built-in profile replaces it, which is how you change the keys of a
built-in game.

```toml
id = "mygame"
name = "My Game"
match = ["mygame", "my game"]   # compared with the game id/name the data reports
theme = "race"                  # tractor | truck | race | rally | default
accent = "#e53935"

[setup]                         # shown while waiting for the game
en = "Install the MyGame dashboard mod."
tr = "MyGame panel modunu kurun."

[radio]
mode = "tablet"                 # "game": radio buttons press the game's keys; "tablet": the tablet plays internet radio
# power = { key = "r" }
# next  = { key = "pagedown" }
# prev  = { key = "pageup" }

[[readout]]                     # small boxes under the speedometer
field = "lapTime"
label = "Lap time"
labels = { tr = "Tur süresi" }
format = "laptime"              # or "text"; numbers otherwise

[[readout]]
field = "fuel"
label = "Fuel"
unit = "%"
scale = 100                     # multiply before showing
decimals = 0

[[page]]                        # pages of buttons; tabs appear when there is more than one
name = "Car"
names = { tr = "Araç" }

  [[page.button]]
  id = "lights"                 # unique within the profile
  label = "Lights"
  labels = { tr = "Farlar" }
  icon = "lights"
  key = "l"                     # key or combo: "lctrl+l"
  active = "lightsLow"          # field that lights the button up
  tone = "ok"                   # ok | warn | danger: color when lit

  [[page.button]]
  id = "horn"
  label = "Horn"
  icon = "horn"
  key = "h"
  hold = true                   # key stays down while the button is held
```

**Keys** are lowercase names: `a`–`z`, `0`–`9`, `f1`–`f12`, `space`, `enter`, `esc`, `tab`,
`backspace`, `lshift`, `rshift`, `lctrl`, `rctrl`, `lalt`, `ralt`, `up`, `down`, `left`, `right`,
`home`, `end`, `pageup`, `pagedown`, `insert`, `delete`, `kp0`–`kp9`, `kpplus`, `kpminus`,
`kpmultiply`, `kpdivide`, `kpenter`, `kpperiod`, `minus`, `equal`, `bracketleft`,
`bracketright`, `semicolon`, `apostrophe`, `grave`, `backslash`, `comma`, `period`, `slash`,
`less`, `capslock`, `numlock`, `scrolllock`, `print`, `pause`. Combine with `+`.

**Game bindings.** With `bindings = "fs22"`, `"ets2"` or `"ats"`, a button can name the game's
own control in `action` and Wheel Studio reads the key from your game settings
(`inputBinding.xml` for FS22, `controls.sii` for the truck games). Rebinding in the game then
just works; `key` is only the fallback.

**Icons:** `engine lights highbeam beacon left right hazard horn cruise worklight camera robot
swap door attach detach select lower power fold tip pipe cover seed map shop menu help slower
faster pause parkbrake wipers lock save abs tc lookback plus minus mirror hud display parking`.

**Radio stations** for the tablet radio come from `dash/stations.toml` and from FS22's
`streamingInternetRadios.xml` if FS22 is installed:

```toml
[[station]]
name = "Radio Paradise"
url = "https://stream.radioparadise.com/mp3-128"
```

## Tablet protocol

For people building a different front end. WebSocket `ws://<pc>:47821/ws`, JSON text messages.

Server to tablet:

- `{"type":"hello","app","version","paired","pinRequired","profiles":[...],"stations":[...],"gameStations":{"fs22":[...]}}`
  on connect and whenever profiles change.
- `{"type":"state","game":{"profile","game","name","source","active","ageMs","data":{...}}}` up
  to 15 times a second while data changes, at least once a second. `game` is null when nothing
  is sending.
- `{"type":"auth","ok":true|false,"reason":"wrong-pin"|"pin-changed"|""}`
- `{"type":"pressed","id":N,"ok":true|false,"error":...}` after each press.

Tablet to server:

- `{"type":"auth","pin":"123456"}`. Five wrong PINs on one connection slow it down.
- `{"type":"press","id":N,"profile":"ets2","button":"lights"}` taps a button;
  add `"down":true` / `"down":false` for hold buttons. Held keys are released if the tablet
  disconnects.
- `{"type":"radio","id":N,"profile":"fs22","what":"power|next|prev|volUp|volDown"}`

The tablet only names a profile button. It can't send arbitrary keys, so a paired tablet can
only press what the profiles define.

## Built-in games

| Game | Data | Keys | Status |
|---|---|---|---|
| Farming Simulator 22 | FS22_WheelFFB 1.7+ through the FFB server | From your FS22 bindings | Tested |
| Euro Truck Simulator 2 | v987-dash SDK plugin (Wheel Studio installs it) | From your ETS2 profile | Plugin tested with a fake host |
| American Truck Simulator | Same plugin | From your ATS profile | Untested |
| Assetto Corsa | Built-in UDP telemetry, nothing to install | AC defaults; check the lights key | Untested |
| DiRT Rally 2.0 | Built-in UDP telemetry; Wheel Studio switches it on | Game defaults | Untested |

On Linux, key presses go through a virtual keyboard (`/dev/uinput`). Most distros give the
logged-in user access to it, and Steam's udev rules do too. On Windows they go through
SendInput. Either way the game window must have focus, as with a real keyboard.
