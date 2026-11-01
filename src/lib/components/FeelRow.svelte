<script lang="ts">
  import { slide } from "svelte/transition";
  import { t } from "../i18n.svelte";
  import { ffbHelp } from "../i18n-ffb";

  let { k, value, onchange, def }: { k: string; value: number; onchange: (v: number) => void; def?: number } = $props();
  let open = $state(false);
  const h = $derived(ffbHelp(k));
</script>

<div class="row" class:open>
  <div class="head">
    <div>
      <button class="name" onclick={() => (open = !open)} aria-expanded={open}>{h.title} <span class="info" class:on={open}>i</span></button>
      <div class="short">{h.short}</div>
    </div>
    <output class="val">{value.toFixed(2)}</output>
  </div>
  <div class="slider">
    <input type="range" min="0" max="1" step="0.01" {value} style:--p="{value * 100}%"
      oninput={(e) => onchange(+e.currentTarget.value)} aria-label={h.title} />
    {#if def !== undefined && Math.abs(def - value) > 0.005}
      <button class="def" style:left="{def * 100}%" title="{t('resetFeel')}: {def}" aria-label="{t('resetFeel')} {def}" onclick={() => onchange(def)}></button>
    {/if}
  </div>
  {#if open}
    <div class="help" transition:slide={{ duration: 160 }}>
      <p>{h.body}</p>
      <div class="ex-title">{t("examples")}</div>
      <ul>{#each h.examples as ex}<li>{ex}</li>{/each}</ul>
    </div>
  {/if}
</div>

<style>
  .row { padding: 14px 16px; border-radius: var(--r-md); background: var(--surface-2); border: 1px solid var(--line); }
  .row.open { border-color: var(--line-strong); }
  .head { display: flex; justify-content: space-between; gap: 12px; align-items: flex-start; }
  .name { all: unset; cursor: pointer; font-weight: 600; display: inline-flex; gap: 8px; align-items: center; }
  .name:focus-visible { outline: 2px solid var(--accent); outline-offset: 3px; border-radius: 4px; }
  .info { width: 16px; height: 16px; border-radius: 50%; display: inline-grid; place-items: center; font-size: 10.5px; font-style: italic; font-family: Georgia, serif; color: var(--text-dim); border: 1px solid var(--line-strong); }
  .info.on, .name:hover .info { color: var(--accent); border-color: var(--accent); }
  .short { color: var(--text-dim); font-size: 12.5px; margin-top: 2px; }
  .val { font-variant-numeric: tabular-nums; font-size: 18px; font-weight: 600; }
  .slider { position: relative; margin-top: 10px; }
  input[type="range"] { width: 100%; }
  .def { position: absolute; top: 2px; width: 4px; height: 16px; margin-left: -2px; border-radius: 2px; border: 0; padding: 0; background: var(--text-dim); opacity: 0.6; cursor: pointer; }
  .def:hover { opacity: 1; background: var(--warn); }
  .help { margin-top: 12px; padding-top: 12px; border-top: 1px solid var(--line); font-size: 13px; line-height: 1.55; color: var(--text-dim); }
  .help p { margin: 0 0 8px; }
  .ex-title { font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-faint); margin-bottom: 4px; }
  ul { margin: 0; padding-left: 18px; }
  li { margin: 3px 0; color: var(--text); }
</style>
