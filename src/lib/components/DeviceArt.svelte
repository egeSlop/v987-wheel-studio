<script lang="ts">
  import wheel from "../../assets/device/wheel.svg?raw";
  import shifter from "../../assets/device/shifter.svg?raw";
  import pedals from "../../assets/device/pedals.svg?raw";

  const ART = { wheel, shifter, pedals } as const;

  let {
    kind,
    active = [],
    selected = null,
    marked = [],
    rotate = 0,
    onpick,
  }: {
    kind: keyof typeof ART;
    /** svg ids lit as "pressed" */
    active?: string[];
    /** svg id outlined as "selected" */
    selected?: string | null;
    /** svg ids with a small "changed" marker */
    marked?: string[];
    /** degrees, for the wheel's live steering */
    rotate?: number;
    onpick?: (svgId: string) => void;
  } = $props();

  let host: HTMLDivElement;

  $effect(() => {
    const els = host.querySelectorAll<SVGElement>(".ctl");
    const a = new Set(active), m = new Set(marked);
    els.forEach((el) => {
      el.classList.toggle("active", a.has(el.id));
      el.classList.toggle("selected", el.id === selected);
      el.classList.toggle("marked", m.has(el.id));
      el.classList.toggle("pickable", !!onpick && !["rim", "rim-marker", "led-bar", "status-led", "hub-logo", "shifter-body", "shifter-knob", "gate", "base-plate", "heel-rest"].includes(el.id));
    });
  });

  function click(e: MouseEvent) {
    const el = (e.target as Element).closest<SVGElement>(".ctl.pickable");
    if (el && onpick) onpick(el.id);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="art {kind}" bind:this={host} onclick={click} style:--rot="{rotate}deg">
  {@html ART[kind]}
</div>

<style>
  .art {
    --dev-body: var(--surface-3);
    --dev-edge: var(--line-strong);
    --dev-ctl: var(--surface-4);
    --dev-label: var(--text-dim);
    width: 100%;
    display: grid;
    place-items: center;
  }
  .art :global(svg) { width: 100%; height: auto; max-height: 100%; overflow: visible; }
  .wheel :global(svg) { transform: rotate(var(--rot)); transition: transform 60ms linear; }
  .art :global(.ctl) { transition: fill 120ms ease, filter 120ms ease, opacity 120ms ease; }
  .art :global(.ctl.pickable) { cursor: pointer; }
  .art :global(.ctl.pickable:hover) { filter: brightness(1.5); }
  .art :global(.ctl.active) { fill: var(--accent); filter: drop-shadow(0 0 6px var(--accent)); }
  .art :global(.ctl.selected) { stroke: var(--accent); stroke-width: 3px; filter: drop-shadow(0 0 4px var(--accent)); }
  .art :global(.ctl.marked) { fill: var(--warn); }
  .art :global(.ctl.marked.active) { fill: var(--accent); }
</style>
