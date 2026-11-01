<script lang="ts">
  import { slide } from "svelte/transition";
  import { t, settingHelp } from "../i18n.svelte";
  import { app, paramValue, setParam, setParams, revertParams } from "../store.svelte";
  import ParamRow from "../components/ParamRow.svelte";
  import StatusChip from "../components/StatusChip.svelte";

  const rev = $derived(!!paramValue("reverse"));
  let revOpen = $state(false);
  // LiteStar's phone app writes some presets two bytes off; the wheel then holds e.g. 8994° / FFB 132.
  const broken = $derived.by(() => {
    const p = app.profile?.params;
    if (!p) return false;
    return p.angle < 90 || p.angle > 1080 || [p.ffb, p.spring, p.damping, p.friction, p.inertia].some((v) => v > 100);
  });
  const changed = $derived(
    !!app.original && !!app.profile &&
    JSON.stringify(app.original.params) !== JSON.stringify(app.profile.params),
  );
</script>

<section>
  {#if broken && app.profile}
    <div class="broken" role="alert">
      <div>
        <b>{t("brokenTitle")}</b>
        <p>{t("brokenBody").replace("{angle}", String(app.profile.params.angle)).replace("{ffb}", String(app.profile.params.ffb))}</p>
      </div>
      <button class="primary" onclick={() => setParams({ angle: 900, ffb: 100 })}>{t("brokenFix")}</button>
    </div>
  {/if}
  <header class="intro">
    <p>{t("tuningIntro")}</p>
    {#if changed}
      <button class="ghost" onclick={revertParams}>↺ {t("revertAll")}</button>
    {/if}
  </header>

  <div class="grid">
    <ParamRow k="angle" min={90} max={1080} step={10} unit="°" />
    <ParamRow k="ffb" />

    <div class="row">
      <div class="head">
        <div>
          <button class="name" onclick={() => (revOpen = !revOpen)}>{settingHelp("reverse").title} <span class="info">i</span></button>
          <div class="short">{settingHelp("reverse").short}</div>
        </div>
        <div class="right">
          <StatusChip st={app.status.reverse} />
          <label class="switch">
            <input type="checkbox" checked={rev} onchange={(e) => setParam("reverse", e.currentTarget.checked, 0)} />
            <span></span>
          </label>
          <span class="state">{rev ? t("reverseOn") : t("reverseOff")}</span>
        </div>
      </div>
      {#if revOpen}
        <div class="help" transition:slide={{ duration: 160 }}>
          <p>{settingHelp("reverse").body}</p>
          <ul>{#each settingHelp("reverse").examples as ex}<li>{ex}</li>{/each}</ul>
        </div>
      {/if}
    </div>

    <ParamRow k="spring" />
    <ParamRow k="damping" />
    <ParamRow k="friction" />
    <ParamRow k="inertia" />
  </div>
</section>

<style>
  .broken { display: flex; gap: 16px; align-items: center; justify-content: space-between; padding: 14px 18px; margin-bottom: 16px; border-radius: var(--r-lg); background: var(--warn-soft); border: 1px solid color-mix(in srgb, var(--warn) 40%, transparent); }
  .broken p { margin: 4px 0 0; color: var(--text-dim); line-height: 1.5; max-width: 80ch; }
  .intro { display: flex; gap: 16px; align-items: center; justify-content: space-between; margin-bottom: 18px; }
  .intro p { margin: 0; color: var(--text-dim); max-width: 70ch; line-height: 1.5; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(380px, 1fr)); gap: 12px; }
  .row { padding: 18px 20px; border-radius: var(--r-lg); background: var(--surface-2); border: 1px solid var(--line); }
  .head { display: flex; justify-content: space-between; gap: 16px; align-items: center; }
  .name { all: unset; cursor: pointer; font-weight: 600; font-size: 15px; display: inline-flex; gap: 8px; align-items: center; }
  .info { width: 17px; height: 17px; border-radius: 50%; display: inline-grid; place-items: center; font-size: 11px; font-style: italic; font-family: Georgia, serif; color: var(--text-dim); border: 1px solid var(--line-strong); }
  .short { color: var(--text-dim); font-size: 13px; margin-top: 3px; }
  .right { display: flex; align-items: center; gap: 12px; }
  .state { min-width: 56px; font-weight: 600; }
  .help { margin-top: 14px; padding-top: 14px; border-top: 1px solid var(--line); font-size: 13.5px; line-height: 1.55; color: var(--text-dim); }
  .help p { margin: 0 0 8px; }
  .help ul { margin: 0; padding-left: 18px; color: var(--text); }
</style>
