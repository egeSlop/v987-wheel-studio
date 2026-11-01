<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl, openPath } from "@tauri-apps/plugin-opener";
  import { t, i18n } from "../i18n.svelte";
  import { api, type Params, type Preset } from "../api";
  import { app, setParams, paramValue } from "../store.svelte";
  import { ffb, refresh as refreshFfb, setConfig } from "../ffb.svelte";

  let all = $state<Preset[]>([]);
  let errors = $state<string[]>([]);
  let dir = $state("");
  let platform = $state<"pc" | "ps4" | "xbox">("pc");
  let source = $state<"" | "vendor" | "studio" | "user">("");
  let q = $state("");
  let refreshing = $state(false);
  let refreshMsg = $state("");
  let newName = $state("");
  let justApplied = $state("");

  async function load() {
    const l = await api.listPresets();
    all = l.presets;
    errors = l.errors;
    dir = l.dir;
  }
  onMount(() => {
    load();
    refreshFfb();
  });

  async function refreshVendor() {
    refreshing = true;
    refreshMsg = "";
    try {
      const l = await api.refreshVendorPresets();
      all = l.presets;
      errors = l.errors;
      refreshMsg = "✓ " + t("refreshed");
    } catch (e) {
      refreshMsg = String(e);
    }
    refreshing = false;
  }

  const connected = $derived(app.phase === "connected");
  const lang = $derived(i18n.lang);
  const shown = $derived(
    all
      .filter((p) => (p.source === "user" || p.platform === platform) && (!source || p.source === source))
      .filter((p) => !q.trim() || (p.name + " " + Object.values(p.names ?? {}).join(" ")).toLowerCase().includes(q.trim().toLowerCase()))
      .sort((a, b) => rank(a) - rank(b) || a.name.localeCompare(b.name)),
  );
  const rank = (p: Preset) => (p.source === "studio" ? 0 : p.source === "user" ? 1 : 2);
  const note = (p: Preset) => p.notes?.[lang] ?? p.notes?.en ?? (lang === "tr" ? undefined : p.notes?.zh);

  function wheelValues(p: Preset): Partial<Params> {
    const w = p.wheel;
    const out: Partial<Params> = {};
    for (const k of ["angle", "ffb", "spring", "damping", "friction", "inertia", "reverse"] as const) {
      if (w[k] !== undefined) (out as Record<string, unknown>)[k] = w[k];
    }
    return out;
  }

  function apply(p: Preset) {
    const w = wheelValues(p);
    if (connected && Object.keys(w).length) setParams(w);
    const fp = p.ffb_server;
    if (fp && ffb.state) {
      const c = structuredClone($state.snapshot(ffb.state.config));
      if (fp.master_gain !== undefined) c.master_gain = fp.master_gain;
      if (fp.invert !== undefined) c.invert = fp.invert;
      if (fp.feel) c.feel = { ...c.feel, ...fp.feel };
      setConfig(c, 0);
    }
    justApplied = p.id;
    setTimeout(() => (justApplied === p.id ? (justApplied = "") : null), 2500);
  }

  async function saveCurrent() {
    if (!newName.trim()) return;
    const p: Preset = {
      format: 1, id: "", name: newName.trim(), platform: "pc", source: "user", origin: "user",
      wheel: app.profile ? { ...app.profile.params } : {},
      ffb_server: ffb.state ? { master_gain: ffb.state.config.master_gain, invert: ffb.state.config.invert, feel: { ...ffb.state.config.feel } } : undefined,
    };
    await api.saveUserPreset(p);
    newName = "";
    await load();
  }

  async function remove(p: Preset) {
    await api.deleteUserPreset(p.id);
    await load();
  }

  const FIELDS: [keyof Params, string][] = [["ffb", "FFB"], ["spring", "SPR"], ["damping", "DMP"], ["friction", "FRC"], ["inertia", "INR"]];
  const differs = (p: Preset, k: keyof Params) => connected && p.wheel[k] !== undefined && paramValue(k) !== p.wheel[k];
  const canApply = (p: Preset) => (connected && Object.keys(wheelValues(p)).length > 0) || (!!p.ffb_server && !!ffb.state);
</script>

