<script lang="ts">
  import { t } from "../i18n.svelte";
  import { app, writeCurve, setLive } from "../store.svelte";
  import StatusChip from "../components/StatusChip.svelte";
  import type { Curve } from "../api";

  const X = 4095, Y = 65535;
  const PEDALS = [
    { i: 0, k: "throttle", color: "var(--ok)" },
    { i: 1, k: "brake", color: "var(--bad)" },
    { i: 2, k: "clutch", color: "var(--accent)" },
  ];
  const shape = (f: (x: number) => number): Curve =>
    [0.25, 0.5, 0.75, 1].map((x) => [Math.round(x * X), Math.round(f(x) * Y)]) as Curve;
  const PRESETS: Record<string, Curve> = {
    linear: shape((x) => x),
    progressive: shape((x) => x * x),
    aggressive: shape((x) => Math.sqrt(x)),
    sCurve: shape((x) => (x < 0.5 ? 2 * x * x : 1 - 2 * (1 - x) * (1 - x))),
  };

  let pedal = $state(0);
  let draft = $state<Curve | null>(null);
  let drag = $state<number | null>(null);
  let svg: SVGSVGElement;

  const stored = $derived(app.profile?.curves[pedal] as Curve | undefined);
  const isEmpty = (c?: Curve) => !c || c.every((p) => p[0] === 0 && p[1] === 0);
  const curve = $derived(draft ?? (isEmpty(stored) ? PRESETS.linear : stored!));
  const dirty = $derived(!!draft);
  const live = $derived(app.live ? [app.live.throttle, app.live.brake, app.live.clutch][pedal] / 255 : null);

  // Output at a given input, by interpolating through (0,0) and the 4 points.
  function outAt(x: number): number {
    const pts: [number, number][] = [[0, 0], ...curve];
    for (let i = 1; i < pts.length; i++) {
      if (x <= pts[i][0]) {
        const [x0, y0] = pts[i - 1], [x1, y1] = pts[i];
        return x1 === x0 ? y1 : y0 + ((x - x0) / (x1 - x0)) * (y1 - y0);
      }
    }
    return pts[pts.length - 1][1];
  }

  const W = 400, H = 300;
  const sx = (x: number) => (x / X) * W, sy = (y: number) => H - (y / Y) * H;
  const path = $derived(["M0," + H, ...curve.map(([x, y]) => `L${sx(x)},${sy(y)}`)].join(" "));

  function move(e: PointerEvent) {
    if (drag === null) return;
    const r = svg.getBoundingClientRect();
    const fx = Math.min(1, Math.max(0, (e.clientX - r.left) / r.width));
    const fy = Math.min(1, Math.max(0, 1 - (e.clientY - r.top) / r.height));
    const c = curve.map((p) => [...p]) as Curve;
    const lo = drag === 0 ? 0 : c[drag - 1][0] + 40;
    const hi = drag === 3 ? X : c[drag + 1][0] - 40;
    c[drag] = [Math.round(Math.min(hi, Math.max(lo, fx * X))), Math.round(fy * Y)];
    if (drag === 3) c[3][0] = X;
    draft = c;
  }
  function choose(i: number) { pedal = i; draft = null; }
</script>

