<script lang="ts">
  import { i18n, t } from "../i18n.svelte";
  import { api } from "../api";
  import { app, mode } from "../store.svelte";

  type Status = "supported" | "unsupported" | "unknown";
  interface Feature {
    id: string;
    cmd: string;
    title: { en: string; tr: string };
    body: { en: string; tr: string };
    /** Harmless probe; omitted when the only test is the action itself. */
    probe?: () => number[];
  }

  const FEATURES: Feature[] = [
    { id: "calib", cmd: "53 0F",
      title: { en: "Center calibration", tr: "Merkez kalibrasyonu" },
      body: { en: "Tells the wheel that where it is now is the center. Use it if the wheel sits a little off when you drive straight. Hold the wheel straight, then press Calibrate.",
              tr: "Direksiyona şu anki konumunun tam orta olduğunu söyler. Düz giderken direksiyon biraz yamuk duruyorsa kullanın. Direksiyonu dümdüz tutup Kalibre et'e basın." } },
    { id: "road", cmd: "53 10",
      title: { en: "Road sensitivity", tr: "Yol hassasiyeti" },
      body: { en: "On PXN direct-drive bases this boosts fine road texture separately from the other forces. The vendor app doesn't offer it for this wheel.",
              tr: "PXN direct-drive tabanlarda ince yol dokusunu diğer kuvvetlerden ayrı olarak güçlendirir. Üreticinin uygulaması bu direksiyonda göstermiyor." } },
    { id: "led", cmd: "53 0B/0C",
      title: { en: "LED colors", tr: "LED renkleri" },
      body: { en: "Sets the color of the light bar on the hub. On PXN wheels this has 10 RGB segments.",
              tr: "Göbekteki ışık çubuğunun rengini ayarlar. PXN direksiyonlarda 10 RGB bölümden oluşur." } },
    { id: "vkey", cmd: "53 13",
      title: { en: "Virtual key press", tr: "Sanal tuş basma" },
      body: { en: "Lets the app press wheel buttons for you, for example to trigger something in a game from the PC. Checking it sends a press with no buttons.",
              tr: "Uygulamanın direksiyon tuşlarına sizin yerinize basmasını sağlar, örneğin bilgisayardan oyunda bir şey tetiklemek için. Denerken hiçbir tuş içermeyen bir basma gönderilir." },
      probe: () => [0x53, 0x13, 0, 0, 0] },
    { id: "fan", cmd: "53 11",
      title: { en: "Fan level", tr: "Fan seviyesi" },
      body: { en: "Cooling fan speed on direct-drive bases. The V987 has no fan the app can control.",
              tr: "Direct-drive tabanlarda soğutma fanının hızı. V987'de uygulamanın kontrol edebileceği bir fan yok." } },
    { id: "dclutch", cmd: "53 12",
      title: { en: "Double clutch (paddles)", tr: "Çift debriyaj (kulakçıklar)" },
      body: { en: "On PXN wheels with clutch paddles, the rear paddles work as a two-stage clutch for race starts.",
              tr: "Debriyaj kulakçığı olan PXN direksiyonlarda arka kulakçıklar kalkışta iki kademeli debriyaj olarak çalışır." },
      probe: () => [0x53, 0x12, 0] },
    { id: "dash", cmd: "53 15",
      title: { en: "Dashboard telemetry", tr: "Gösterge telemetrisi" },
      body: { en: "Streams game telemetry to wheels with a built-in display.",
              tr: "Oyun telemetrisini ekranlı direksiyonlara gönderir." },
      probe: () => [0x53, 0x15, 0] },
  ];

  // Results of probes done during development on a Pace V987, firmware 70.
  const KNOWN: Record<string, Status> = { dclutch: "unsupported", dash: "unsupported" };

  let status = $state<Record<string, Status>>(loadStatus());
  function loadStatus() {
    try { return { ...KNOWN, ...JSON.parse(localStorage.getItem("labs:" + app.name) ?? "{}") }; } catch { return { ...KNOWN }; }
  }
  function remember(id: string, s: Status) {
    status[id] = s;
    try { localStorage.setItem("labs:" + app.name, JSON.stringify(status)); } catch {}
  }

  async function probe(f: Feature, bytes: number[]) {
    const r = await api.raw(bytes);
    const ok = !!r && parseInt(r.slice(0, 2), 16) === (bytes[0] | 0x80) && parseInt(r.slice(3, 5), 16) === bytes[1];
    remember(f.id, ok ? "supported" : "unsupported");
  }

  // Feature-specific controls.
  let calibStep = $state<0 | 1>(0);
  let road = $state(50);
  let led = $state("#ff3030");

  async function calibrate() {
    const f = FEATURES[0];
    if (calibStep === 0) {
      const r = await api.raw([0x53, 0x0f, 1]);
      if (!r) return remember(f.id, "unsupported");
      remember(f.id, "supported");
      calibStep = 1;
    } else {
      await api.raw([0x53, 0x0f, 2]);
      calibStep = 0;
    }
  }
  function ledBytes(sub: number) {
    const n = parseInt(led.slice(1), 16), rgb = [n >> 16, (n >> 8) & 255, n & 255];
    return [0x53, sub, mode(), ...Array(5).fill(rgb).flat()];
  }

  // Raw console
  let rawHex = $state("50 01");
  let rawOut = $state<string | null | undefined>(undefined);
  async function sendRaw() {
    const bytes = rawHex.trim().split(/[\s,]+/).map((x) => parseInt(x, 16)).filter((x) => !isNaN(x) && x >= 0 && x <= 255);
    if (bytes.length) rawOut = await api.raw(bytes);
  }
  const L = (o: { en: string; tr: string }) => o[i18n.lang];
