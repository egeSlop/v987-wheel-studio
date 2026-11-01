<script lang="ts">
  import { t } from "../i18n.svelte";
  import { app, scan, connect } from "../store.svelte";
  import DeviceArt from "./DeviceArt.svelte";
</script>

<div class="connect">
  <div class="hero"><DeviceArt kind="wheel" /></div>
  <h2>{t("connectTitle")}</h2>
  <p class="lead">{t("scanHint")}</p>
  <p class="lead small">{t("connectBody")}</p>

  <div class="found">
    {#each app.found as d (d.id)}
      <button class="dev" disabled={app.phase === "connecting"} onclick={() => connect(d)}>
        <div>
          <div class="dn">{d.name}</div>
          <div class="di">{d.id}{d.rssi != null ? ` · ${d.rssi} dBm` : ""}</div>
        </div>
        <span class="go">{app.phase === "connecting" ? t("connecting") : t("connect") + " →"}</span>
      </button>
    {:else}
      {#if app.phase !== "scanning"}<div class="none">{t("noWheel")}</div>{/if}
    {/each}
  </div>

  <button class="primary" disabled={app.phase === "scanning" || app.phase === "connecting"} onclick={scan}>
    {app.phase === "scanning" ? t("scanning") : "⟳ " + t("scan")}
  </button>
  {#if app.error}<div class="err-text">{app.error}</div>{/if}
</div>

<style>
  .connect { display: flex; flex-direction: column; align-items: center; gap: 12px; padding: 20px 0 40px; text-align: center; }
  .hero { width: min(240px, 50vw); filter: drop-shadow(0 18px 40px rgba(0, 0, 0, 0.35)); }
  h2 { margin: 6px 0 0; font-size: 22px; }
  .lead { color: var(--text-dim); max-width: 60ch; line-height: 1.55; margin: 0; }
  .small { font-size: 13px; color: var(--text-faint); }
  .found { display: grid; gap: 8px; width: min(520px, 90%); margin-top: 6px; }
  .dev { display: flex; justify-content: space-between; align-items: center; text-align: left; padding: 14px 16px; border-radius: var(--r-md); border: 1px solid var(--line-strong); background: var(--surface-2); color: var(--text); cursor: pointer; }
  .dev:hover:not(:disabled) { border-color: var(--accent); }
  .dn { font-weight: 600; }
  .di { font-size: 12px; color: var(--text-faint); margin-top: 3px; font-family: ui-monospace, monospace; }
  .go { color: var(--accent); font-weight: 600; }
  .none { color: var(--text-dim); padding: 16px; border: 1px dashed var(--line-strong); border-radius: var(--r-md); }
  .err-text { color: var(--bad); max-width: 60ch; }
</style>