<section>
  <header class="intro">
    <p>{t("pedalsIntro")}</p>
    <button class="ghost" onclick={() => setLive(!app.liveOn)}>{app.liveOn ? "■ " + t("liveStop") : "▶ " + t("liveStart")}</button>
  </header>

  <div class="layout">
    <div class="card editor">
      <div class="seg">
        {#each PEDALS as p}
          <button class:on={pedal === p.i} onclick={() => choose(p.i)} style:--c={p.color}><span class="sw"></span>{t(p.k)}</button>
        {/each}
      </div>

      <div class="plot">
        <span class="ylab">{t("output")}</span>
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <svg bind:this={svg} viewBox="0 0 {W} {H}" onpointermove={move} onpointerup={() => (drag = null)} onpointerleave={() => (drag = null)}
          style:--c={PEDALS[pedal].color}>
          {#each [0.25, 0.5, 0.75] as g}
            <line x1={g * W} y1="0" x2={g * W} y2={H} class="grid" />
            <line x1="0" y1={g * H} x2={W} y2={g * H} class="grid" />
          {/each}
          <line x1="0" y1={H} x2={W} y2="0" class="diag" />
          <path d={path} class="curve" />
          {#if live !== null}
            <line x1={live * W} y1={H} x2={live * W} y2={sy(outAt(live * X))} class="liveline" />
            <circle cx={live * W} cy={sy(outAt(live * X))} r="7" class="livedot" />
          {/if}
          {#each curve as [x, y], i}
            <circle cx={sx(x)} cy={sy(y)} r={drag === i ? 11 : 9} class="pt" onpointerdown={(e) => { drag = i; (e.target as Element).setPointerCapture?.(e.pointerId); }} />
          {/each}
        </svg>
        <span class="xlab">{t("pedalTravel")}</span>
      </div>
    </div>

    <aside class="card side">
      <div class="state">
        {#if isEmpty(stored) && !draft}<span class="faint">{t("noCurve")}</span>{/if}
        <StatusChip st={app.status[`curve:${pedal}`]} />
      </div>
      <div class="presets">
        {#each Object.keys(PRESETS) as k}
          <button class="preset" onclick={() => (draft = PRESETS[k].map((p) => [...p]) as Curve)}>
            <svg viewBox="0 0 40 30"><path d={"M0,30 " + PRESETS[k].map(([x, y]) => `L${(x / X) * 40},${30 - (y / Y) * 30}`).join(" ")} /></svg>
            {t(k)}
          </button>
        {/each}
      </div>
      <p class="hint">{t("curveHint")}</p>
      <button class="primary" disabled={!dirty} onclick={async () => { await writeCurve(pedal, curve); draft = null; }}>{t("write")}</button>
      {#if dirty}<button class="ghost" onclick={() => (draft = null)}>{t("cancel")}</button>{/if}
    </aside>
  </div>
</section>

<style>
  .intro { display: flex; gap: 16px; align-items: center; justify-content: space-between; margin-bottom: 18px; }
  .intro p { margin: 0; color: var(--text-dim); max-width: 75ch; line-height: 1.5; }
  .layout { display: grid; grid-template-columns: minmax(0, 1.6fr) minmax(260px, 1fr); gap: 14px; }
  .card { background: var(--surface-2); border: 1px solid var(--line); border-radius: var(--r-lg); padding: 16px; }
  .seg { display: inline-grid; grid-auto-flow: column; background: var(--surface-3); border-radius: var(--r-sm); padding: 3px; gap: 3px; margin-bottom: 14px; }
  .seg button { border: 0; background: transparent; color: var(--text-dim); padding: 7px 14px; border-radius: 6px; cursor: pointer; display: flex; gap: 8px; align-items: center; }
  .seg button.on { background: var(--surface-1); color: var(--text); box-shadow: var(--shadow-sm); }
  .sw { width: 9px; height: 9px; border-radius: 50%; background: var(--c); }
  .plot { display: grid; grid-template-columns: auto 1fr; grid-template-rows: 1fr auto; gap: 6px; }
  .ylab { writing-mode: vertical-rl; transform: rotate(180deg); font-size: 11px; color: var(--text-faint); text-align: center; }
  .xlab { grid-column: 2; font-size: 11px; color: var(--text-faint); text-align: center; }
  svg { width: 100%; height: auto; background: var(--surface-1); border-radius: var(--r-md); touch-action: none; overflow: visible; }
  .grid { stroke: var(--line); }
  .diag { stroke: var(--line-strong); stroke-dasharray: 4 5; }
  .curve { fill: none; stroke: var(--c); stroke-width: 3; stroke-linejoin: round; }
  .pt { fill: var(--surface-1); stroke: var(--c); stroke-width: 3; cursor: grab; }
  .pt:hover { fill: var(--c); }
  .livedot { fill: var(--c); filter: drop-shadow(0 0 6px var(--c)); }
  .liveline { stroke: var(--c); stroke-dasharray: 2 3; opacity: 0.6; }
  .side { display: flex; flex-direction: column; gap: 12px; }
  .state { display: flex; gap: 10px; align-items: center; min-height: 26px; }
  .faint { color: var(--text-faint); font-size: 13px; }
  .presets { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .preset { display: flex; align-items: center; gap: 10px; padding: 8px 10px; border-radius: var(--r-sm); border: 1px solid var(--line-strong); background: var(--surface-1); color: var(--text); cursor: pointer; font-size: 13px; }
  .preset:hover { border-color: var(--accent); }
  .preset svg { width: 34px; background: none; }
  .preset path { fill: none; stroke: var(--accent); stroke-width: 2.5; }
  .hint { font-size: 13px; color: var(--text-dim); line-height: 1.5; margin: 0; }
  @media (max-width: 980px) { .layout { grid-template-columns: 1fr; } }
</style>
