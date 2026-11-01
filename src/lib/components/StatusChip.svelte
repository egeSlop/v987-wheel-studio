<script lang="ts">
  import { t } from "../i18n.svelte";
  import type { ApplyState } from "../store.svelte";
  let { st }: { st: ApplyState | undefined } = $props();
  const label = { sending: "stSending", acked: "stAcked", verified: "stVerified", failed: "stFailed" } as const;
  const long = { sending: "stSending", acked: "stAckedLong", verified: "stVerifiedLong", failed: "stFailedLong" } as const;
</script>

{#if st}
  <span class="chip {st}" title={t(long[st])} aria-live="polite">
    {#if st === "sending"}<span class="spin"></span>
    {:else if st === "verified"}<svg viewBox="0 0 16 16"><path d="M3 8.5l3 3 7-7" /></svg>
    {:else if st === "acked"}<svg viewBox="0 0 16 16"><path d="M3 8.5l3 3 7-7" /></svg>
    {:else}<svg viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" /></svg>{/if}
    {t(label[st])}
  </span>
{/if}

<style>
  .chip {
    display: inline-flex; align-items: center; gap: 6px;
    font-size: 12px; font-weight: 500; line-height: 1;
    padding: 5px 9px; border-radius: 999px; white-space: nowrap;
    border: 1px solid transparent;
    animation: pop 180ms ease;
  }
  .chip svg { width: 13px; height: 13px; fill: none; stroke: currentColor; stroke-width: 2; stroke-linecap: round; stroke-linejoin: round; }
  .sending { color: var(--text-dim); background: var(--surface-3); }
  .acked { color: var(--accent); background: var(--accent-soft); }
  .verified { color: var(--ok); background: var(--ok-soft); }
  .failed { color: var(--bad); background: var(--bad-soft); }
  .spin { width: 10px; height: 10px; border-radius: 50%; border: 2px solid currentColor; border-right-color: transparent; animation: spin 700ms linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  @keyframes pop { from { transform: scale(0.9); opacity: 0; } }
</style>
