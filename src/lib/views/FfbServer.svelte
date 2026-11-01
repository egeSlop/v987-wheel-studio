<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
  import { t } from "../i18n.svelte";
  import { ffbHelp } from "../i18n-ffb";
  import { ffb, watch, start, stop, setConfig, saved, runTest, live } from "../ffb.svelte";
  import FeelRow from "../components/FeelRow.svelte";
  import type { Feel, FfbConfig } from "../api";

  const DOCS = "https://github.com/egeSlop/ffb-server/blob/main/docs/FFB-SERVER.md";
  const DEFAULT_FEEL: Feel = { centering: 0.55, centering_parked: 0.08, damping: 0.3, damping_engine_off: 0.65, bumps: 0.6, impacts: 0.8, texture: 0.25, engine: 0.1, soft_lock: 0.8 };
  const FEEL_KEYS = Object.keys(DEFAULT_FEEL) as (keyof Feel)[];

  onMount(() => watch());

  // Re-render "live" once in a while so a stopped stream is noticed.
  let tick = $state(0);
  onMount(() => {
    const i = setInterval(() => tick++, 500);
    return () => clearInterval(i);
  });

  const st = $derived(ffb.state);
  const status = $derived(ffb.status ?? st?.status ?? null);
  const cfg = $derived(st?.config);
  const isLive = $derived(tick >= 0 && live());
  const mode = $derived(status?.mode ?? "idle");
  const forces = $derived(isLive && ffb.frame ? ffb.frame.forces : status?.forces);
  const tel = $derived(isLive ? (ffb.frame?.telemetry as Record<string, any> | null) : null);
  const serving = $derived(!!st?.running || !!st?.external);
  const game = $derived(status?.clients.find((c) => c.game)?.game ?? status?.clients[0]?.source ?? "");

  function edit(f: (c: FfbConfig) => void) {
    if (!cfg) return;
    const c = structuredClone($state.snapshot(cfg)) as FfbConfig;
    f(c);
    setConfig(c);
  }
  const copy = (s: string) => navigator.clipboard?.writeText(s);

  const modeLabel = $derived(
    mode === "telemetry" ? t("ffbModeTelemetry") : mode === "forces" ? t("ffbModeForces") : mode === "test" ? t("ffbModeTest") : t("ffbModeIdle"),
  );

  const BARS = [
    ["spring", "fSpring", false],
    ["damper", "fDamper", false],
    ["constant", "fConstant", true],
    ["texture", "fTexture", false],
    ["engine", "fEngine", false],
  ] as const;

  const spark = $derived.by(() => {
    const h = ffb.history;
    if (h.length < 2) return "";
    return h.map((v, i) => `${(i / 149) * 300},${30 - v * 28}`).join(" ");
  });
</script>

