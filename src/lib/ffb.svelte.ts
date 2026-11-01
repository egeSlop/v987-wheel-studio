import { api, type FfbConfig, type FfbForces, type FfbState, type FfbStatus } from "./api";

/** Live view of the FFB server. Data comes from its public /stream endpoint, the same one
 *  community tools use, so it works for this app's server and for the headless one alike. */
export const ffb = $state({
  state: null as FfbState | null,
  /** Latest status line from /stream. */
  status: null as FfbStatus | null,
  /** Latest live frame (only while a game is driving). */
  frame: null as { mode: string; game: string | null; telemetry: Record<string, unknown> | null; forces: FfbForces } | null,
  frameAt: 0,
  streamOk: false,
  busy: false,
  error: "",
  /** Rolling history of the push force, for the sparkline. */
  history: [] as number[],
});

let ws: WebSocket | null = null;
let wsUrl = "";
let poll: ReturnType<typeof setInterval> | null = null;
let users = 0;

function openStream(listen: string) {
  const url = `ws://${listen}/stream`;
  if (ws && wsUrl === url && ws.readyState <= 1) return;
  ws?.close();
  wsUrl = url;
  try {
    ws = new WebSocket(url);
  } catch {
    ws = null;
    return;
  }
  ws.onopen = () => (ffb.streamOk = true);
  ws.onclose = () => {
    ffb.streamOk = false;
    ws = null;
  };
  ws.onmessage = (e) => {
    try {
      const m = JSON.parse(e.data);
      if (m.type === "status") ffb.status = m;
      else if (m.type === "frame") {
        ffb.frame = m;
        ffb.frameAt = Date.now();
        ffb.history.push(m.forces.constant);
        if (ffb.history.length > 150) ffb.history.splice(0, ffb.history.length - 150);
      }
    } catch {}
  };
}

let wasExternal = false;

export async function refresh() {
  try {
    let s = await api.ffbState();
    // The launcher's headless server went away (game closed): take over again.
    if (wasExternal && !s.running && !s.external && s.config.autostart) {
      s = await api.ffbStart().catch(() => s);
    }
    wasExternal = s.external;
    ffb.state = s;
    if (s.running || s.external) openStream(s.config.listen);
    else {
      ws?.close();
      ffb.status = null;
    }
  } catch (e) {
    ffb.error = String(e);
  }
}

/** Starts watching while at least one view needs it. Returns the stop function. */
export function watch(): () => void {
  users++;
  if (!poll) {
    refresh();
    poll = setInterval(refresh, 1500);
  }
  return () => {
    users--;
    if (users <= 0 && poll) {
      clearInterval(poll);
      poll = null;
      ws?.close();
    }
  };
}

async function act(f: () => Promise<FfbState>) {
  ffb.busy = true;
  ffb.error = "";
  try {
    ffb.state = await f();
    await refresh();
  } catch (e) {
    ffb.error = String(e);
    await refresh();
  } finally {
    ffb.busy = false;
  }
}

export const start = () => act(api.ffbStart);
export const stop = () => act(api.ffbStop);

let saveTimer: ReturnType<typeof setTimeout> | null = null;
export const saved = $state({ at: 0, pending: false });

/** Debounced save; the running server applies it within a few milliseconds. */
export function setConfig(c: FfbConfig, delay = 200) {
  if (ffb.state) ffb.state.config = c;
  saved.pending = true;
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(async () => {
    try {
      ffb.state = await api.ffbSetConfig($state.snapshot(c) as FfbConfig);
      saved.at = Date.now();
    } catch (e) {
      ffb.error = String(e);
    } finally {
      saved.pending = false;
    }
  }, delay);
}

export async function runTest() {
  ffb.error = "";
  try {
    await api.ffbTest();
  } catch (e) {
    ffb.error = String(e);
  }
}

export const live = () => !!ffb.frame && Date.now() - ffb.frameAt < 1000;
