<script lang="ts">
  import { slide } from "svelte/transition";
  import { t, settingHelp } from "../i18n.svelte";
  import { app, paramValue, setParam, type ParamKey } from "../store.svelte";
  import StatusChip from "./StatusChip.svelte";

  let { k, min = 0, max = 100, step = 10, unit = "" }: {
    k: Exclude<ParamKey, "reverse">; min?: number; max?: number; step?: number; unit?: string;
  } = $props();

  let open = $state(false);
  const h = $derived(settingHelp(k));
  const value = $derived((paramValue(k) as number | undefined) ?? 0);
  const orig = $derived(app.original?.params[k] as number | undefined);
  const pct = (v: number) => ((v - min) / (max - min)) * 100;
</script>

<div class="row" class:open>
  <div class="head">
    <div class="titles">
      <button class="name" onclick={() => (open = !open)} aria-expanded={open}>
        {h.title}
        <span class="info" class:on={open}>i</span>
      </button>
      <div class="short">{h.short}</div>
    </div>
    <div class="right">
      <StatusChip st={app.status[k]} />
      <output class="val">{value}<small>{unit}</small></output>
    </div>
  </div>

  <div class="slider">
    <input
      type="range" {min} {max} {step} {value}
      style:--p="{pct(value)}%"
      oninput={(e) => setParam(k, +e.currentTarget.value)}
      aria-label={h.title}
    />
    {#if orig !== undefined && orig !== value}
      <button class="orig" style:left="{pct(orig)}%" title="{t('original')}: {orig}{unit}" onclick={() => setParam(k, orig)}
        aria-label="{t('revert')} {orig}{unit}"></button>
    {/if}
    <div class="ticks"><span>{min}{unit}</span><span>{max}{unit}</span></div>
  </div>

  {#if open}
    <div class="help" transition:slide={{ duration: 160 }}>
      <p>{h.body}</p>
      <div class="ex-title">{t("examples")}</div>
      <ul>{#each h.examples as ex}<li>{ex}</li>{/each}</ul>
      {#if h.tip}<p class="tip"><b>{t("tip")}:</b> {h.tip}</p>{/if}
    </div>
  {/if}
</div>

<style>
  .row { padding: 18px 20px; border-radius: var(--r-lg); background: var(--surface-2); border: 1px solid var(--line); }
  .row.open { border-color: var(--line-strong); }
  .head { display: flex; justify-content: space-between; gap: 16px; align-items: flex-start; }
  .name { all: unset; cursor: pointer; font-weight: 600; font-size: 15px; display: inline-flex; gap: 8px; align-items: center; }
  .name:focus-visible { outline: 2px solid var(--accent); outline-offset: 3px; border-radius: 4px; }
  .info { width: 17px; height: 17px; border-radius: 50%; display: inline-grid; place-items: center; font-size: 11px; font-style: italic; font-family: Georgia, serif;
    color: var(--text-dim); border: 1px solid var(--line-strong); transition: all 120ms; }
  .info.on, .name:hover .info { color: var(--accent); border-color: var(--accent); }
  .short { color: var(--text-dim); font-size: 13px; margin-top: 3px; }
  .right { display: flex; align-items: center; gap: 12px; }
  .val { font-variant-numeric: tabular-nums; font-size: 22px; font-weight: 600; min-width: 64px; text-align: right; }
  .val small { font-size: 13px; color: var(--text-dim); margin-left: 1px; }
  .slider { position: relative; margin-top: 14px; }
  input[type="range"] { width: 100%; }
  .orig { position: absolute; top: 2px; width: 4px; height: 16px; margin-left: -2px; border-radius: 2px; border: 0; padding: 0;
    background: var(--text-dim); opacity: 0.6; cursor: pointer; }
  .orig:hover { opacity: 1; background: var(--warn); }
  .ticks { display: flex; justify-content: space-between; font-size: 11px; color: var(--text-faint); margin-top: 4px; font-variant-numeric: tabular-nums; }
  .help { margin-top: 14px; padding-top: 14px; border-top: 1px solid var(--line); font-size: 13.5px; line-height: 1.55; color: var(--text-dim); }
  .help p { margin: 0 0 10px; }
  .ex-title { font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-faint); margin-bottom: 6px; }
  ul { margin: 0 0 6px; padding-left: 18px; }
  li { margin: 3px 0; color: var(--text); }
  .tip { background: var(--accent-soft); color: var(--text); padding: 8px 10px; border-radius: var(--r-sm); margin-top: 8px; }
</style>