<section>
  <header class="intro"><p>{t("ffbIntro")}</p></header>

  <!-- Server state -->
  <div class="hero" class:on={serving} class:drive={mode === "telemetry" || mode === "forces"}>
    <div class="state">
      <span class="dot"></span>
      <div>
        <div class="big">{st?.running ? t("ffbRunning") : st?.external ? t("ffbExternal") : t("ffbStopped")}</div>
        <div class="dim mono">{serving ? `ws://${cfg?.listen}` : st?.error ?? ""}</div>
      </div>
    </div>
    <div class="actions">
      {#if st?.running}
        <button class="ghost" disabled={ffb.busy || mode === "test" || !status?.wheel} onclick={runTest} title={t("ffbTestHint")}>
          {mode === "test" ? `${t("ffbModeTest")}: ${status?.testStep ?? ""}` : t("ffbTest")}
        </button>
        <button class="ghost" disabled={ffb.busy} onclick={stop}>{t("ffbStop")}</button>
      {:else if !st?.external}
        <button class="primary" disabled={ffb.busy} onclick={start}>{t("ffbStart")}</button>
      {/if}
    </div>
  </div>
  {#if st?.external}<p class="note">{t("ffbExternalHint")}</p>{/if}
  {#if ffb.error}<p class="note err">{ffb.error}</p>{/if}

  {#if serving}
    <div class="cards">
      <div class="card">
        <div class="label">{t("ffbWheel")}</div>
        {#if status?.wheel}
          <div class="v ok">{status.wheel.name}</div>
          <div class="dim mono sm">{status.wheel.path}</div>
        {:else}
          <div class="v bad">{t("ffbNoWheel")}</div>
          <div class="dim sm">{status?.wheelError ?? ""}</div>
        {/if}
      </div>
      <div class="card">
        <div class="label">{t("ffbGame")}</div>
        {#if status?.clients.length}
          <div class="v ok">{game || "?"}</div>
          {#each status.clients as c (c.id)}
            <div class="dim sm">{c.source || c.peer} · {c.frames} {t("frames")}{c.errors ? ` · ${c.errors} ${t("errors")}` : ""}</div>
            {#if c.lastError}<div class="bad sm mono">{c.lastError}</div>{/if}
          {/each}
        {:else}
          <div class="v dim">{t("ffbNoGame")}</div>
          <div class="dim sm">{t("ffbNoGameHint")}</div>
        {/if}
      </div>
      <div class="card">
        <div class="label">{modeLabel}</div>
        {#if tel}
          <div class="tel">
            <span>{t("speed")}<b>{(((tel.speed as number) ?? 0) * 3.6).toFixed(0)} km/h</b></span>
            <span>{t("rpm")}<b>{tel.rpm != null ? Math.round(tel.rpm as number) : "–"}</b></span>
            <span>{t("surface")}<b>{(tel.wheels as any[])?.find((w) => w.steered)?.surface ?? (tel.wheels as any[])?.[0]?.surface ?? "–"}</b></span>
            <span>{t("engineOn")}<b>{tel.engine ? t("on") : t("off")}</b></span>
          </div>
        {:else}
          <div class="dim sm">{mode === "idle" ? t("ffbIdleHint") : ""}</div>
        {/if}
      </div>
    </div>

    <!-- Live forces -->
    <div class="forces">
      {#each BARS as [k, label, signed]}
        {@const v = forces ? (forces[k] as number) : 0}
        <div class="bar">
          <div class="bl"><span>{t(label)}</span><b>{v.toFixed(2)}</b></div>
          <div class="track" class:signed>
            {#if signed}
              <div class="fill" style:left="{50 + Math.min(0, v) * 50}%" style:width="{Math.abs(v) * 50}%"></div>
              <div class="mid"></div>
            {:else}
              <div class="fill" style:width="{Math.min(1, v) * 100}%"></div>
            {/if}
          </div>
          {#if k === "texture" || k === "engine"}<div class="hz">{forces ? forces[k === "texture" ? "textureHz" : "engineHz"] : 0} Hz</div>{/if}
        </div>
      {/each}
      <svg class="spark" viewBox="0 0 300 60" preserveAspectRatio="none" aria-hidden="true">
        <line x1="0" y1="30" x2="300" y2="30" />
        {#if spark}<polyline points={spark} />{/if}
      </svg>
    </div>
  {/if}

  {#if cfg}
    <h3>{t("ffbFeel")} <span class="saved">{saved.pending ? t("saving") : saved.at ? "✓ " + t("savedLive") : ""}</span></h3>
    <p class="dim hint">{t("ffbFeelHint")}</p>
    <div class="grid">
      <FeelRow k="master_gain" value={cfg.master_gain} def={0.8} onchange={(v) => edit((c) => (c.master_gain = v))} />
      {#each FEEL_KEYS as k}
        <FeelRow {k} value={cfg.feel[k]} def={DEFAULT_FEEL[k]} onchange={(v) => edit((c) => (c.feel[k] = v))} />
      {/each}
    </div>

    <h3>{t("ffbGeneral")}</h3>
    <div class="grid">
      <label class="toggle">
        <div><b>{ffbHelp("invert").title}</b><div class="dim sm">{ffbHelp("invert").short}</div></div>
        <span class="switch"><input type="checkbox" checked={cfg.invert} onchange={(e) => edit((c) => (c.invert = e.currentTarget.checked))} /><span></span></span>
      </label>
      <label class="toggle">
        <div><b>{t("autostart")}</b><div class="dim sm">{t("autostartHint")}</div></div>
        <span class="switch"><input type="checkbox" checked={cfg.autostart} onchange={(e) => edit((c) => (c.autostart = e.currentTarget.checked))} /><span></span></span>
      </label>
    </div>

    <h3>{t("ffbForModders")}</h3>
    <div class="mod">
      <p class="dim">{t("ffbModdersHint")}</p>
      <div class="ep"><span>{t("endpoint")}</span><code>ws://{cfg.listen}/telemetry</code><button class="ghost sm" onclick={() => copy(`ws://${cfg.listen}/telemetry`)}>{t("copy")}</button></div>
      <div class="ep"><span>{t("watchEndpoint")}</span><code>ws://{cfg.listen}/stream</code><button class="ghost sm" onclick={() => copy(`ws://${cfg.listen}/stream`)}>{t("copy")}</button></div>
      <pre>{`{"type":"hello","game":"My Game","source":"MyMod 1.0","protocol":1}
{"type":"telemetry","speed":12.5,"engine":true,"rpm":1400,
 "wheels":[{"i":1,"steered":true,"x":1,"y":0.02,"surface":"gravel"}, ...]}
{"type":"forces","spring":0.4,"damper":0.2,"constant":-0.1,"texture":0.1,"textureHz":30}`}</pre>
      <div class="links">
        <button class="ghost sm" onclick={() => openUrl(DOCS)}>{t("docs")} ↗</button>
        <button class="ghost sm" onclick={() => st && revealItemInDir(st.configPath)}>{t("settingsFile")}</button>
      </div>
    </div>
  {/if}
</section>

<style>
  .intro p { margin: 0 0 16px; color: var(--text-dim); max-width: 85ch; line-height: 1.5; }
  .dim { color: var(--text-dim); }
  .sm { font-size: 12.5px; margin-top: 3px; }
  .mono { font-family: ui-monospace, monospace; }
  .ok { color: var(--ok); }
  .bad { color: var(--bad); }
  .hero { display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap; padding: 18px 20px; border-radius: var(--r-lg); background: var(--surface-2); border: 1px solid var(--line); }
  .hero.on { border-color: color-mix(in srgb, var(--ok) 40%, var(--line)); }
  .state { display: flex; gap: 14px; align-items: center; }
  .dot { width: 12px; height: 12px; border-radius: 50%; background: var(--text-faint); }
  .hero.on .dot { background: var(--ok); box-shadow: 0 0 0 4px var(--ok-soft); }
  .hero.drive .dot { animation: pulse 1.2s ease-in-out infinite; }
  @keyframes pulse { 50% { box-shadow: 0 0 0 8px transparent; } }
  .big { font-size: 18px; font-weight: 650; }
  .actions { display: flex; gap: 8px; flex-wrap: wrap; }
  .note { margin: 10px 2px 0; color: var(--text-dim); line-height: 1.5; max-width: 85ch; }
  .note.err { color: var(--bad); }
  .cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 12px; margin-top: 12px; }
  .card { padding: 14px 16px; border-radius: var(--r-md); background: var(--surface-2); border: 1px solid var(--line); min-width: 0; }
  .label { font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-faint); margin-bottom: 6px; }
  .v { font-weight: 600; overflow-wrap: anywhere; }
  .tel { display: grid; grid-template-columns: 1fr 1fr; gap: 6px 14px; font-size: 12.5px; color: var(--text-dim); }
  .tel span { display: flex; justify-content: space-between; gap: 8px; }
  .tel b { color: var(--text); font-variant-numeric: tabular-nums; }
  .forces { margin-top: 12px; padding: 16px 18px; border-radius: var(--r-md); background: var(--surface-2); border: 1px solid var(--line); display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 14px 18px; }
  .bl { display: flex; justify-content: space-between; font-size: 12.5px; color: var(--text-dim); margin-bottom: 6px; }
  .bl b { color: var(--text); font-variant-numeric: tabular-nums; }
  .track { position: relative; height: 8px; border-radius: 4px; background: var(--surface-4); overflow: hidden; }
  .fill { position: absolute; top: 0; bottom: 0; left: 0; background: var(--accent); border-radius: 4px; transition: width 60ms linear, left 60ms linear; }
  .mid { position: absolute; left: 50%; top: -2px; bottom: -2px; width: 2px; margin-left: -1px; background: var(--text-faint); }
  .hz { font-size: 11px; color: var(--text-faint); margin-top: 4px; font-variant-numeric: tabular-nums; }
  .spark { grid-column: 1 / -1; width: 100%; height: 48px; }
  .spark line { stroke: var(--line-strong); stroke-width: 1; }
  .spark polyline { fill: none; stroke: var(--accent); stroke-width: 1.6; vector-effect: non-scaling-stroke; }
  h3 { font-size: 13px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-faint); margin: 26px 0 6px; font-weight: 600; display: flex; gap: 12px; align-items: baseline; }
  .saved { text-transform: none; letter-spacing: 0; color: var(--ok); font-weight: 500; }
  .hint { margin: 0 0 12px; font-size: 13px; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 10px; }
  .toggle { display: flex; justify-content: space-between; align-items: center; gap: 12px; padding: 14px 16px; border-radius: var(--r-md); background: var(--surface-2); border: 1px solid var(--line); cursor: pointer; }
  .mod { padding: 16px 18px; border-radius: var(--r-md); background: var(--surface-2); border: 1px solid var(--line); margin-top: 8px; }
  .mod p { margin: 0 0 12px; line-height: 1.5; max-width: 85ch; }
  .ep { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; margin: 6px 0; }
  .ep span { min-width: 120px; color: var(--text-dim); font-size: 13px; }
  code, pre { font-family: ui-monospace, monospace; font-size: 12.5px; user-select: text; }
  code { background: var(--surface-3); padding: 4px 8px; border-radius: 6px; }
  pre { background: var(--surface-1); border: 1px solid var(--line); padding: 12px; border-radius: var(--r-sm); overflow-x: auto; margin: 12px 0; color: var(--text-dim); }
  .links { display: flex; gap: 8px; flex-wrap: wrap; }
</style>
