import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface FoundDevice { id: string; name: string; rssi: number | null }
export interface DeviceState { mode: number; firmware: number; pedalState: number; leverState: number; angleMode: number }
export interface Params { angle: number; ffb: number; spring: number; damping: number; friction: number; inertia: number; reverse: boolean }
export type Curve = [number, number][];
export interface Profile {
  mode: number;
  keymap: [number, number][];
  params: Params;
  curves: [Curve, Curve, Curve];
  pagesOk: boolean[];
}
export interface Applied { acked: boolean; ack: string | null; readback: Profile | null }
export interface LiveInput { pressed: number[]; angle: number; throttle: number; brake: number; clutch: number; gear: number }
export interface LogEntry { dir: "tx" | "rx"; hex: string; note: string | null }

// ---- Presets (TOML files; see docs/PRESETS.md). snake_case mirrors the files.
export interface WheelParams { angle?: number; ffb?: number; spring?: number; damping?: number; friction?: number; inertia?: number; reverse?: boolean }
export interface Feel { centering: number; centering_parked: number; damping: number; damping_engine_off: number; bumps: number; impacts: number; texture: number; engine: number; soft_lock: number }
export interface FfbProfile { master_gain?: number; invert?: boolean; feel?: Feel }
export interface Preset {
  format: number; id: string; name: string; platform: string; source: "vendor" | "studio" | "user" | string;
  game?: string; tutorial?: string; names?: Record<string, string>; notes?: Record<string, string>;
  links?: { label: string; url: string }[]; wheel: WheelParams; keymap?: Record<string, number>;
  ffb_server?: FfbProfile; vendor?: { id: string; mode: number; raw: string }; origin: "bundled" | "downloaded" | "user" | string;
}
export interface PresetListing { presets: Preset[]; errors: string[]; dir: string }

// ---- FFB server
export interface FfbConfig { master_gain: number; invert: boolean; stale_ms: number; listen: string; device: string; autostart: boolean; feel: Feel }
export interface FfbForces { spring: number; damper: number; constant: number; texture: number; textureHz: number; engine: number; engineHz: number }
export interface FfbClient { id: number; peer: string; game: string; source: string; kind: string; frames: number; errors: number; lastError: string | null; connectedMs: number; lastFrameMs: number | null }
export interface FfbStatus {
  server: string; protocol: number; listen: string; wheel: { name: string; path: string } | null; wheelError: string | null;
  mode: "idle" | "telemetry" | "forces" | "test"; testStep: string | null; game: string | null; forces: FfbForces;
  clients: FfbClient[]; subscribers: number; configPath: string; restartNeeded: boolean;
}
export interface FfbState { running: boolean; error: string | null; external: boolean; status: FfbStatus | null; config: FfbConfig; configPath: string }

export const api = {
  scan: (millis = 4000) => invoke<FoundDevice[]>("scan", { millis }),
  connect: (id: string) => invoke<{ name: string; state: DeviceState | null }>("connect", { id }),
  disconnect: () => invoke<void>("disconnect"),
  readState: () => invoke<DeviceState | null>("read_state"),
  readProfile: (mode: number) => invoke<Profile>("read_profile", { mode }),
  setAngle: (mode: number, angle: number) => invoke<Applied>("set_angle", { mode, angle }),
  setFfb: (mode: number, ffb: number, reverse: boolean) => invoke<Applied>("set_ffb", { mode, ffb, reverse }),
  setForces: (mode: number, p: Pick<Params, "spring" | "damping" | "friction" | "inertia">) =>
    invoke<Applied>("set_forces", { mode, ...p }),
  setKeymap: (mode: number, target: number, replacement: number) =>
    invoke<Applied>("set_keymap", { mode, target, replacement }),
  writeCurve: (mode: number, pedal: number, curve: Curve) => invoke<Applied>("write_curve", { mode, pedal, curve }),
  setMode: (mode: number) => invoke<Applied>("set_mode", { mode }),
  resetProfile: (mode: number) => invoke<Applied>("reset_profile", { mode }),
  testMode: (on: boolean) => invoke<boolean>("test_mode", { on }),
  raw: (bytes: number[]) => invoke<string | null>("raw", { bytes }),
  listPresets: () => invoke<PresetListing>("list_presets"),
  refreshVendorPresets: () => invoke<PresetListing>("refresh_vendor_presets"),
  saveUserPreset: (preset: Preset) => invoke<Preset>("save_user_preset", { preset }),
  deleteUserPreset: (id: string) => invoke<void>("delete_user_preset", { id }),
  ffbState: () => invoke<FfbState>("ffb_state"),
  ffbStart: () => invoke<FfbState>("ffb_start"),
  ffbStop: () => invoke<FfbState>("ffb_stop"),
  ffbSetConfig: (config: FfbConfig) => invoke<FfbState>("ffb_set_config", { config }),
  ffbTest: () => invoke<boolean>("ffb_test"),
};

export const on = {
  live: (f: (l: LiveInput) => void): Promise<UnlistenFn> => listen<LiveInput>("live", (e) => f(e.payload)),
  log: (f: (l: LogEntry) => void): Promise<UnlistenFn> => listen<LogEntry>("log", (e) => f(e.payload)),
  connection: (f: (s: string) => void): Promise<UnlistenFn> => listen<string>("connection", (e) => f(e.payload)),
};
