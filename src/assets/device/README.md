# Device illustrations

Hand-drawn, stylised SVG diagrams of the wheel set: `wheel.svg` (front), `shifter.svg` (top), and `pedals.svg` (front). The `*_preview.png` files are check renders on a neutral backdrop. The app does not use them.

## Highlighting controls

Every interactive control has `id="<id>"`, `class="ctl"` and `data-ctl="<id>"`.
Each control sets its base `fill` on itself, and its body shape inherits that fill. Labels, holes, screws and highlights set their own fills. A single CSS rule can therefore recolour the control body:

```css
.ctl { cursor: pointer; transition: fill .12s; }
.ctl:hover  { fill: color-mix(in srgb, var(--accent) 45%, transparent); }
.ctl.active { fill: var(--accent); }
```

This only works when the SVG is inlined into the DOM (for example `?raw` import + `innerHTML`). It does not work when the SVG is loaded through `<img>`. Exceptions:

- `led-bar` is filled with a gradient.
- On `shifter-knob`, only the ball recolours. The metal top face keeps its colour so the engraved gear pattern stays readable.

## Theme variables (all optional, with fallbacks)

| variable       | fallback  | used for                                   |
|----------------|-----------|--------------------------------------------|
| `--dev-body`   | `#1c1e22` | main matte body: rim, hub, shells, decks   |
| `--dev-edge`   | `#3a3d44` | outlines, seams, grooves, screws           |
| `--dev-ctl`    | `#2a2d33` | button caps, paddles, knob, gear markers   |
| `--dev-shade`  | `#121316` | recesses, holes, ledges, feet              |
| `--dev-carbon` | `#25272c` | wheel carbon spoke plate                   |
| `--dev-metal`  | `#b9bdc4` | pedal pads, knob face                      |
| `--dev-label`  | `#e6e7ea` | printed legends and icons                  |
| `--dev-marker` | `#d0141b` | rim top-centre stripe, status LED          |

Gradients and patterns in `<defs>` use white or black at low opacity, so they work on any base colour.

## wheel.svg (viewBox 0 0 600 600)

| id | what |
|----|------|
| `rim` | steering rim (textured grip zones are a subtle overlay) |
| `rim-marker` | red top-dead-centre stripe |
| `led-bar` | RGB light strip on top of the hub |
| `status-led` | small red dot below the bottom button row |
| `hub-logo` | round logo medallion in the hub centre |
| `paddle-left` / `paddle-right` | shift paddles behind the rim, either side of the hub |
| `dpad-up` / `dpad-down` / `dpad-left` / `dpad-right` | the four arms of the left D-pad |
| `btn-y` / `btn-x` / `btn-b` / `btn-a` | right face cluster: top Y, left X, right B, bottom A |
| `btn-lb` / `btn-lt` / `btn-sl` | left row under the D-pad, left to right: L1/LB, L2/LT, L3/SL |
| `btn-sr` / `btn-rt` / `btn-rb` | right row, left to right: R3/SR, R2/RT, R1/RB |
| `btn-back` | bottom centre row #1: SHARE / BACK |
| `btn-home` | bottom centre row #2: HOME (house icon) |
| `btn-prog` | bottom centre row #3: upload-arrow icon (PROG) |
| `btn-start` | bottom centre row #4: OPTIONS / START |

Non-interactive helpers: `base` (stand), `spokes` (carbon plate), `hub`.

## shifter.svg (viewBox 0 0 400 560)

| id | what |
|----|------|
| `shifter-body` | teardrop housing (including the foot) |
| `shifter-knob` | gear knob (ball + brushed-metal face) |
| `gate` | H-gate slot plate under the knob |
| `gear-1` / `gear-3` / `gear-5` | gear markers, top row of the engraved H on the knob face |
| `gear-2` / `gear-4` / `gear-6` / `gear-r` | gear markers, bottom row (R at far right) |
| `btn-handbrake` | (P) button, top-left |
| `btn-lohi` | LO/HI range button, top-right |

The gear markers sit at the end of each slot in the H pattern engraved on the knob face, which is also where the photo shows the pattern. The artwork is wrapped in one fitting `transform` group.

## pedals.svg (viewBox 0 0 600 420)

| id | what |
|----|------|
| `pedal-clutch` | left pedal |
| `pedal-brake` | middle pedal |
| `pedal-throttle` | right pedal (the taller one) |
| `base-plate` | sloped pedal deck with side walls and divider grooves |
| `heel-rest` | textured floor plate in front, with the smooth heel pad |
