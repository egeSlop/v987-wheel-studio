# Preset files

A preset is one TOML file. Wheel Studio reads them from three places, and a later one replaces an
earlier one with the same `id`:

| Where | What |
|---|---|
| built into the app (`src-tauri/presets/`) | LiteStar's presets as of the release, and Wheel Studio's own |
| `~/.local/share/v987-wheel-studio/presets/vendor/` | written by "Update from LiteStar" |
| `~/.local/share/v987-wheel-studio/presets/user/` | your own, saved from the Presets page |

On Windows the folder is `%APPDATA%\v987-wheel-studio\presets\`. "Open folder" on the Presets page
takes you there. Drop a `.toml` file into `user/` and it shows up the next time you open the page.

## Example

```toml
format = 1
id = "user-ets2-heavy"
name = "ETS2, heavy trucks"
platform = "pc"            # pc, ps4 or xbox
source = "user"            # vendor, studio or user
game = "Euro Truck Simulator 2"
tutorial = "https://..."   # optional link shown as "Guide / video"

[names]                    # the name in other languages, optional
tr = "ETS2, ağır tırlar"

[notes]                    # tips per language, shown under "Tip"
en = "Set 900° in the game too."
tr = "Oyunda da 900° seçin."

[[links]]                  # extra buttons on the card
label = "My setup video"
url = "https://..."

[wheel]                    # every key is optional; only the ones present are applied
angle = 900                # 90..1080
ffb = 100                  # 0..100
spring = 30
damping = 80
friction = 80
inertia = 80
reverse = false

[keymap]                   # only remapped controls: "code" = replacement (see src/lib/keys.ts)
"16" = 17

[ffb_server]               # loaded into the FFB server when the preset is applied
master_gain = 0.8

[ffb_server.feel]          # same keys as the [feel] table in ffb-server.toml
centering = 0.55
damping_engine_off = 0.65
```

`[vendor]` (`id`, `mode`, `raw`) appears only in LiteStar presets. `raw` is the original
`keboardValue` blob from LiteStar's server, kept so the decoding can be checked; the layout is in
[PROTOCOL.md](PROTOCOL.md#vendor-presets-online).

A LiteStar preset that has only `angle` under `[wheel]` is one where LiteStar stores 0 for every
force (the Xbox ones). Applying it changes only the angle, so it never turns force feedback off.
