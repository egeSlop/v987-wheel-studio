<script lang="ts">
  import { onDestroy } from "svelte";
  import { t } from "../i18n.svelte";
  import { app, setLive } from "../store.svelte";
  import { controlByCode } from "../keys";
  import DeviceArt from "../components/DeviceArt.svelte";

  const pressedIds = $derived((app.live?.pressed ?? []).map((c) => controlByCode.get(c)?.svg).filter(Boolean) as string[]);
  // Test-mode steering is a u8 with centre at 128; scale it to the configured rotation angle.
  const lockDeg = $derived((app.profile?.params.angle ?? 900) / 2);
  const steerDeg = $derived(app.live ? ((app.live.angle - 128) / 128) * lockDeg : 0);
  const gear = $derived(app.live ? (app.live.gear === 255 ? "R" : app.live.gear === 0 ? t("neutral") : String(app.live.gear)) : "–");
  const pedals = $derived([
    { k: "clutch", v: app.live?.clutch ?? 0, id: "pedal-clutch" },
    { k: "brake", v: app.live?.brake ?? 0, id: "pedal-brake" },
    { k: "throttle", v: app.live?.throttle ?? 0, id: "pedal-throttle" },
  ]);

  onDestroy(() => { if (app.liveOn) setLive(false); });
</script>

<section>
  <header class="intro">
    <p>{t("liveIntro")}</p>
    <button class={app.liveOn ? "ghost" : "primary"} onclick={() => setLive(!app.liveOn)}>
      {#if app.liveOn}<span class="dot"></span>{t("liveStop")}{:else}▶ {t("liveStart")}{/if}
    </button>
  </header>

  <div class="stage" class:idle={!app.liveOn}>
    <div class="card wheel">
      <DeviceArt kind="wheel" active={pressedIds} rotate={steerDeg} />
      <div class="readout">
        <span>{t("steering")}</span>
        <b>{steerDeg > 0 ? "+" : ""}{steerDeg.toFixed(0)}°</b>
      </div>
    </div>

    <div class="side">
      <div class="card shifter">
        <DeviceArt kind="shifter" active={pressedIds} />
        <div class="gear"><span>{t("gear")}</span><b>{gear}</b></div>
      </div>

      <div class="card pedals">
        <DeviceArt kind="pedals" active={pressedIds.filter((id) => id.startsWith("pedal"))} />
        <div class="bars">
          {#each pedals as p}
            <div class="bar">
              <div class="track"><div class="fill {p.k}" style:height="{(p.v / 255) * 100}%"></div></div>
              <div class="pv">{Math.round((p.v / 255) * 100)}%</div>
              <div class="pl">{t(p.k)}</div>
            </div>
          {/each}
        </div>
      </div>
    </div>
  </div>
</section>

<style>
  .intro { display: flex; gap: 16px; align-items: center; justify-content: space-between; margin-bottom: 18px; }
  .intro p { margin: 0; color: var(--text-dim); max-width: 70ch; line-height: 1.5; }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--bad); display: inline-block; margin-right: 8px; animation: blink 1s infinite; }
  @keyframes blink { 50% { opacity: 0.3; } }
  .stage { display: grid; grid-template-columns: minmax(0, 1.35fr) minmax(0, 1fr); gap: 14px; transition: opacity 200ms; }
  .stage.idle { opacity: 0.55; }
  .card { background: var(--surface-2); border: 1px solid var(--line); border-radius: var(--r-lg); padding: 16px; position: relative; }
  .wheel { display: grid; place-items: center; min-height: 420px; }
  .readout { position: absolute; left: 18px; bottom: 14px; display: flex; flex-direction: column; }
  .readout span, .gear span { font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-faint); }
  .readout b { font-size: 26px; font-variant-numeric: tabular-nums; }
  .side { display: grid; gap: 14px; grid-template-rows: auto auto; }
  .shifter { display: grid; grid-template-columns: 1fr auto; align-items: center; gap: 12px; }
  .shifter :global(.art) { max-width: 170px; justify-self: center; }
  .gear { text-align: center; padding-right: 10px; }
  .gear b { display: block; font-size: 54px; line-height: 1; font-weight: 700; color: var(--accent); min-width: 60px; }
  .pedals { display: grid; grid-template-columns: 1fr; gap: 10px; }
  .pedals :global(.art) { max-width: 300px; justify-self: center; }
  .bars { display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px; }
  .bar { display: grid; justify-items: center; gap: 4px; }
  .track { width: 22px; height: 90px; border-radius: 6px; background: var(--surface-3); position: relative; overflow: hidden; }
  .fill { position: absolute; bottom: 0; left: 0; right: 0; transition: height 40ms linear; }
  .fill.throttle { background: var(--ok); }
  .fill.brake { background: var(--bad); }
  .fill.clutch { background: var(--accent); }
  .pv { font-variant-numeric: tabular-nums; font-weight: 600; font-size: 13px; }
  .pl { font-size: 11px; color: var(--text-dim); }
  @media (max-width: 980px) { .stage { grid-template-columns: 1fr; } }
</style>