</script>

<section>
  <header class="intro"><p>{t("labsIntro")}</p></header>

  <div class="grid">
    {#each FEATURES as f (f.id)}
      {@const s = status[f.id] ?? "unknown"}
      <div class="card {s}">
        <div class="h">
          <div><div class="name">{L(f.title)}</div><code class="cmd">{f.cmd}</code></div>
          <span class="badge {s}">{t(s)}</span>
        </div>
        <p>{L(f.body)}</p>
        <div class="act">
          {#if f.id === "calib"}
            <button class="primary sm" onclick={calibrate}>{calibStep === 0 ? (i18n.lang === "tr" ? "Kalibre et" : "Calibrate") : (i18n.lang === "tr" ? "Düz tutuyorum, bitir" : "Held straight: finish")}</button>
          {:else if f.id === "road"}
            <input type="range" min="0" max="100" step="10" bind:value={road} style:--p="{road}%" />
            <b class="num">{road}</b>
            <button class="ghost sm" onclick={async () => { const r = await api.raw([0x53, 0x10, mode(), road]); remember(f.id, r ? "supported" : "unsupported"); }}>{t("send")}</button>
          {:else if f.id === "led"}
            <input type="color" bind:value={led} />
            <button class="ghost sm" onclick={async () => { const a = await api.raw(ledBytes(0x0b)); const b = await api.raw(ledBytes(0x0c)); remember(f.id, a || b ? "supported" : "unsupported"); }}>{t("send")}</button>
          {:else if f.id === "fan"}
            {#each [0, 1, 2, 3] as lv}<button class="ghost sm" onclick={async () => { const r = await api.raw([0x53, 0x11, lv]); remember(f.id, r ? "supported" : "unsupported"); }}>{lv}</button>{/each}
          {:else if f.probe && s === "unknown"}
            <button class="ghost sm" onclick={() => probe(f, f.probe!())}>{t("check")}</button>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  <div class="card raw">
    <div class="name">{t("rawTitle")}</div>
    <p>{t("rawHint")}</p>
    <div class="rawrow">
      <input class="mono" bind:value={rawHex} onkeydown={(e) => e.key === "Enter" && sendRaw()} />
      <button class="primary sm" onclick={sendRaw}>{t("send")}</button>
    </div>
    {#if rawOut !== undefined}<code class="out">{rawOut ?? t("noReply")}</code>{/if}
  </div>
</section>

<style>
  .intro p { margin: 0 0 16px; color: var(--text-dim); max-width: 85ch; line-height: 1.5; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 12px; }
  .card { background: var(--surface-2); border: 1px solid var(--line); border-radius: var(--r-lg); padding: 16px; display: flex; flex-direction: column; gap: 8px; }
  .card.unsupported { opacity: 0.6; }
  .card p { margin: 0; color: var(--text-dim); font-size: 13.5px; line-height: 1.5; flex: 1; }
  .h { display: flex; justify-content: space-between; gap: 10px; align-items: flex-start; }
  .name { font-weight: 600; }
  .cmd { font-size: 11px; color: var(--text-faint); }
  .badge { font-size: 11.5px; font-weight: 600; padding: 4px 9px; border-radius: 999px; white-space: nowrap; }
  .badge.supported { color: var(--ok); background: var(--ok-soft); }
  .badge.unsupported { color: var(--text-faint); background: var(--surface-3); }
  .badge.unknown { color: var(--warn); background: var(--warn-soft); }
  .act { display: flex; gap: 8px; align-items: center; min-height: 32px; }
  .act input[type="range"] { flex: 1; }
  .num { min-width: 30px; font-variant-numeric: tabular-nums; }
  input[type="color"] { width: 44px; height: 30px; padding: 0; border: 0; background: none; cursor: pointer; }
  .raw { margin-top: 14px; max-width: 720px; }
  .rawrow { display: flex; gap: 8px; }
  .rawrow input { flex: 1; }
  .mono, .out { font-family: ui-monospace, "JetBrains Mono", monospace; }
  .out { font-size: 12.5px; background: var(--surface-1); padding: 8px 10px; border-radius: var(--r-sm); word-break: break-all; }
</style>