<section>
  <header class="intro"><p>{t("presetsIntro2")}</p></header>

  <div class="bar">
    <div class="seg">
      {#each [["pc", "PC"], ["ps4", "PS4"], ["xbox", "Xbox"]] as [v, l]}
        <button class:on={platform === v} onclick={() => (platform = v as typeof platform)}>{l}</button>
      {/each}
    </div>
    <div class="seg">
      {#each [["", "all"], ["studio", "srcStudio"], ["vendor", "srcVendor"], ["user", "srcUser"]] as [v, l]}
        <button class:on={source === v} onclick={() => (source = v as typeof source)}>{t(l)}</button>
      {/each}
    </div>
    <input type="search" placeholder={t("search")} bind:value={q} />
    <div class="grow"></div>
    <button class="ghost sm" disabled={refreshing} onclick={refreshVendor}>{refreshing ? t("loading") : "⟳ " + t("refreshVendor")}</button>
    <button class="ghost sm" onclick={() => dir && openPath(dir).catch(() => {})}>{t("presetFolder")}</button>
  </div>
  {#if refreshMsg}<p class="msg">{refreshMsg}</p>{/if}
  {#each errors as e}<p class="msg err">{e}</p>{/each}
  {#if !connected}<p class="msg">{t("needsWheel")}</p>{/if}

  <div class="grid">
    {#each shown as p (p.id)}
      {@const n = note(p)}
      <article class="card" class:applied={justApplied === p.id} class:studio={p.source === "studio"}>
        <div class="top">
          <div class="titles">
            <div class="name">{p.name}</div>
            {#if p.names?.zh}<div class="faint">{p.names.zh}</div>{/if}
            <div class="badges">
              <span class="badge {p.source}">{p.source === "studio" ? t("srcStudio") : p.source === "user" ? t("srcUser") : t("srcVendor")}</span>
              {#if p.ffb_server}<span class="badge ffb">FFB</span>{/if}
            </div>
          </div>
          {#if p.wheel.angle !== undefined}<div class="angle" class:diff={differs(p, "angle")}>{p.wheel.angle}°</div>{/if}
        </div>

        {#if p.wheel.ffb !== undefined}
          <div class="meters">
            {#each FIELDS as [k, l]}
              {@const v = p.wheel[k] as number}
              <div class="m" class:diff={differs(p, k)} title={`${k}: ${v}`}>
                <div class="mt"><div style:height="{v}%"></div></div>
                <span>{l}</span><b>{v}</b>
              </div>
            {/each}
          </div>
        {:else}
          <div class="faint small">{t("angleOnly")}</div>
        {/if}

        {#if n}<details><summary>{t("tip")}</summary><pre>{n}</pre></details>{/if}
        {#if p.ffb_server}<div class="faint small">⚡ {t("ffbPart")}</div>{/if}

        <div class="actions">
          {#each p.links ?? [] as l}<button class="link" onclick={() => openUrl(l.url)}>{l.label} ↗</button>{/each}
          {#if p.tutorial}<button class="link" onclick={() => openUrl(p.tutorial!)}>{t("tutorial")} ↗</button>{/if}
          <div class="grow"></div>
          {#if p.source === "user"}<button class="ghost sm" onclick={() => remove(p)}>{t("delete")}</button>{/if}
          <button class="primary sm" disabled={!canApply(p)} onclick={() => apply(p)}>{justApplied === p.id ? "✓ " + t("applied") : t("apply")}</button>
        </div>
      </article>
    {:else}
      <p class="faint">{t("noResults")}</p>
    {/each}
  </div>

  <h3>{t("myPresets")}</h3>
  <div class="save">
    <input placeholder={t("presetName")} bind:value={newName} onkeydown={(e) => e.key === "Enter" && saveCurrent()} />
    <button class="ghost" disabled={!newName.trim() || (!app.profile && !ffb.state)} onclick={saveCurrent}>{t("saveCurrent")}</button>
  </div>
</section>

<style>
  .intro p { margin: 0 0 16px; color: var(--text-dim); max-width: 85ch; line-height: 1.5; }
  .bar { display: flex; align-items: center; gap: 10px; margin-bottom: 10px; flex-wrap: wrap; }
  .bar input { width: 200px; }
  .seg { display: inline-grid; grid-auto-flow: column; background: var(--surface-3); border-radius: var(--r-sm); padding: 3px; gap: 3px; }
  .seg button { border: 0; background: transparent; color: var(--text-dim); padding: 7px 14px; border-radius: 6px; cursor: pointer; }
  .seg button.on { background: var(--surface-1); color: var(--text); box-shadow: var(--shadow-sm); }
  .grow { flex: 1; }
  .msg { margin: 4px 0 10px; color: var(--text-dim); font-size: 13px; }
  .msg.err { color: var(--bad); }
  .faint { color: var(--text-faint); font-size: 13px; }
  .small { font-size: 12px; }
  h3 { font-size: 13px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-faint); margin: 26px 0 10px; font-weight: 600; }
  .save { display: flex; gap: 8px; max-width: 520px; }
  .save input { flex: 1; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(290px, 1fr)); gap: 12px; }
  .card { background: var(--surface-2); border: 1px solid var(--line); border-radius: var(--r-lg); padding: 14px 16px; display: flex; flex-direction: column; gap: 10px; transition: border-color 200ms; }
  .card.studio { border-color: color-mix(in srgb, var(--accent) 35%, var(--line)); }
  .card.applied { border-color: var(--ok); }
  .top { display: flex; justify-content: space-between; gap: 10px; }
  .name { font-weight: 600; }
  .badges { display: flex; gap: 6px; margin-top: 6px; }
  .badge { font-size: 10.5px; padding: 2px 7px; border-radius: 999px; background: var(--surface-3); color: var(--text-dim); font-weight: 600; }
  .badge.studio { background: var(--accent-soft); color: var(--accent); }
  .badge.user { background: var(--ok-soft); color: var(--ok); }
  .badge.ffb { background: var(--warn-soft); color: var(--warn); }
  .angle { font-size: 20px; font-weight: 700; font-variant-numeric: tabular-nums; }
  .diff { color: var(--accent); }
  .meters { display: grid; grid-template-columns: repeat(5, 1fr); gap: 8px; }
  .m { display: grid; justify-items: center; gap: 3px; font-size: 10.5px; color: var(--text-faint); }
  .m b { font-size: 13px; color: var(--text); font-variant-numeric: tabular-nums; }
  .m.diff b, .m.diff span { color: var(--accent); }
  .mt { width: 10px; height: 44px; background: var(--surface-3); border-radius: 4px; position: relative; overflow: hidden; }
  .mt div { position: absolute; bottom: 0; left: 0; right: 0; background: var(--text-dim); }
  .m.diff .mt div { background: var(--accent); }
  details { font-size: 12.5px; color: var(--text-dim); }
  summary { cursor: pointer; color: var(--text-dim); }
  details pre { white-space: pre-wrap; font: 12px/1.5 ui-monospace, monospace; background: var(--surface-1); border: 1px solid var(--line); padding: 8px 10px; border-radius: var(--r-sm); margin: 6px 0 0; user-select: text; }
  .actions { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; margin-top: auto; }
  .link { all: unset; cursor: pointer; color: var(--accent); font-size: 12.5px; }
  .link:hover { text-decoration: underline; }
</style>
