import { api, on, type Curve, type DeviceState, type FoundDevice, type LiveInput, type LogEntry, type Params, type Profile } from "./api";

export type ApplyState = "sending" | "acked" | "verified" | "failed";
export type ParamKey = keyof Params;

export const app = $state({
  phase: "idle" as "idle" | "scanning" | "connecting" | "connected" | "lost",
  found: [] as FoundDevice[],
  deviceId: "",
  name: "",
  error: "",
  state: null as DeviceState | null,
  profile: null as Profile | null,
  /** Values as found when we connected, for "revert". */
  original: null as Profile | null,
  status: {} as Record<string, ApplyState>,
  liveOn: false,
  live: null as LiveInput | null,
  log: [] as (LogEntry & { t: number })[],
});

export const mode = () => app.state?.mode ?? 1;

let listenersReady = false;
async function ensureListeners() {
  if (listenersReady) return;
  listenersReady = true;
  await on.log((e) => {
    app.log.push({ ...e, t: Date.now() });
    if (app.log.length > 400) app.log.splice(0, app.log.length - 400);
  });
  await on.live((l) => (app.live = l));
  await on.connection((s) => {
    if (s === "disconnected" && app.phase === "connected") {
      app.phase = "lost";
      app.liveOn = false;
    }
  });
}

export async function scan() {
  await ensureListeners();
  app.error = "";
  app.phase = "scanning";
  try {
    app.found = await api.scan(4000);
  } catch (e) {
    app.error = String(e);
  }
  if (app.phase === "scanning") app.phase = "idle";
}

export async function connect(dev: FoundDevice) {
  await ensureListeners();
  app.error = "";
  app.phase = "connecting";
  try {
    const r = await api.connect(dev.id);
    app.deviceId = dev.id;
    app.name = r.name;
    app.state = r.state;
    app.profile = await api.readProfile(mode());
    app.original = structuredClone($state.snapshot(app.profile)) as Profile;
    app.status = {};
    app.phase = "connected";
  } catch (e) {
    app.error = String(e);
    app.phase = "idle";
  }
}

export async function reconnect() {
  const d = app.found.find((f) => f.id === app.deviceId) ?? { id: app.deviceId, name: app.name, rssi: null };
  await connect(d);
}

export async function disconnect() {
  if (app.liveOn) await setLive(false);
  await api.disconnect();
  app.phase = "idle";
  app.profile = null;
  app.state = null;
}

export async function refresh() {
  app.state = await api.readState();
  app.profile = await api.readProfile(mode());
}

/** Copies the pages present in a readback into the current profile. */
function merge(rb: Profile | null) {
  if (!rb || !app.profile) return;
  const p = app.profile;
  if (rb.pagesOk[0] || rb.pagesOk[1] || rb.pagesOk[2]) {
    for (const [code, rep] of rb.keymap) {
      const e = p.keymap.find((k) => k[0] === code);
      if (e) e[1] = rep;
      else p.keymap.push([code, rep]);
    }
  }
  if (rb.pagesOk[2]) p.params = rb.params;
  for (let i = 0; i < 3; i++) if (rb.pagesOk[3 + i]) p.curves[i] = rb.curves[i];
}

// ---- Parameters: grouped by the command that carries them, coalesced while in flight.

type Group = "angle" | "ffb" | "forces";
const GROUP_OF: Record<ParamKey, Group> = {
  angle: "angle", ffb: "ffb", reverse: "ffb", spring: "forces", damping: "forces", friction: "forces", inertia: "forces",
};
const KEYS_OF: Record<Group, ParamKey[]> = {
  angle: ["angle"], ffb: ["ffb", "reverse"], forces: ["spring", "damping", "friction", "inertia"],
};

/** What the user asked for; the UI shows this while the wheel catches up. */
export const desired = $state({} as Partial<Params>);
const inflight: Partial<Record<Group, boolean>> = {};
const timers: Partial<Record<Group, ReturnType<typeof setTimeout>>> = {};

