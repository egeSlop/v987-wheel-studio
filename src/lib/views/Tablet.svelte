<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { i18n } from "../i18n.svelte";
  import { td } from "../i18n-dash";
  import { api, type DashConfig, type DashState } from "../api";

  const DOCS = "https://github.com/egeSlop/v987-wheel-studio/blob/main/docs/DASH.md";
  const GAME_NAMES: Record<string, string> = {
    fs22: "Farming Simulator 22", ets2: "Euro Truck Simulator 2", ats: "American Truck Simulator", ac: "Assetto Corsa", dr2: "DiRT Rally 2.0",
  };
  const SOURCES = ["ffb", "udp", "dr2", "ac"] as const;

  let st = $state<DashState | null>(null);
  let err = $state("");
  let busy = $state(false);
  let copied = $state(false);
  let setupMsg = $state<Record<string, string>>({});
  let testing = $state("");
  let testMsg = $state("");
  let openProfile = $state<string | null>(null);

  async function refresh(setup = false) {
    try {
      const s = await api.dashState(setup);
      if (!setup && st) s.setup = st.setup;
      st = s;
    } catch (e) {
      err = String(e);
    }
  }

  onMount(() => {
    refresh(true);
    const i = setInterval(() => refresh(false), 1500);
    return () => clearInterval(i);
  });

  async function act(f: () => Promise<DashState>) {
    busy = true;
    err = "";
    try {
      const s = await f();
      s.setup = st?.setup ?? s.setup;
      st = s;
    } catch (e) {
      err = String(e);
    } finally {
      busy = false;
    }
  }

  function edit(f: (c: DashConfig) => void) {
    if (!st) return;
    const c = structuredClone($state.snapshot(st.config)) as DashConfig;
    f(c);
    act(() => api.dashSetConfig(c));
  }

  async function copy(s: string) {
    await navigator.clipboard?.writeText(s);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }

  async function setup(id: string) {
    setupMsg[id] = "…";
    try {
      await api.dashSetupGame(id);
      setupMsg[id] = td("done");
      await refresh(true);
    } catch (e) {
      setupMsg[id] = String(e);
    }
  }

  async function test(profile: string, button: string) {
    testing = profile + ":" + button;
    testMsg = td("testIn");
    await new Promise((r) => setTimeout(r, 3000));
    try {
      const k = await api.dashTest(profile, button);
      testMsg = `${td("pressed")}: ${k}`;
    } catch (e) {
      testMsg = String(e);
    }
    setTimeout(() => {
      if (testing === profile + ":" + button) testing = "";
    }, 4000);
  }

  const status = $derived(st?.status ?? null);
  const url = $derived(status?.urls[0] ?? "");
  const game = $derived(status?.game ?? null);
  const ago = (ms: number | null) => (ms == null ? "–" : ms < 1500 ? (i18n.lang === "tr" ? "şimdi" : "now") : `${Math.round(ms / 1000)} s`);
  const mins = (ms: number) => (ms < 60000 ? `${Math.round(ms / 1000)} s` : `${Math.round(ms / 60000)} min`);
  const lbl = (b: { label: string; labels: Record<string, string> }) => b.labels?.[i18n.lang] ?? b.label;
  const show = (v: unknown) => (typeof v === "number" ? (Number.isInteger(v) ? String(v) : v.toFixed(1)) : typeof v === "boolean" ? (v ? "✓" : "–") : String(v));
</script>

