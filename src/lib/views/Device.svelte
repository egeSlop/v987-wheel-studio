<script lang="ts">
  import { t } from "../i18n.svelte";
  import { app, mode, switchMode, factoryReset } from "../store.svelte";
  import { MODES } from "../keys";
  import StatusChip from "../components/StatusChip.svelte";

  let confirmReset = $state(false);
  let pendingMode = $state<number | null>(null);
  let copied = $state(false);

  function copyLog() {
    const txt = app.log.map((l) => `${new Date(l.t).toISOString().slice(11, 23)} ${l.dir} ${l.hex}${l.note ? "  # " + l.note : ""}`).join("\n");
    navigator.clipboard.writeText(txt);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<section class="cols">
  <div class="col">
    <div class="card">
      <h3>{t("info")}</h3>
      <dl>
        <dt>Bluetooth</dt><dd>{app.name}</dd>
        <dt>{t("firmware")}</dt><dd>{app.state?.firmware ?? "–"}</dd>
        <dt>{t("mode")}</dt><dd>{MODES[mode()] ?? mode()}</dd>
        <dt>{t("pedals")}</dt><dd>{app.state?.pedalState ? t("detected") : t("notDetected")}</dd>
        <dt>{t("shifter")}</dt><dd>{app.state?.leverState ? t("detected") : t("notDetected")}</dd>
        <dt>USB</dt><dd>11ff:2370 · LITE STAR</dd>
      </dl>
    </div>

    <div class="card">
      <div class="h"><h3>{t("modeTitle")}</h3><StatusChip st={app.status.mode} /></div>
      <p>{t("modeHint")}</p>
      <div class="modes">
        {#each [1, 2] as m}
          <button class="mode" class:on={mode() === m} onclick={() => mode() !== m && (pendingMode = m)}>
            <b>{MODES[m]}</b>
            <span>{m === 1 ? "Sim · FFB" : "Xbox · no FFB"}</span>
          </button>
        {/each}
      </div>
      {#if pendingMode !== null}
        <div class="confirm">
          <p>⚠ {t("modeWarn")}</p>
          <div class="btns">
            <button class="ghost" onclick={() => (pendingMode = null)}>{t("cancel")}</button>
            <button class="primary" onclick={() => { const m = pendingMode!; pendingMode = null; switchMode(m); }}>{t("confirm")}: {MODES[pendingMode]}</button>
          </div>
        </div>
      {/if}
    </div>

    <div class="card danger">
      <div class="h"><h3>{t("resetTitle")}</h3><StatusChip st={app.status.reset} /></div>
      <p>{t("resetHint")}</p>
      {#if !confirmReset}
        <button class="ghost bad" onclick={() => (confirmReset = true)}>{t("reset")}…</button>
      {:else}
        <div class="confirm">
          <p>{t("resetConfirm")}</p>
          <div class="btns">
            <button class="ghost" onclick={() => (confirmReset = false)}>{t("cancel")}</button>
            <button class="primary bad" onclick={() => { confirmReset = false; factoryReset(); }}>{t("reset")}</button>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <div class="card log">
    <div class="h">
      <h3>{t("protocolLog")}</h3>
      <div class="btns">
        <button class="ghost sm" onclick={copyLog}>{copied ? t("copied") : "⧉"}</button>
        <button class="ghost sm" onclick={() => (app.log = [])}>{t("clear")}</button>
      </div>
    </div>
    <div class="lines">
      {#each app.log.slice(-250).toReversed() as l}
        <div class="ln {l.dir}"><span class="dir">{l.dir === "tx" ? "→" : "←"}</span><code>{l.hex || "—"}</code>{#if l.note}<em>{l.note}</em>{/if}</div>
      {/each}
    </div>
  </div>
</section>

<style>
  .cols { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 14px; align-items: start; }
  .col { display: grid; gap: 14px; }
  .card { background: var(--surface-2); border: 1px solid var(--line); border-radius: var(--r-lg); padding: 16px 18px; }
  .card p { color: var(--text-dim); font-size: 13.5px; line-height: 1.55; margin: 6px 0 12px; }
  h3 { margin: 0; font-size: 15px; }
  .h { display: flex; justify-content: space-between; align-items: center; gap: 10px; }
  dl { display: grid; grid-template-columns: auto 1fr; gap: 6px 18px; margin: 12px 0 0; font-size: 14px; }
  dt { color: var(--text-dim); }
  dd { margin: 0; font-weight: 500; }
  .modes { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
  .mode { text-align: left; padding: 12px 14px; border-radius: var(--r-md); border: 1px solid var(--line-strong); background: var(--surface-1); color: var(--text); cursor: pointer; display: grid; gap: 3px; }
  .mode span { font-size: 12px; color: var(--text-dim); }
  .mode.on { border-color: var(--accent); background: var(--accent-soft); }
  .confirm { margin-top: 12px; padding: 12px; border-radius: var(--r-md); background: var(--warn-soft); }
  .confirm p { color: var(--text); margin: 0 0 10px; }
  .btns { display: flex; gap: 8px; justify-content: flex-end; }
  .bad { color: var(--bad); }
  button.primary.bad { background: var(--bad); color: white; }
  .log { display: flex; flex-direction: column; max-height: calc(100vh - 150px); }
  .lines { margin-top: 10px; overflow: auto; font-size: 12px; display: grid; gap: 1px; }
  .ln { display: flex; gap: 8px; padding: 3px 6px; border-radius: 4px; }
  .ln.tx { color: var(--accent); }
  .ln.rx { color: var(--text-dim); }
  .ln code { font-family: ui-monospace, "JetBrains Mono", monospace; }
  .ln em { color: var(--text-faint); }
  @media (max-width: 980px) { .cols { grid-template-columns: 1fr; } }
</style>
