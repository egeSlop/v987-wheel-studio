// Wire codes for every control, their printed labels, and the SVG element that draws them.
// Source: com.star.v10.extension.KeyCode + the V987 manual's D-Input table.

export interface Control {
  code: number;
  /** Label printed on the hardware / in the manual. */
  label: string;
  /** Element id in the device SVGs. */
  svg: string;
  /** i18n key describing where it is. */
  where: "wheel" | "shifter" | "pedals";
  /** D-Input button number from the manual (1-based), if any. */
  dinput?: number;
}

export const CONTROLS: Control[] = [
  { code: 10, label: "A", svg: "btn-a", where: "wheel", dinput: 1 },
  { code: 9, label: "B", svg: "btn-b", where: "wheel", dinput: 2 },
  { code: 11, label: "X", svg: "btn-x", where: "wheel", dinput: 3 },
  { code: 8, label: "Y", svg: "btn-y", where: "wheel", dinput: 4 },
  { code: 12, label: "LB", svg: "btn-lb", where: "wheel", dinput: 5 },
  { code: 13, label: "RB", svg: "btn-rb", where: "wheel", dinput: 6 },
  { code: 14, label: "LT", svg: "btn-lt", where: "wheel", dinput: 7 },
  { code: 15, label: "RT", svg: "btn-rt", where: "wheel", dinput: 8 },
  { code: 0, label: "BACK", svg: "btn-back", where: "wheel", dinput: 9 },
  { code: 3, label: "START", svg: "btn-start", where: "wheel", dinput: 10 },
  { code: 1, label: "SL", svg: "btn-sl", where: "wheel", dinput: 11 },
  { code: 2, label: "SR", svg: "btn-sr", where: "wheel", dinput: 12 },
  { code: 16, label: "HOME", svg: "btn-home", where: "wheel", dinput: 13 },
  { code: 17, label: "SHARE", svg: "btn-prog", where: "wheel" },
  { code: 4, label: "▲", svg: "dpad-up", where: "wheel" },
  { code: 5, label: "▶", svg: "dpad-right", where: "wheel" },
  { code: 6, label: "▼", svg: "dpad-down", where: "wheel" },
  { code: 7, label: "◀", svg: "dpad-left", where: "wheel" },
  { code: 23, label: "+", svg: "paddle-right", where: "wheel" },
  { code: 24, label: "−", svg: "paddle-left", where: "wheel" },
  { code: 25, label: "P", svg: "btn-handbrake", where: "shifter", dinput: 16 },
  { code: 26, label: "LO⇌HI", svg: "btn-lohi", where: "shifter", dinput: 24 },
  { code: 27, label: "1", svg: "gear-1", where: "shifter", dinput: 17 },
  { code: 28, label: "2", svg: "gear-2", where: "shifter", dinput: 18 },
  { code: 29, label: "3", svg: "gear-3", where: "shifter", dinput: 19 },
  { code: 30, label: "4", svg: "gear-4", where: "shifter", dinput: 20 },
  { code: 31, label: "5", svg: "gear-5", where: "shifter", dinput: 21 },
  { code: 32, label: "6", svg: "gear-6", where: "shifter", dinput: 22 },
  { code: 33, label: "R", svg: "gear-r", where: "shifter", dinput: 23 },
  { code: 20, label: "GAS", svg: "pedal-throttle", where: "pedals" },
  { code: 21, label: "BRK", svg: "pedal-brake", where: "pedals" },
  { code: 22, label: "CLT", svg: "pedal-clutch", where: "pedals" },
];

export const controlByCode = new Map(CONTROLS.map((c) => [c.code, c]));
export const controlBySvg = new Map(CONTROLS.map((c) => [c.svg, c]));

export const DISABLED = 254;

export interface Target { code: number; label: string; group: "pad" | "keyboard" | "mouse" | "off" }

const pad: Target[] = CONTROLS.map((c) => ({ code: c.code, label: c.label, group: "pad" as const }));

const kb: [number, string][] = [
  ...Array.from({ length: 26 }, (_, i) => [64 + i, String.fromCharCode(65 + i)] as [number, string]),
  ...Array.from({ length: 9 }, (_, i) => [90 + i, String(i + 1)] as [number, string]),
  [99, "0"],
  ...Array.from({ length: 12 }, (_, i) => [100 + i, `F${i + 1}`] as [number, string]),
  [112, "Esc"], [113, "`"], [114, "-"], [115, "="], [116, "Tab"], [117, "Caps"],
  [118, "L-Ctrl"], [119, "L-Shift"], [120, "L-Alt"], [121, "L-Win"],
  [122, "R-Ctrl"], [123, "R-Shift"], [124, "R-Alt"], [125, "R-Win"],
  [126, "Enter"], [127, "Backspace"], [128, "["], [129, "]"], [130, "\\"], [131, ";"], [132, "'"],
  [133, ","], [134, "."], [135, "/"], [136, "Space"],
  [137, "↑"], [138, "↓"], [139, "←"], [140, "→"],
  [141, "Num /"], [142, "Num *"], [143, "Num -"], [144, "Num +"], [145, "Num Enter"], [146, "Num ."],
  ...Array.from({ length: 10 }, (_, i) => [147 + i, `Num ${i}`] as [number, string]),
];

export const TARGETS: Target[] = [
  ...pad,
  ...kb.map(([code, label]) => ({ code, label, group: "keyboard" as const })),
  { code: 48, label: "Mouse L", group: "mouse" },
  { code: 49, label: "Mouse R", group: "mouse" },
  { code: 50, label: "Mouse M", group: "mouse" },
  { code: DISABLED, label: "—", group: "off" },
];

export const targetByCode = new Map(TARGETS.map((t) => [t.code, t]));

export function targetLabel(code: number): string {
  return targetByCode.get(code)?.label ?? `#${code}`;
}

export const MODES: Record<number, string> = {
  1: "D-Input", 2: "X-Input", 3: "PS3 HID", 4: "PS3", 5: "PS4", 6: "PS4 pad",
  7: "Xbox 360", 8: "Xbox One", 9: "Xbox One pad", 10: "Android", 11: "Switch",
};