<section>
  <div class="intro"><p>{td("intro")}</p></div>

  <div class="hero" class:on={st?.running}>
    <div class="state">
      <span class="dot"></span>
      <div>
        <div class="big">{st?.running ? td("running") : td("stopped")}</div>
        {#if st?.error}<div class="sm bad">{st.error}</div>{/if}
      </div>
    </div>
    <div class="actions">
      {#if st?.running}
        <button class="ghost" disabled={busy} onclick={() => act(api.dashStop)}>{td("stop")}</button>
      {:else}
        <button class="primary" disabled={busy} onclick={() => act(api.dashStart)}>{td("start")}</button>
      {/if}
    </div>
  </div>
  {#if err}<p class="note err">{err}</p>{/if}

  {#if status}
    <div class="pairing">
      <div class="qr" aria-label="QR code">{@html status.qrSvg}</div>
      <div class="pinfo">
        <div class="label">{td("openOn")}</div>
        <div class="url">
          <span class="mono">{url}</span>
          <button class="ghost sm" onclick={() => copy(url)}>{copied ? td("copied") : td("copy")}</button>
        </div>
        {#if status.urls[1]}<div class="sm dim mono">{status.urls[1]}</div>{/if}
        <p class="sm dim">{td("scan")}</p>
        {#if st?.config.require_pin}
          <div class="pinrow">
            <div>
              <div class="label">{td("pin")}</div>
              <div class="pin mono">{st.config.pin.slice(0, 3)} {st.config.pin.slice(3)}</div>
            </div>
            <button class="ghost sm" disabled={busy} onclick={() => act(api.dashNewPin)}>{td("newPin")}</button>
          </div>
          <p class="sm dim">{td("pinHelp")}</p>
        {/if}
        <p class="sm faint">{td("firewall", { port: status.port })}</p>
      </div>
    </div>

    <div class="cards">
      <div class="card">
        <div class="label">{td("now")}</div>
        {#if game}
          <div class="v">{game.name}</div>
          <div class="sm dim">{game.active ? td("live") : td("paused")} · {td("via")} {game.source}</div>
          <div class="tel">
            {#each Object.entries(game.data).filter(([k]) => !["active", "name", "type", "game", "source"].includes(k)).slice(0, 12) as [k, v]}
              <span>{k}<b>{show(v)}</b></span>
            {/each}
          </div>
        {:else}
          <div class="dim">{td("noGame")}</div>
        {/if}
      </div>
      <div class="card">
        <div class="label">{td("tablets")}</div>
        {#each status.tablets as tb (tb.id)}
          <div class="row">
            <span><b>{tb.device}</b> <span class="dim sm">{tb.peer}</span></span>
            <span class="sm" class:ok={tb.paired} class:dim={!tb.paired}>{tb.paired ? td("paired") : td("notPaired")}</span>
          </div>
          <div class="sm faint">{mins(tb.connectedMs)} · {tb.presses} {td("presses")}</div>
        {:else}
          <div class="dim">{td("noTablets")}</div>
        {/each}
      </div>
      <div class="card">
        <div class="label">{td("sources")}</div>
        {#each SOURCES as s}
          {@const info = status.sources[s]}
          <div class="row sm">
            <span>{td(`src_${s}` as any)}</span>
            <span class:ok={info?.up} class:bad={!info?.up}>{info?.up ? (s === "ffb" ? td("connected") : td("listening")) : td("down")}</span>
          </div>
          {#if info?.lastDataMs != null}<div class="sm faint">{td("lastData")}: {ago(info.lastDataMs)} · {info.game}</div>{/if}
          {#if info?.error && !info.up}<div class="sm bad">{info.error}</div>{/if}
        {/each}
        <div class="row sm"><span>{td("src_feed")}</span><span class="ok">{td("listening")}</span></div>
        <div class="label sp">{td("keyboard")}</div>
        {#if status.keyboardError}
          <div class="sm bad">{status.keyboardError}</div>
        {:else}
          <div class="sm dim">{status.keyboardReady ? td("kbReady") : td("kbLazy")}</div>
        {/if}
      </div>
    </div>
  {/if}

  <h3>{td("games")}</h3>
  <div class="games">
    {#each st?.setup ?? [] as g (g.id)}
      <div class="game">
        <div>
          <div class="v">{GAME_NAMES[g.id] ?? g.id}</div>
          <div class="sm" class:ok={g.state === "ready"} class:warn={g.state === "missing"} class:dim={g.state === "absent" || g.state === "manual"}>
            {td(g.state) + (g.detail ? ` · ${g.detail}` : "") + (g.id !== "fs22" ? ` · ${td("untested")}` : "")}
          </div>
          {#if g.id === "fs22" && g.state !== "ready" && g.state !== "absent"}<div class="sm dim">{td("fs22Manual")}</div>{/if}
          {#if g.id === "dr2" && g.state === "manual"}<div class="sm dim">{td("dr2Missing")}</div>{/if}
          {#if setupMsg[g.id]}<div class="sm mono pre">{setupMsg[g.id]}</div>{/if}
        </div>
        {#if (g.id === "ets2" || g.id === "ats" || g.id === "dr2") && g.state !== "absent" && g.state !== "manual"}
          <button class={g.state === "ready" ? "ghost sm" : "primary sm"} onclick={() => setup(g.id)}>{g.state === "ready" ? td("reinstall") : td("setUp")}</button>
        {/if}
      </div>
    {/each}
  </div>

  <h3>{td("profiles")}</h3>
  <p class="hint dim">{td("profilesHelp")}</p>
  <div class="actions">
    <button class="ghost sm" onclick={() => api.dashOpenProfiles().catch((e) => (err = String(e)))}>{td("openFolder")}</button>
    <button class="ghost sm" onclick={() => act(api.dashReload)}>{td("reload")}</button>
    <button class="ghost sm" onclick={() => openUrl(DOCS)}>{td("docs")}</button>
  </div>
  {#each status?.profileErrors ?? [] as e}<p class="note err">{e}</p>{/each}
  <div class="plist">
    {#each status?.profiles ?? [] as p (p.id)}
      <div class="prof">
        <button class="phead" onclick={() => (openProfile = openProfile === p.id ? null : p.id)}>
          <span><b>{p.name}</b> <span class="dim sm">{p.id} · {p.origin === "user" ? td("custom") : td("builtIn")} · {p.buttons.length} {td("buttons")}</span></span>
          <span class="dim">{openProfile === p.id ? "−" : "+"}</span>
        </button>
        {#if openProfile === p.id}
          <p class="sm dim">{td("testHelp")}</p>
          <div class="btns">
            {#each p.buttons as b (b.id)}
              <button class="ghost sm" disabled={!st?.running} onclick={() => test(p.id, b.id)} title={b.action || b.key}>
                {lbl(b)} <span class="faint mono">{b.key}</span>
              </button>
            {/each}
          </div>
          {#if testing.startsWith(p.id + ":")}<p class="sm mono">{testMsg}</p>{/if}
        {/if}
      </div>
    {/each}
  </div>

  {#if st}
    <h3>{td("autostart")}</h3>
    <div class="grid">
      <label class="toggle">
        <span><b>{td("autostart")}</b><br /><span class="sm dim">{td("autostartHelp")}</span></span>
        <input type="checkbox" checked={st.config.autostart} onchange={(e) => edit((c) => (c.autostart = e.currentTarget.checked))} />
      </label>
      <label class="toggle">
        <span><b>{td("requirePin")}</b><br /><span class="sm dim">{td("requirePinHelp")}</span></span>
        <input type="checkbox" checked={st.config.require_pin} onchange={(e) => edit((c) => (c.require_pin = e.currentTarget.checked))} />
      </label>
      <label class="toggle">
        <span><b>{td("port")}</b><br /><span class="sm dim">{td("portHelp")}</span></span>
        <input
          class="num"
          type="number"
          min="1024"
          max="65535"
          value={st.config.port}
          onchange={(e) => {
            const v = Number(e.currentTarget.value);
            if (v >= 1024 && v <= 65535) edit((c) => (c.port = v));
          }}
        />
      </label>
    </div>
    <p class="sm faint">{st.configPath}</p>
  {/if}
</section>

<style>
  .intro p { margin: 0 0 16px; color: var(--text-dim); max-width: 85ch; line-height: 1.5; }
  .dim { color: var(--text-dim); }
  .faint { color: var(--text-faint); }
  .sm { font-size: 12.5px; margin-top: 3px; }
  .mono { font-family: ui-monospace, monospace; }
  .ok { color: var(--ok); }
  .bad { color: var(--bad); }
  .warn { color: var(--warn, #e0a800); }
  .pre { white-space: pre-wrap; overflow-wrap: anywhere; }
  .hero { display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap; padding: 18px 20px; border-radius: var(--r-lg); background: var(--surface-2); border: 1px solid var(--line); }
  .hero.on { border-color: color-mix(in srgb, var(--ok) 40%, var(--line)); }
  .state { display: flex; gap: 14px; align-items: center; }
  .dot { width: 12px; height: 12px; border-radius: 50%; background: var(--text-faint); }
  .hero.on .dot { background: var(--ok); box-shadow: 0 0 0 4px var(--ok-soft); }
  .big { font-size: 18px; font-weight: 650; }
  .actions { display: flex; gap: 8px; flex-wrap: wrap; }
  .note { margin: 10px 2px 0; color: var(--text-dim); line-height: 1.5; }
  .note.err { color: var(--bad); }
  .pairing { display: flex; gap: 20px; align-items: flex-start; margin-top: 12px; padding: 18px; border-radius: var(--r-md); background: var(--surface-2); border: 1px solid var(--line); flex-wrap: wrap; }
  .qr { width: 188px; height: 188px; border-radius: 12px; overflow: hidden; background: #fff; flex: none; }
  .qr :global(svg) { width: 100%; height: 100%; display: block; }
  .pinfo { flex: 1; min-width: 260px; display: grid; gap: 6px; align-content: start; }
  .url { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
  .url .mono { font-size: 20px; font-weight: 650; user-select: all; }
  .pinrow { display: flex; align-items: flex-end; gap: 16px; margin-top: 8px; }
  .pin { font-size: 28px; font-weight: 700; letter-spacing: 0.12em; }
  .cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 12px; margin-top: 12px; }
  .card { padding: 14px 16px; border-radius: var(--r-md); background: var(--surface-2); border: 1px solid var(--line); min-width: 0; display: grid; gap: 4px; align-content: start; }
  .label { font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-faint); margin-bottom: 4px; }
  .label.sp { margin-top: 10px; }
  .v { font-weight: 600; overflow-wrap: anywhere; }
  .row { display: flex; justify-content: space-between; gap: 10px; align-items: baseline; }
  .tel { display: grid; grid-template-columns: 1fr 1fr; gap: 4px 14px; font-size: 12.5px; color: var(--text-dim); margin-top: 8px; }
  .tel span { display: flex; justify-content: space-between; gap: 8px; min-width: 0; }
  .tel b { color: var(--text); font-variant-numeric: tabular-nums; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  h3 { font-size: 13px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-faint); margin: 26px 0 8px; font-weight: 600; }
  .hint { margin: 0 0 10px; font-size: 13px; max-width: 85ch; line-height: 1.5; }
  .games { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 10px; }
  .game { display: flex; justify-content: space-between; align-items: center; gap: 12px; padding: 14px 16px; border-radius: var(--r-md); background: var(--surface-2); border: 1px solid var(--line); }
  .plist { display: grid; gap: 8px; margin-top: 10px; }
  .prof { border-radius: var(--r-md); background: var(--surface-2); border: 1px solid var(--line); padding: 4px 14px; }
  .phead { all: unset; cursor: pointer; display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 10px 0; }
  .btns { display: flex; flex-wrap: wrap; gap: 6px; padding-bottom: 12px; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 10px; }
  .toggle { display: flex; justify-content: space-between; align-items: center; gap: 12px; padding: 14px 16px; border-radius: var(--r-md); background: var(--surface-2); border: 1px solid var(--line); cursor: pointer; }
  .num { width: 96px; }
</style>
