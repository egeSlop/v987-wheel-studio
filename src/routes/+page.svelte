<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { i18n, setLang, t } from "$lib/i18n.svelte";
  import { app, scan, disconnect, reconnect, mode } from "$lib/store.svelte";
  import { MODES } from "$lib/keys";
  import DeviceArt from "$lib/components/DeviceArt.svelte";
  import Tuning from "$lib/views/Tuning.svelte";
  import Live from "$lib/views/Live.svelte";
  import Buttons from "$lib/views/Buttons.svelte";
  import Pedals from "$lib/views/Pedals.svelte";
  import Presets from "$lib/views/Presets.svelte";
  import Device from "$lib/views/Device.svelte";
  import Labs from "$lib/views/Labs.svelte";
  import FfbServer from "$lib/views/FfbServer.svelte";
  import Tablet from "$lib/views/Tablet.svelte";
  import Connect from "$lib/components/Connect.svelte";
  import { ffb, watch } from "$lib/ffb.svelte";

  const TABS = [
    { id: "tuning", k: "tabTuning", icon: "M4 12h4m4 0h8M8 12a2 2 0 1 0 4 0 2 2 0 1 0-4 0M4 6h10m4 0h2m-6 0a2 2 0 1 0 4 0 2 2 0 1 0-4 0M4 18h2m4 0h10M6 18a2 2 0 1 0 4 0 2 2 0 1 0-4 0", C: Tuning },
    { id: "live", k: "tabLive", icon: "M12 3a9 9 0 1 0 0 18 9 9 0 1 0 0-18m0 6a3 3 0 1 0 0 6 3 3 0 1 0 0-6M3 12h6m6 0h6m-9 3v6", C: Live },
    { id: "buttons", k: "tabButtons", icon: "M5 7a2 2 0 1 0 4 0 2 2 0 1 0-4 0m10 0a2 2 0 1 0 4 0 2 2 0 1 0-4 0M5 17a2 2 0 1 0 4 0 2 2 0 1 0-4 0m10 0a2 2 0 1 0 4 0 2 2 0 1 0-4 0", C: Buttons },
    { id: "pedals", k: "tabPedals", icon: "M4 20L20 4M4 20c8 0 10-6 16-16", C: Pedals },
    { id: "ffb", k: "tabFfb", icon: "M13 2L4 14h7l-1 8 9-12h-7z", C: FfbServer, free: true },
    { id: "tablet", k: "tabTablet", icon: "M6 3h12a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zM11 18h2", C: Tablet, free: true },
    { id: "presets", k: "tabPresets", icon: "M5 4h14v16l-7-4-7 4z", C: Presets, free: true },
    { id: "device", k: "tabDevice", icon: "M9 3h6v4H9zM5 7h14v14H5zM9 12h6m-6 4h4", C: Device },
    { id: "labs", k: "tabLabs", icon: "M9 3h6M10 3v6L5 19a1 1 0 0 0 1 2h12a1 1 0 0 0 1-2l-5-10V3", C: Labs },
  ] as const;

  let tab = $state<(typeof TABS)[number]["id"]>("tuning");
  const cur = $derived(TABS.find((x) => x.id === tab)!);
  const View = $derived(cur.C);
  const linked = $derived(app.phase === "connected" || app.phase === "lost");
  const needsWheel = $derived(!("free" in cur && cur.free) && !linked);
  const ffbOn = $derived(!!ffb.state?.running || !!ffb.state?.external);
  const ffbDriving = $derived(ffb.status?.mode === "telemetry" || ffb.status?.mode === "forces");

  onMount(() => { scan(); return watch(); });
</script>