export function paramValue<K extends ParamKey>(k: K): Params[K] | undefined {
  return (desired[k] as Params[K] | undefined) ?? app.profile?.params[k];
}

export function setParam<K extends ParamKey>(k: K, v: Params[K], delay = 180) {
  (desired as Record<string, unknown>)[k] = v;
  const g = GROUP_OF[k];
  for (const key of KEYS_OF[g]) app.status[key] = "sending";
  clearTimeout(timers[g]);
  timers[g] = setTimeout(() => flush(g), delay);
}

export function setParams(p: Partial<Params>) {
  for (const [k, v] of Object.entries(p)) setParam(k as ParamKey, v as never, 50);
}

async function flush(g: Group) {
  if (inflight[g] || !app.profile) return;
  inflight[g] = true;
  const cur = app.profile.params;
  const want = { ...cur, ...desired } as Params;
  const m = mode();
  try {
    const r =
      g === "angle" ? await api.setAngle(m, want.angle)
      : g === "ffb" ? await api.setFfb(m, want.ffb, want.reverse)
      : await api.setForces(m, want);
    merge(r.readback);
    for (const k of KEYS_OF[g]) {
      const ok = r.readback?.pagesOk[2] ? r.readback.params[k] === want[k] : false;
      app.status[k] = ok ? "verified" : r.acked ? "acked" : "failed";
      // Only forget the request if a newer one didn't arrive meanwhile.
      if (desired[k] === want[k]) delete desired[k];
    }
  } catch (e) {
    for (const k of KEYS_OF[g]) app.status[k] = "failed";
    app.error = String(e);
  } finally {
    inflight[g] = false;
  }
  // The user kept dragging while we were busy: send the latest value.
  if (KEYS_OF[g].some((k) => k in desired)) flush(g);
}

export function revertParams() {
  if (app.original) setParams(app.original.params);
}

// ---- Key mapping

export async function remap(code: number, replacement: number) {
  const key = `key:${code}`;
  app.status[key] = "sending";
  try {
    const r = await api.setKeymap(mode(), code, replacement);
    merge(r.readback);
    const now = app.profile?.keymap.find((k) => k[0] === code)?.[1];
    app.status[key] = now === replacement ? "verified" : r.acked ? "acked" : "failed";
  } catch (e) {
    app.status[key] = "failed";
    app.error = String(e);
  }
}

export function mappingOf(code: number): number {
  return app.profile?.keymap.find((k) => k[0] === code)?.[1] ?? code;
}

// ---- Pedal curves

export async function writeCurve(pedal: number, curve: Curve) {
  const key = `curve:${pedal}`;
  app.status[key] = "sending";
  try {
    const r = await api.writeCurve(mode(), pedal, curve);
    merge(r.readback);
    const now = app.profile?.curves[pedal];
    const same = now && now.every((pt, i) => pt[0] === curve[i][0] && pt[1] === curve[i][1]);
    app.status[key] = same ? "verified" : r.acked ? "acked" : "failed";
  } catch (e) {
    app.status[key] = "failed";
    app.error = String(e);
  }
}

// ---- Live test mode

export async function setLive(onOff: boolean) {
  if (onOff) {
    app.liveOn = await api.testMode(true);
  } else {
    await api.testMode(false).catch(() => {});
    app.liveOn = false;
    app.live = null;
  }
}

// ---- Device

export async function switchMode(m: number) {
  app.status["mode"] = "sending";
  const r = await api.setMode(m);
  app.status["mode"] = r.acked ? "acked" : "failed";
  // The wheel re-enumerates; give it a moment, then re-read everything for the new mode.
  await new Promise((res) => setTimeout(res, 1500));
  await refresh().catch(() => {});
  if (app.state?.mode === m) app.status["mode"] = "verified";
}

export async function factoryReset() {
  app.status["reset"] = "sending";
  const r = await api.resetProfile(mode());
  await refresh().catch(() => {});
  app.status["reset"] = r.acked ? "verified" : "failed";
}
