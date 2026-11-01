<script lang="ts">
  import { t } from "../i18n.svelte";
  import { app, mappingOf, remap, setLive } from "../store.svelte";
  import { CONTROLS, TARGETS, controlByCode, controlBySvg, targetLabel, DISABLED, type Target } from "../keys";
  import DeviceArt from "../components/DeviceArt.svelte";
  import StatusChip from "../components/StatusChip.svelte";

  let selected = $state<number | null>(null);
  let group = $state<Target["group"]>("pad");
  let filter = $state("");

  const sel = $derived(selected === null ? null : controlByCode.get(selected) ?? null);
  const remappedIds = $derived(CONTROLS.filter((c) => mappingOf(c.code) !== c.code).map((c) => c.svg));
  const pressedIds = $derived((app.live?.pressed ?? []).map((c) => controlByCode.get(c)?.svg).filter(Boolean) as string[]);
  const options = $derived(
    TARGETS.filter((x) => x.group === group && (!filter || x.label.toLowerCase().includes(filter.toLowerCase()))),
  );

  // Pressing a physical control while live view is on selects it.
  $effect(() => {
    const p = app.live?.pressed.filter((c) => controlByCode.has(c) && c < 20 || (c >= 23 && c <= 33));
    if (p && p.length === 1) selected = p[0];
  });

  function pick(svgId: string) {
    const c = controlBySvg.get(svgId);
    if (c) selected = c.code;
  }
</script>

<section>
  <header class="intro">
    <p>{t("buttonsIntro")} <span class="faint">{t("perMode")}</span></p>
    <button class="ghost" onclick={() => setLive(!app.liveOn)}>{app.liveOn ? "■ " + t("liveStop") : "▶ " + t("liveStart")}</button>
  </header>

  <div class="layout">
    <div class="arts">
      <div class="card"><DeviceArt kind="wheel" onpick={pick} selected={sel?.svg} marked={remappedIds} active={pressedIds} /></div>
      <div class="card small"><DeviceArt kind="shifter" onpick={pick} selected={sel?.svg} marked={remappedIds} active={pressedIds} /></div>
    </div>

    <aside class="card panel">
      {#if sel}
        <div class="sel-head">
          <div class="big">{sel.label}</div>
          <div>
            <div class="faint">{t("sends")}</div>
            <div class="now">
              {targetLabel(mappingOf(sel.code))}
              {#if mappingOf(sel.code) !== sel.code}<span class="badge">{t("remapped")}</span>{/if}
            </div>
          </div>
          <div class="grow"></div>
          <StatusChip st={app.status[`key:${sel.code}`]} />
        </div>

        <div class="faint label">{t("remapTo")}</div>
        <div class="seg">
          {#each [["pad", "groupPad"], ["keyboard", "groupKeyboard"], ["mouse", "groupMouse"], ["off", "groupOff"]] as [g, k]}
            <button class:on={group === g} onclick={() => { group = g as Target["group"]; filter = ""; }}>{t(k)}</button>
          {/each}
        </div>
        {#if group === "keyboard"}
          <input class="search" placeholder="A, F5, Space, Num 1…" bind:value={filter} />
        {/if}
        <div class="keys" class:pad={group === "pad"}>
          {#each options as o (o.code)}
            <button class="key" class:current={mappingOf(sel.code) === o.code} onclick={() => remap(sel.code, o.code)}>{o.label}</button>
          {/each}
        </div>
        {#if mappingOf(sel.code) !== sel.code}
          <button class="ghost reset" onclick={() => remap(sel.code, sel.code)}>↺ {t("resetMapping")} ({sel.label})</button>
        {/if}
      {:else}
        <div class="empty">{t("pickControl")}</div>
      {/if}

      <div class="list">
        {#each CONTROLS.filter((c) => c.where !== "pedals") as c (c.code)}
          {@const m = mappingOf(c.code)}
          <button class="li" class:on={selected === c.code} onclick={() => (selected = c.code)}>
            <span class="lk">{c.label}</span>
            <span class="arrow">→</span>
            <span class:changed={m !== c.code} class:off={m === DISABLED}>{targetLabel(m)}</span>
          </button>
        {/each}
      </div>
    </aside>
  </div>
</section>

<style>
  .intro { display: flex; gap: 16px; align-items: center; justify-content: space-between; margin-bottom: 18px; }
  .intro p { margin: 0; color: var(--text-dim); max-width: 80ch; line-height: 1.5; }
  .faint { color: var(--text-faint); }
  .layout { display: grid; grid-template-columns: minmax(0, 1.3fr) minmax(320px, 1fr); gap: 14px; }
  .arts { display: grid; grid-template-columns: 1fr 0.42fr; gap: 14px; align-items: start; }
  .card { background: var(--surface-2); border: 1px solid var(--line); border-radius: var(--r-lg); padding: 14px; }
  .panel { display: flex; flex-direction: column; gap: 12px; min-height: 0; }
  .sel-head { display: flex; align-items: center; gap: 14px; }
  .big { font-size: 26px; font-weight: 700; min-width: 64px; height: 64px; border-radius: 14px; display: grid; place-items: center; background: var(--accent-soft); color: var(--accent); padding: 0 10px; }
  .now { font-size: 18px; font-weight: 600; display: flex; gap: 8px; align-items: center; }
  .badge { font-size: 11px; font-weight: 600; color: var(--warn); background: var(--warn-soft); border-radius: 999px; padding: 3px 8px; }
  .grow { flex: 1; }
  .label { font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; }
  .seg { display: grid; grid-template-columns: repeat(4, 1fr); background: var(--surface-3); border-radius: var(--r-sm); padding: 3px; gap: 3px; }
  .seg button { border: 0; background: transparent; color: var(--text-dim); padding: 7px 4px; border-radius: 6px; font-size: 12.5px; cursor: pointer; }
  .seg button.on { background: var(--surface-1); color: var(--text); box-shadow: var(--shadow-sm); }
  .search { width: 100%; }
  .keys { display: grid; grid-template-columns: repeat(auto-fill, minmax(58px, 1fr)); gap: 6px; max-height: 220px; overflow: auto; padding: 2px; }
  .key { padding: 8px 4px; border-radius: 8px; border: 1px solid var(--line-strong); background: var(--surface-1); color: var(--text); cursor: pointer; font-size: 13px; }
  .key:hover { border-color: var(--accent); }
  .key.current { background: var(--accent); border-color: var(--accent); color: var(--on-accent); }
  .reset { align-self: flex-start; }
  .empty { color: var(--text-dim); padding: 28px; text-align: center; border: 1px dashed var(--line-strong); border-radius: var(--r-md); }
  .list { border-top: 1px solid var(--line); padding-top: 10px; display: grid; grid-template-columns: 1fr 1fr; gap: 2px 10px; overflow: auto; }
  .li { all: unset; cursor: pointer; display: grid; grid-template-columns: 52px 16px 1fr; padding: 5px 8px; border-radius: 6px; font-size: 13px; }
  .li:hover, .li.on { background: var(--surface-3); }
  .lk { font-weight: 600; }
  .arrow { color: var(--text-faint); }
  .changed { color: var(--warn); font-weight: 600; }
  .off { color: var(--text-faint); }
  @media (max-width: 1100px) { .layout { grid-template-columns: 1fr; } }
</style>