<div class="shell" in:fade={{ duration: 200 }}>
  <aside class="side">
    <div class="brand"><span class="logo"></span>{t("appName")}</div>

    <div class="device">
      <div class="mini"><DeviceArt kind="wheel" /></div>
      {#if linked}
        <div class="dname">{app.name.replace(/^FFB WHEEL\s*/i, "") || app.name}</div>
        <div class="dmeta">
          <span class="pill" class:lost={app.phase === "lost"}><i></i>{app.phase === "lost" ? t("disconnected") : "BLE"}</span>
          <span class="pill">{MODES[mode()] ?? mode()}</span>
          <span class="pill">fw {app.state?.firmware ?? "–"}</span>
        </div>
      {:else}
        <div class="dname dim">{t("notConnected")}</div>
        <button class="primary sm" onclick={() => (tab = "tuning")}>{t("connectWheel")}</button>
      {/if}
    </div>

    <nav>
      {#each TABS as x}
        <button class:on={tab === x.id} class:off={!("free" in x && x.free) && !linked} onclick={() => (tab = x.id)}>
          <svg viewBox="0 0 24 24"><path d={x.icon} /></svg>{t(x.k)}
          {#if x.id === "ffb"}<span class="led" class:on={ffbOn} class:drive={ffbDriving}></span>{/if}
        </button>
      {/each}
    </nav>

    <div class="foot">
      <div class="lang">
        <button class:on={i18n.lang === "tr"} onclick={() => setLang("tr")}>TR</button>
        <button class:on={i18n.lang === "en"} onclick={() => setLang("en")}>EN</button>
      </div>
      {#if linked}<button class="ghost sm" onclick={disconnect}>{t("disconnect")}</button>{/if}
    </div>
  </aside>

  <main>
    {#if app.phase === "lost"}
      <div class="banner">
        {t("disconnected")}
        <button class="primary sm" onclick={reconnect}>{t("reconnect")}</button>
      </div>
    {/if}
    {#if app.error && linked}
      <div class="banner err" role="alert">{app.error}<button class="ghost sm" onclick={() => (app.error = "")}>×</button></div>
    {/if}
    <h1>{t(cur.k)}</h1>
    {#key tab + String(needsWheel)}
      <div in:fade={{ duration: 140 }}>
        {#if needsWheel}<Connect />{:else}<View />{/if}
      </div>
    {/key}
  </main>
</div>

<style>
  .shell { display: grid; grid-template-columns: 248px 1fr; height: 100vh; }
  .side { background: var(--surface-1); border-right: 1px solid var(--line); display: flex; flex-direction: column; padding: 18px 14px; gap: 18px; }
  .brand { display: flex; align-items: center; gap: 10px; font-weight: 700; letter-spacing: 0.01em; padding: 0 6px; }
  .logo { width: 22px; height: 22px; border-radius: 50%; border: 4px solid var(--accent); box-shadow: inset 0 0 0 3px var(--surface-1), inset 0 0 0 6px var(--text-dim); }
  .device { background: var(--surface-2); border: 1px solid var(--line); border-radius: var(--r-lg); padding: 12px; display: grid; gap: 8px; justify-items: center; }
  .mini { width: 120px; }
  .dname { font-weight: 600; text-align: center; }
  .dmeta { display: flex; flex-wrap: wrap; gap: 6px; justify-content: center; }
  .pill { font-size: 11px; padding: 3px 8px; border-radius: 999px; background: var(--surface-3); color: var(--text-dim); display: inline-flex; align-items: center; gap: 5px; }
  .pill i { width: 6px; height: 6px; border-radius: 50%; background: var(--ok); }
  .pill.lost i { background: var(--bad); }
  nav { display: grid; gap: 2px; }
  nav button { all: unset; cursor: pointer; display: flex; align-items: center; gap: 12px; padding: 10px 12px; border-radius: var(--r-sm); color: var(--text-dim); font-weight: 500; }
  nav button:hover { background: var(--surface-2); color: var(--text); }
  nav button.on { background: var(--accent-soft); color: var(--text); }
  nav button.on svg { stroke: var(--accent); }
  nav svg { width: 19px; height: 19px; fill: none; stroke: currentColor; stroke-width: 1.8; stroke-linecap: round; stroke-linejoin: round; }
  .foot { margin-top: auto; display: flex; justify-content: space-between; align-items: center; gap: 8px; }
  .lang { display: inline-grid; grid-auto-flow: column; background: var(--surface-3); border-radius: var(--r-sm); padding: 3px; gap: 2px; }
  .lang button { border: 0; background: transparent; color: var(--text-dim); padding: 5px 10px; border-radius: 6px; cursor: pointer; font-size: 12px; font-weight: 600; }
  .lang button.on { background: var(--surface-1); color: var(--text); box-shadow: var(--shadow-sm); }
  main { overflow: auto; padding: 26px 32px 40px; }
  h1 { font-size: 24px; margin: 0 0 14px; letter-spacing: -0.01em; }
  .banner { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 10px 14px; border-radius: var(--r-md); background: var(--warn-soft); margin-bottom: 14px; font-weight: 500; }
  .banner.err { background: var(--bad-soft); color: var(--text); font-weight: 400; }

  .dim { color: var(--text-dim); }
  nav button.off { opacity: 0.55; }
  .led { margin-left: auto; width: 8px; height: 8px; border-radius: 50%; background: var(--surface-4); }
  .led.on { background: var(--ok); }
  .led.drive { box-shadow: 0 0 0 3px var(--ok-soft); }
</style>
