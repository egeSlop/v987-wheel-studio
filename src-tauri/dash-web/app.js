// Wheel Dash: tablet page of V987 Wheel Studio. Plain JS, no build step.
// Talks to Wheel Studio over ws://<pc>:47821/ws. Protocol in docs/DASH.md.
"use strict";

(function () {
  var $ = function (id) { return document.getElementById(id); };

  // ------------------------------------------------------------ text
  var TEXT = {
    en: {
      start: "Start", startBody: "Keeps the screen on and fills it. Wheel Studio on your PC sends the game's data here, and the buttons press keys in the game.",
      startFs: "Full screen", gear: "Gear", fuel: "Fuel", locked: "Locked",
      waiting: "Waiting for a game", waitingBody: "Start a supported game: Farming Simulator 22, Euro Truck Simulator 2, American Truck Simulator, Assetto Corsa or DiRT Rally 2.0. Mods can add more.",
      live: "Live", paused: "Paused", noGame: "No game", connecting: "Connecting to Wheel Studio…", reconnecting: "Lost Wheel Studio. Reconnecting…",
      pinTitle: "Pair this tablet", pinBody: "Enter the PIN shown in Wheel Studio on the Tablet page, or scan its QR code.", wrongPin: "Wrong PIN. Try again.",
      later: "Not now", settings: "Settings", profile: "Profile", language: "Language", units: "Speed units", size: "Button size",
      sizeM: "Normal", sizeL: "Large", sizeXL: "Extra large", layout: "Show", layoutBoth: "Radio and buttons", layoutButtons: "Buttons only", layoutRadio: "Radio and gauges only",
      night: "Night mode (dimmer)", haptics: "Vibrate on press", awake: "Keep the screen on", pair: "Enter PIN", unpair: "Forget pairing", reload: "Reload page",
      auto: "Auto (follow the game)", notPaired: "Pair first: tap here to enter the PIN.", keyFail: "Wheel Studio couldn't press the key: ", noKey: "This button has no key: ",
      gameRadio: "Game radio", tabletRadio: "Tablet radio", radioOff: "Off", radioOn: "On", buffering: "Tuning…", playing: "Playing", cantPlay: "Can't play this station",
      tapPlay: "Tap ⏻ to play", stations: "stations", awakeOn: "The screen stays on", awakeOff: "The screen may turn off", fsNo: "This browser can't go full screen. Add the page to your home screen instead.",
      paired: "Paired. The buttons work now.", on: "On", off: "Off", version: "Version", pc: "PC", followGame: "follows the game: the tablet can't hear it, it just keeps count",
      blinkL: "Left", blinkR: "Right", hazard: "Hazards", beacon: "Beacon", lights: "Lights", high: "High beam", park: "Parking brake", cruise: "Cruise", engine: "Engine",
    },
    tr: {
      start: "Başlat", startBody: "Ekranı açık tutar ve tam ekrana geçer. PC'deki Wheel Studio oyunun verilerini buraya yollar, tuşlar da oyunda tuşlara basar.",
      startFs: "Tam ekran", gear: "Vites", fuel: "Yakıt", locked: "Kilitli",
      waiting: "Oyun bekleniyor", waitingBody: "Desteklenen bir oyun açın: Farming Simulator 22, Euro Truck Simulator 2, American Truck Simulator, Assetto Corsa ya da DiRT Rally 2.0. Modlarla başka oyunlar da eklenebilir.",
      live: "Canlı", paused: "Duraklatıldı", noGame: "Oyun yok", connecting: "Wheel Studio'ya bağlanılıyor…", reconnecting: "Wheel Studio bağlantısı koptu. Yeniden bağlanılıyor…",
      pinTitle: "Tableti eşleştir", pinBody: "Wheel Studio'nun Tablet sayfasındaki PIN'i girin ya da oradaki QR kodu okutun.", wrongPin: "PIN yanlış. Tekrar deneyin.",
      later: "Şimdi değil", settings: "Ayarlar", profile: "Profil", language: "Dil", units: "Hız birimi", size: "Tuş boyutu",
      sizeM: "Normal", sizeL: "Büyük", sizeXL: "Çok büyük", layout: "Göster", layoutBoth: "Radyo ve tuşlar", layoutButtons: "Sadece tuşlar", layoutRadio: "Sadece radyo ve göstergeler",
      night: "Gece modu (daha karanlık)", haptics: "Basınca titret", awake: "Ekranı açık tut", pair: "PIN gir", unpair: "Eşleştirmeyi unut", reload: "Sayfayı yenile",
      auto: "Otomatik (oyunu takip et)", notPaired: "Önce eşleştirin: PIN girmek için buraya dokunun.", keyFail: "Wheel Studio tuşa basamadı: ", noKey: "Bu tuşa bir klavye tuşu atanmamış: ",
      gameRadio: "Oyun radyosu", tabletRadio: "Tablet radyosu", radioOff: "Kapalı", radioOn: "Açık", buffering: "Ayarlanıyor…", playing: "Çalıyor", cantPlay: "Bu istasyon çalınamıyor",
      tapPlay: "Çalmak için ⏻", stations: "istasyon", awakeOn: "Ekran açık kalacak", awakeOff: "Ekran kapanabilir", fsNo: "Bu tarayıcı tam ekrana geçemiyor. Sayfayı ana ekrana ekleyin.",
      paired: "Eşleşti. Tuşlar artık çalışıyor.", on: "Açık", off: "Kapalı", version: "Sürüm", pc: "PC", followGame: "oyunu takip ediyor: tablet oyunun sesini duymaz, sadece sayar",
      blinkL: "Sol", blinkR: "Sağ", hazard: "Dörtlü", beacon: "Tepe lambası", lights: "Farlar", high: "Uzun far", park: "El freni", cruise: "Sabitleyici", engine: "Motor",
    },
  };

  // ------------------------------------------------------------ settings (per tablet)
  var store = {
    get: function (k, d) { try { var v = localStorage.getItem("wd." + k); return v === null ? d : JSON.parse(v); } catch (e) { return d; } },
    set: function (k, v) { try { localStorage.setItem("wd." + k, JSON.stringify(v)); } catch (e) {} },
    del: function (k) { try { localStorage.removeItem("wd." + k); } catch (e) {} },
  };
  var prefs = {
    lang: store.get("lang", "auto"), units: store.get("units", "auto"), size: store.get("size", "m"), layout: store.get("layout", "both"),
    night: store.get("night", false), haptics: store.get("haptics", true), awake: store.get("awake", true), profile: store.get("profile", "auto"),
  };

  function lang() {
    if (prefs.lang !== "auto") return prefs.lang;
    return (navigator.language || "en").toLowerCase().indexOf("tr") === 0 ? "tr" : "en";
  }
  function t(k) { return (TEXT[lang()] && TEXT[lang()][k]) || TEXT.en[k] || k; }
  function loc(obj, fallback) {
    if (obj && obj[lang()]) return obj[lang()];
    return fallback || "";
  }
  function applyText() {
    document.documentElement.lang = lang();
    var els = document.querySelectorAll("[data-t]");
    for (var i = 0; i < els.length; i++) els[i].textContent = t(els[i].getAttribute("data-t"));
  }

  // ------------------------------------------------------------ icons
  var ICONS = {
    engine: "M5 10h3l2-3h5l1 2h2v3h2v-2h1v6h-1v-2h-2v3h-3l-2 2H9l-1-2H5z",
    lights: "M10 6C6 6 4 9 4 12s2 6 6 6c2 0 3-2 3-6s-1-6-3-6zM16 8h5M16 12h5M16 16h5",
    highbeam: "M10 6C6 6 4 9 4 12s2 6 6 6c2 0 3-2 3-6s-1-6-3-6zM16 7l5-1M16 12h5M16 17l5 1",
    beacon: "M7 20h10M8 20v-6a4 4 0 0 1 8 0v6M12 4v2M5 7l1.5 1.5M19 7l-1.5 1.5M3 13h2M19 13h2",
    left: "M10 5 3 12l7 7v-4h8V9h-8z",
    right: "M14 5l7 7-7 7v-4H6V9h8z",
    hazard: "M12 4 3 19h18zM12 9.5 8 16.5h8z",
    horn: "M3 10v4h3l7 4V6L6 10zM16 9a4 4 0 0 1 0 6M18.5 6.5a7.5 7.5 0 0 1 0 11",
    cruise: "M4 16a8 8 0 1 1 16 0M12 16l4-5M8 20h8",
    worklight: "M4 9h6v6H4zM10 10l3-2v8l-3-2M15 8l5-2M15 12h5M15 16l5 2",
    camera: "M4 8h3l2-3h6l2 3h3v11H4zM12 10a3.5 3.5 0 1 0 0 7 3.5 3.5 0 0 0 0-7z",
    robot: "M6 9h12v10H6zM12 5v4M9 13h.01M15 13h.01M9 16h6M3 13v3M21 13v3",
    swap: "M4 8h14l-3-3M20 16H6l3 3",
    door: "M6 3h10v18H6zM16 5l3 1v13l-3 1M12 12h.01",
    attach: "M4 14h6M10 11h4v6h-4zM14 14h6M17 11v6",
    detach: "M4 14h5M15 14h5M9 11v6M15 11v6M11 9l2-2M11 19l2 2",
    select: "M4 6h16M4 12h10M4 18h16M18 10l2 2-2 2",
    lower: "M12 4v12M7 11l5 5 5-5M5 20h14",
    power: "M12 3v8M6.3 7a8 8 0 1 0 11.4 0",
    fold: "M4 18h16M6 18V8l6 4 6-4v10",
    tip: "M3 15h12l3-6h3M5 15l2-7h8M7 19h.01M15 19h.01",
    pipe: "M4 18V9a3 3 0 0 1 3-3h10M17 3v6M20 6h-3",
    cover: "M3 14h18M5 14c0-4 3-7 7-7s7 3 7 7M3 18h18",
    seed: "M12 20c-4-2-6-6-6-11 4 0 7 2 6 6M12 20c4-2 6-6 6-11-4 0-7 2-6 6M12 20v-6",
    map: "M3 6l6-2 6 2 6-2v14l-6 2-6-2-6 2zM9 4v14M15 6v14",
    shop: "M4 9h16l-1 11H5zM8 9V7a4 4 0 0 1 8 0v2",
    menu: "M4 7h16M4 12h16M4 17h16",
    help: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM9.5 9.5a2.5 2.5 0 1 1 3.5 2.3c-.7.3-1 .9-1 1.7M12 17h.01",
    slower: "M19 5v14l-8-7zM11 5v14l-8-7z",
    faster: "M5 5v14l8-7zM13 5v14l8-7z",
    pause: "M8 5v14M16 5v14",
    parkbrake: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM10 16V8h3a2.5 2.5 0 0 1 0 5h-3",
    wipers: "M4 18a10 10 0 0 1 16 0M12 18 7 9",
    lock: "M7 11V8a5 5 0 0 1 10 0v3M5 11h14v10H5zM12 15v2",
    save: "M5 4h11l3 3v13H5zM8 4v5h8V4M8 20v-6h8v6",
    abs: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 7a5 5 0 1 0 0 10 5 5 0 0 0 0-10z",
    tc: "M6 19c3-2 3-12 6-14 3 2 3 12 6 14M8 13h8",
    lookback: "M3 12s3-6 9-6 9 6 9 6-3 6-9 6-9-6-9-6zM12 9l-3 3 3 3M9 12h6",
    plus: "M12 5v14M5 12h14",
    minus: "M5 12h14",
    mirror: "M4 7h11a3 3 0 0 1 3 3v2a3 3 0 0 1-3 3H4zM18 11h3",
    hud: "M3 5h18v12H3zM7 9h4M7 13h10M15 9h2",
    display: "M4 4h16v12H4zM9 20h6M12 16v4",
    parking: "M6 3h12v18H6zM10 16V8h3a2.5 2.5 0 0 1 0 5h-3",
  };
  function icon(name) {
    var d = ICONS[name];
    return d ? '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="' + d + '"/></svg>' : "";
  }
  function esc(s) { return String(s == null ? "" : s).replace(/[&<>"']/g, function (c) { return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]; }); }

  // ------------------------------------------------------------ state
  var S = {
    ws: null, connected: false, everConnected: false, retry: 0,
    hello: null, profiles: {}, paired: false, pinRequired: true,
    game: null, profileId: "generic", page: store.get("page", {}),
    radio: { mode: {}, idx: store.get("radioIdx", {}), on: {}, vol: store.get("vol", 0.8), playing: false, status: "" },
    reqId: 0, holds: {},
  };

  // ------------------------------------------------------------ connection
  function wsUrl() {
    var proto = location.protocol === "https:" ? "wss://" : "ws://";
    return proto + location.host + "/ws";
  }
  function connect() {
    try { S.ws = new WebSocket(wsUrl()); } catch (e) { scheduleReconnect(); return; }
    S.ws.onopen = function () { S.connected = true; S.everConnected = true; S.retry = 0; showConn(false); };
    S.ws.onclose = function () {
      S.connected = false;
      releaseAllHolds();
      showConn(true);
      scheduleReconnect();
    };
    S.ws.onerror = function () {};
    S.ws.onmessage = function (e) {
      var m; try { m = JSON.parse(e.data); } catch (x) { return; }
      if (m.type === "hello") onHello(m);
      else if (m.type === "state") onState(m.game);
      else if (m.type === "auth") onAuth(m);
      else if (m.type === "pressed") onPressed(m);
    };
  }
  function scheduleReconnect() {
    S.retry = Math.min(S.retry + 1, 6);
    setTimeout(connect, [500, 1000, 1500, 2500, 4000, 5000][S.retry - 1] || 5000);
  }
  function send(o) {
    if (S.ws && S.ws.readyState === 1) { S.ws.send(JSON.stringify(o)); return true; }
    return false;
  }
  function showConn(on) {
    $("conn").classList.toggle("hidden", !on);
    $("connText").textContent = S.everConnected ? t("reconnecting") : t("connecting");
  }

  function onHello(m) {
    S.hello = m;
    S.profiles = {};
    (m.profiles || []).forEach(function (p) { S.profiles[p.id] = p; });
    S.pinRequired = m.pinRequired;
    S.paired = !!m.paired;
    var pin = store.get("pin", "");
    if (!S.paired && pin) send({ type: "auth", pin: pin });
    updateLock();
    fillProfileSelect();
    renderProfile(true);
    renderAbout();
  }
  function onAuth(m) {
    if (m.ok) {
      var wasPaired = S.paired;
      S.paired = true;
      $("pinBox").classList.add("hidden");
      if (!wasPaired && pinTyped) toast(t("paired"));
      pinTyped = false;
    } else {
      S.paired = false;
      if (m.reason === "wrong-pin") {
        store.del("pin");
        if (pinTyped) { $("pinErr").textContent = t("wrongPin"); pinBuf = ""; drawPin(); }
      } else if (m.reason === "pin-changed") {
        store.del("pin");
      }
    }
    updateLock();
  }
  function onPressed(m) {
    if (m.ok) return;
    if (m.error === "not-paired") { toast(t("notPaired"), true); openPin(); return; }
    toast((m.error && m.error.indexOf("no keyboard key") >= 0 ? t("noKey") : t("keyFail")) + (m.error || ""), true);
    var el = document.querySelector('.btn[data-req="' + m.id + '"]');
    if (el) { el.classList.remove("fail"); void el.offsetWidth; el.classList.add("fail"); }
  }
  function updateLock() {
    var locked = S.pinRequired && !S.paired;
    $("lockPill").classList.toggle("hidden", !locked);
    document.body.classList.toggle("locked", locked);
    var b = document.querySelectorAll(".btn");
    for (var i = 0; i < b.length; i++) b[i].classList.toggle("locked", locked);
  }

  // ------------------------------------------------------------ profile + rendering
  function currentProfile() { return S.profiles[S.profileId] || S.profiles.generic || { id: "generic", name: "Game", pages: [], readouts: [], radio: {} }; }
  function pickProfile() {
    var id = prefs.profile !== "auto" && S.profiles[prefs.profile] ? prefs.profile : (S.game && S.game.profile) || store.get("lastProfile", "generic");
    if (!S.profiles[id]) id = "generic";
    return id;
  }

  function renderProfile(force) {
    var id = pickProfile();
    if (!force && id === S.profileId) return;
    S.profileId = id;
    store.set("lastProfile", id);
    var p = currentProfile();
    document.body.setAttribute("data-theme", p.theme || "default");
    document.body.style.setProperty("--accent", p.accent || "#7c8cff");
    renderPages();
    renderChips();
    renderRadio();
  }

  function renderPages() {
    var p = currentProfile();
    var pages = p.page || p.pages || [];
    var cur = Math.min(S.page[p.id] || 0, Math.max(0, pages.length - 1));
    var nav = $("pages");
    nav.innerHTML = "";
    nav.classList.toggle("hidden", pages.length < 2);
    pages.forEach(function (pg, i) {
      var b = document.createElement("button");
      b.textContent = loc(pg.names, pg.name) || "Page " + (i + 1);
      if (i === cur) b.className = "on";
      b.onclick = function () { S.page[p.id] = i; store.set("page", S.page); renderPages(); haptic(8); };
      nav.appendChild(b);
    });
    var grid = $("grid");
    grid.innerHTML = "";
    var pg = pages[cur];
    if (!pg) return;
    (pg.button || pg.buttons || []).forEach(function (b) {
      var el = document.createElement("button");
      el.className = "btn" + (b.wide ? " wide" : "") + (b.hold ? " hold" : "") + (b.tone ? " " + b.tone : "");
      el.setAttribute("data-id", b.id);
      if (b.active) el.setAttribute("data-active", b.active);
      el.innerHTML = icon(b.icon) + "<span>" + esc(loc(b.labels, b.label) || b.id) + "</span>";
      bindButton(el, b);
      grid.appendChild(el);
    });
    updateLock();
    updateButtons();
    fitGrid();
  }

  // Few buttons: let them grow into the free space (up to 1.7x), so the panel never looks empty.
  function fitGrid() {
    var grid = $("grid");
    grid.style.removeProperty("--row-h");
    var n = 0, kids = grid.children;
    for (var i = 0; i < kids.length; i++) n += kids[i].classList.contains("wide") ? 2 : 1;
    if (!n) return;
    var cs = getComputedStyle(document.body);
    var minW = parseFloat(cs.getPropertyValue("--btn-min")) || 118, baseH = parseFloat(cs.getPropertyValue("--btn-h")) || 88;
    var cols = Math.max(1, Math.floor((grid.clientWidth + 8) / (minW + 8)));
    var rows = Math.ceil(n / cols);
    var avail = grid.clientHeight;
    if (!avail) return;
    var h = Math.floor((avail - (rows - 1) * 8) / rows);
    if (h > baseH) grid.style.setProperty("--row-h", Math.min(h, baseH * 1.7) + "px");
  }
  window.addEventListener("resize", function () { setTimeout(fitGrid, 50); });(el, b) {
    var start = null, moved = false, pid = null;
    el.addEventListener("pointerdown", function (e) {
      pid = e.pointerId; start = { x: e.clientX, y: e.clientY, t: Date.now() }; moved = false;
      el.classList.add("press");
      if (b.hold) { press(b, true, el); try { el.setPointerCapture(e.pointerId); } catch (x) {} }
    });
    el.addEventListener("pointermove", function (e) {
      if (!start || e.pointerId !== pid || b.hold) return;
      if (Math.abs(e.clientX - start.x) > 12 || Math.abs(e.clientY - start.y) > 12) { moved = true; el.classList.remove("press"); }
    });
    var end = function (e, cancel) {
      if (!start) return;
      el.classList.remove("press");
      if (b.hold) press(b, false, el);
      else if (!cancel && !moved) press(b, null, el);
      start = null;
    };
    el.addEventListener("pointerup", function (e) { end(e, false); });
    el.addEventListener("pointercancel", function (e) { end(e, true); });
    el.addEventListener("contextmenu", function (e) { e.preventDefault(); });
  }

  function press(b, down, el) {
    if (S.pinRequired && !S.paired) { if (down !== false) { toast(t("notPaired"), true); openPin(); } return; }
    var id = ++S.reqId;
    if (el) el.setAttribute("data-req", id);
    var msg = { type: "press", id: id, profile: S.profileId, button: b.id };
    if (down !== null) msg.down = down;
    if (down === true) S.holds[b.id] = true;
    if (down === false) delete S.holds[b.id];
    if (down !== false) haptic(14);
    send(msg);
  }
  function releaseAllHolds() { S.holds = {}; var b = document.querySelectorAll(".btn.press"); for (var i = 0; i < b.length; i++) b[i].classList.remove("press"); }

  function renderChips() {
    var p = currentProfile();
    var box = $("chips");
    box.innerHTML = "";
    (p.readout || p.readouts || []).forEach(function (r) {
      var c = document.createElement("div");
      c.className = "chip";
      c.setAttribute("data-field", r.field);
      c.innerHTML = "<small>" + esc(loc(r.labels, r.label) || r.field) + "</small><b>–</b>";
      box.appendChild(c);
    });
  }

  // ------------------------------------------------------------ live data
  function num(v) { return typeof v === "number" && isFinite(v) ? v : null; }
  function mph() { return prefs.units === "mph" || (prefs.units === "auto" && S.profileId === "ats"); }
  function lapTime(s) {
    if (s == null || s <= 0) return "–";
    var m = Math.floor(s / 60), r = s - m * 60;
    return m + ":" + (r < 10 ? "0" : "") + r.toFixed(3);
  }
  function fmt(r, v) {
    if (v == null) return "–";
    if (typeof v === "boolean") return v ? t("on") : t("off");
    if (typeof v === "string") return esc(v);
    if (r.format === "laptime") return lapTime(v);
    if (r.format === "text") return esc(v);
    var x = v * (r.scale || 1);
    var unit = r.unit || "";
    if (unit === "km/h" && mph()) { x = x / 1.609344; unit = "mph"; }
    if (unit === "km" && mph()) { x = x / 1.609344; unit = "mi"; }
    if ((r.field === "cruise" || r.field === "speedLimit") && x <= 0.5) return "–";
    var s = x.toFixed(r.decimals || 0);
    return s + (unit ? "<em>" + esc(unit) + "</em>" : "");
  }

  function onState(g) {
    S.game = g;
    renderProfile(false);
    var chip = $("gameChip");
    chip.classList.toggle("live", !!(g && g.active));
    chip.classList.toggle("idle", !!(g && !g.active));
    $("gameName").textContent = g ? g.name : (currentProfile().id !== "generic" ? currentProfile().name : t("noGame"));
    $("gameState").textContent = g ? (g.active ? t("live") : t("paused")) : "";
    var d = (g && g.data) || {};
    var waiting = !g;
    $("cluster").classList.toggle("wait", waiting);
    $("waiting").classList.toggle("hidden", !waiting);
    if (waiting) {
      var hint = t("waitingBody");
      var p = currentProfile();
      if (p.setup && loc(p.setup, p.setup.en) && prefs.profile !== "auto") hint = loc(p.setup, p.setup.en);
      $("waitingHint").textContent = hint;
    }

    var sp = num(d.speedKmh);
    if (sp == null && num(d.speed) != null) sp = Math.abs(d.speed) * 3.6;
    var useMph = mph();
    $("speed").textContent = sp == null ? "0" : Math.round(useMph ? sp / 1.609344 : sp);
    $("speedUnit").textContent = useMph ? "mph" : "km/h";
    var gear = d.gear == null ? "–" : String(d.gear);
    $("gear").textContent = gear;
    $("gearBox").classList.toggle("rev", /^R/.test(gear));
    $("gearBox").querySelector("small").textContent = t("gear") + (d.gearGroup ? " · " + d.gearGroup : "");

    var rpm = num(d.rpm), max = num(d.maxRpm);
    // Games that don't report the rev limit: scale to the highest rpm seen in this car.
    var vkey = (g && g.game) + "|" + (d.vehicle || "");
    if (S.rpmKey !== vkey) { S.rpmKey = vkey; S.rpmSeen = 0; }
    if (rpm != null) S.rpmSeen = Math.max(S.rpmSeen || 0, rpm);
    if (!max && rpm != null) max = Math.max(3000, S.rpmSeen * 1.04);
    $("rpmRow").classList.toggle("hidden", rpm == null);
    if (rpm != null) {
      var frac = max ? Math.min(1, rpm / max) : Math.min(1, rpm / 8000);
      $("rpmBar").style.width = (frac * 100).toFixed(1) + "%";
      $("rpmBar").parentNode.classList.toggle("hot", frac > 0.9);
      $("rpmVal").textContent = Math.round(rpm);
    }
    var fuel = num(d.fuel);
    $("fuelRow").classList.toggle("hidden", fuel == null);
    if (fuel != null) {
      $("fuelBar").style.width = (Math.max(0, Math.min(1, fuel)) * 100).toFixed(1) + "%";
      $("fuelBar").parentNode.classList.toggle("low", fuel < 0.15);
      $("fuelVal").textContent = Math.round(fuel * 100) + "%";
    }
    var chips = document.querySelectorAll(".chip");
    var p2 = currentProfile();
    var rs = p2.readout || p2.readouts || [];
    for (var i = 0; i < chips.length; i++) {
      var f = chips[i].getAttribute("data-field");
      var r = rs.filter(function (x) { return x.field === f; })[0] || {};
      chips[i].querySelector("b").innerHTML = fmt(r, d[f]);
    }
    renderTells(d);
    updateButtons();
    renderRadio();
  }

  function renderTells(d) {
    var list = [
      ["blinkerLeft", "left", "blinkL", "ok blink"], ["hazard", "hazard", "hazard", "danger blink"], ["blinkerRight", "right", "blinkR", "ok blink"],
      ["lightsLow", "lights", "lights", "ok"], ["lightsHigh", "highbeam", "high", "ok"], ["beacon", "beacon", "beacon", "warn blink"],
      ["parkingBrake", "parkbrake", "park", "danger"], ["cruiseOn", "cruise", "cruise", "ok"],
    ];
    var html = "";
    list.forEach(function (x) {
      if (d[x[0]] === true) html += '<span class="tell ' + x[3] + '">' + icon(x[1]) + t(x[2]) + "</span>";
    });
    if (d.engine === false && S.game && S.game.active) html += '<span class="tell warn">' + icon("engine") + t("engine") + " · " + t("off") + "</span>";
    $("tells").innerHTML = html;
  }

  function updateButtons() {
    var d = (S.game && S.game.data) || {};
    var b = document.querySelectorAll(".btn[data-active]");
    for (var i = 0; i < b.length; i++) {
      var f = b[i].getAttribute("data-active");
      var on = d[f] === true || (typeof d[f] === "number" && d[f] > 0);
      b[i].classList.toggle("on", on);
      b[i].classList.toggle("blink", on && (f === "blinkerLeft" || f === "blinkerRight" || f === "hazard"));
    }
  }

  // ------------------------------------------------------------ radio
  var audio = $("radio");
  function radioMode() {
    var p = currentProfile();
    var r = p.radio || {};
    var hasGame = !!(r.next || r.power);
    var m = S.radio.mode[p.id] || store.get("radioMode." + p.id, null) || r.mode || "tablet";
    if (m === "game" && !hasGame) m = "tablet";
    return m;
  }
  function stations() {
    var p = currentProfile();
    if (radioMode() === "game") {
      var src = (p.radio || {}).stations;
      return (src && S.hello && S.hello.gameStations && S.hello.gameStations[src]) || [];
    }
    return (S.hello && S.hello.stations) || [];
  }
  function freqOf(name, i) {
    // A made-up but stable FM frequency for the display.
    var h = 0;
    for (var k = 0; k < name.length; k++) h = (h * 31 + name.charCodeAt(k)) >>> 0;
    var f = 87.6 + ((h % 200) / 10);
    return f.toFixed(1);
  }
  function radioKey() { return S.profileId + ":" + radioMode(); }

  function renderRadio() {
    var p = currentProfile();
    var mode = radioMode();
    var list = stations();
    var key = radioKey();
    var idx = S.radio.idx[key] || 0;
    if (list.length) idx = ((idx % list.length) + list.length) % list.length;
    var st = list[idx];
    var on = mode === "game" ? S.radio.on[key] !== false : S.radio.playing;
    $("lcd").classList.toggle("off", !on && mode === "tablet" && !S.radio.status);
    $("band").textContent = mode === "game" ? "GAME" : "NET";
    $("srcLabel").textContent = mode === "game" ? "GAME" : "TAB";
    $("srcBtn").classList.toggle("game", mode === "game");
    var r = p.radio || {};
    $("srcBtn").classList.toggle("dis", !(r.next || r.power));
    $("station").textContent = st ? st.name : (mode === "game" ? t("gameRadio") : t("tabletRadio"));
    $("freq").textContent = st ? "FM " + freqOf(st.name, idx) : "";
    var state;
    if (mode === "game") state = (S.radio.on[key] === false ? t("radioOff") : t("radioOn"));
    else state = S.radio.status || (S.radio.playing ? t("playing") : t("tapPlay"));
    $("radioState").textContent = state;
    $("volView").textContent = mode === "tablet" ? "VOL " + Math.round(S.radio.vol * 30) : "";
    var g = S.game;
    var bits = [];
    if (g && g.data && g.data.vehicle) bits.push(g.data.vehicle);
    if (g) bits.push(g.name);
    if (mode === "game" && st) bits.push(t("followGame"));
    if (list.length) bits.push(list.length + " " + t("stations"));
    var rds = bits.join("  ·  ") || "Wheel Dash";
    if ($("rdsText").textContent !== rds) $("rdsText").textContent = rds;
    $("rds").classList.toggle("still", rds.length < 34);
    var gameKeys = { power: r.power, next: r.next, prev: r.prev, volUp: r.volUp || r.vol_up, volDown: r.volDown || r.vol_down };
    var hk = document.querySelectorAll(".hk[data-radio]");
    for (var i = 0; i < hk.length; i++) {
      var w = hk[i].getAttribute("data-radio");
      hk[i].classList.toggle("dis", mode === "game" && !gameKeys[w]);
    }
    // Presets: first six stations.
    var pre = $("presets");
    var want = list.slice(0, 6).map(function (s) { return s.name; }).join("|") + "#" + idx;
    if (pre.getAttribute("data-k") !== want) {
      pre.setAttribute("data-k", want);
      pre.innerHTML = "";
      list.slice(0, 6).forEach(function (s, i) {
        var b = document.createElement("button");
        b.textContent = (i + 1) + " " + s.name;
        if (i === idx) b.className = "on";
        b.onclick = function () { haptic(10); tune(i - idx); };
        pre.appendChild(b);
      });
      pre.classList.toggle("hidden-presets", list.length === 0 || mode === "game");
    }
  }

  function tune(step) {
    var key = radioKey();
    var list = stations();
    var mode = radioMode();
    var cur = S.radio.idx[key] || 0;
    if (mode === "game") {
      // Step the game's radio one channel at a time; we only follow along.
      var what = step > 0 ? "next" : "prev";
      var n = Math.abs(step);
      for (var i = 0; i < n; i++) setTimeout(function () { radioPress(what); }, i * 260);
      S.radio.idx[key] = cur + step;
    } else {
      S.radio.idx[key] = cur + step;
      if (S.radio.playing || step !== 0) playStation();
    }
    store.set("radioIdx", S.radio.idx);
    renderRadio();
  }

  function radioPress(what) {
    if (S.pinRequired && !S.paired) { toast(t("notPaired"), true); openPin(); return false; }
    send({ type: "radio", id: ++S.reqId, profile: S.profileId, what: what });
    return true;
  }

  function playStation() {
    var list = stations();
    if (!list.length) return;
    var key = radioKey();
    var idx = (((S.radio.idx[key] || 0) % list.length) + list.length) % list.length;
    var st = list[idx];
    S.radio.status = t("buffering");
    audio.src = st.url;
    audio.volume = S.radio.vol;
    var p = audio.play();
    if (p && p.catch) p.catch(function () { S.radio.status = t("cantPlay"); S.radio.playing = false; renderRadio(); });
    S.radio.playing = true;
    if ("mediaSession" in navigator) {
      try {
        navigator.mediaSession.metadata = new MediaMetadata({ title: st.name, artist: "Wheel Dash", album: currentProfile().name, artwork: [{ src: "icon-256.png", sizes: "256x256", type: "image/png" }] });
        navigator.mediaSession.setActionHandler("nexttrack", function () { tune(1); });
        navigator.mediaSession.setActionHandler("previoustrack", function () { tune(-1); });
        navigator.mediaSession.setActionHandler("pause", function () { stopRadio(); });
        navigator.mediaSession.setActionHandler("play", function () { playStation(); });
      } catch (e) {}
    }
    renderRadio();
  }
  function stopRadio() {
    audio.pause();
    audio.removeAttribute("src");
    try { audio.load(); } catch (e) {}
    S.radio.playing = false;
    S.radio.status = "";
    renderRadio();
  }
  audio.addEventListener("playing", function () { S.radio.status = ""; renderRadio(); });
  audio.addEventListener("waiting", function () { if (S.radio.playing) { S.radio.status = t("buffering"); renderRadio(); } });
  audio.addEventListener("error", function () { if (S.radio.playing) { S.radio.status = t("cantPlay"); S.radio.playing = false; renderRadio(); } });

  function onRadioButton(what) {
    haptic(12);
    var mode = radioMode();
    var key = radioKey();
    if (mode === "game") {
      if (what === "next") return tune(1);
      if (what === "prev") return tune(-1);
      if (radioPress(what) && what === "power") S.radio.on[key] = S.radio.on[key] === false;
      renderRadio();
      return;
    }
    if (what === "power") return S.radio.playing ? stopRadio() : playStation();
    if (what === "next") return tune(1);
    if (what === "prev") return tune(-1);
    if (what === "volUp" || what === "volDown") {
      S.radio.vol = Math.max(0, Math.min(1, S.radio.vol + (what === "volUp" ? 1 / 15 : -1 / 15)));
      audio.volume = S.radio.vol;
      store.set("vol", S.radio.vol);
      renderRadio();
    }
  }
  var hks = document.querySelectorAll(".hk[data-radio]");
  for (var i = 0; i < hks.length; i++) {
    (function (el) {
      el.addEventListener("pointerdown", function () { el.classList.add("press"); });
      el.addEventListener("pointerup", function () { el.classList.remove("press"); onRadioButton(el.getAttribute("data-radio")); });
      el.addEventListener("pointercancel", function () { el.classList.remove("press"); });
    })(hks[i]);
  }
  $("srcBtn").onclick = function () {
    var p = currentProfile();
    var r = p.radio || {};
    if (!(r.next || r.power)) return;
    var m = radioMode() === "game" ? "tablet" : "game";
    S.radio.mode[p.id] = m;
    store.set("radioMode." + p.id, m);
    if (m === "game" && S.radio.playing) stopRadio();
    haptic(10);
    toast(m === "game" ? t("gameRadio") : t("tabletRadio"));
    renderRadio();
  };

  // ------------------------------------------------------------ wake + full screen
  var wake = $("wake");
  var wakeLock = null;
  function startAwake() {
    if (!prefs.awake) return;
    var p = wake.play();
    if (p && p.catch) p.catch(function () {});
    if ("wakeLock" in navigator && window.isSecureContext) {
      navigator.wakeLock.request("screen").then(function (l) { wakeLock = l; }).catch(function () {});
    }
    $("awakeBtn").classList.add("on");
  }
  function stopAwake() {
    wake.pause();
    if (wakeLock) { try { wakeLock.release(); } catch (e) {} wakeLock = null; }
    $("awakeBtn").classList.remove("on");
  }
  document.addEventListener("visibilitychange", function () { if (document.visibilityState === "visible" && prefs.awake && started) startAwake(); });

  function fsElement() { return document.fullscreenElement || document.webkitFullscreenElement; }
  function enterFs() {
    var el = document.documentElement;
    var f = el.requestFullscreen || el.webkitRequestFullscreen;
    if (!f) { toast(t("fsNo")); return; }
    try { var r = f.call(el, { navigationUI: "hide" }); if (r && r.catch) r.catch(function () {}); } catch (e) {}
    try { if (screen.orientation && screen.orientation.lock) screen.orientation.lock("landscape").catch(function () {}); } catch (e) {}
  }
  function exitFs() { var f = document.exitFullscreen || document.webkitExitFullscreen; if (f) f.call(document); }
  function syncFsBtn() { $("fsBtn").classList.toggle("on", !!fsElement()); }
  document.addEventListener("fullscreenchange", syncFsBtn);
  document.addEventListener("webkitfullscreenchange", syncFsBtn);
  $("fsBtn").onclick = function () { fsElement() ? exitFs() : enterFs(); };
  $("awakeBtn").onclick = function () {
    prefs.awake = !prefs.awake; store.set("awake", prefs.awake);
    prefs.awake ? startAwake() : stopAwake();
    $("setAwake").checked = prefs.awake;
    toast(prefs.awake ? t("awakeOn") : t("awakeOff"));
  };

  var started = false;
  $("startBtn").onclick = function () {
    started = true;
    store.set("fs", $("startFs").checked);
    if ($("startFs").checked) enterFs();
    startAwake();
    // Unlock audio on iOS: play/pause once inside the tap.
    try { audio.muted = true; var p = audio.play(); if (p && p.catch) p.catch(function () {}); audio.pause(); audio.muted = false; } catch (e) {}
    $("start").classList.add("hidden");
    if (S.pinRequired && !S.paired && !store.get("pin", "")) openPin();
  };
  $("startFs").checked = store.get("fs", true);

  // ------------------------------------------------------------ PIN
  var pinBuf = "", pinTyped = false;
  function drawPin() {
    var h = "";
    for (var i = 0; i < 6; i++) h += '<i class="' + (i < pinBuf.length ? "f" : "") + '"></i>';
    $("pinDots").innerHTML = h;
  }
  function openPin() { pinBuf = ""; $("pinErr").textContent = ""; drawPin(); $("pinBox").classList.remove("hidden"); }
  (function buildPad() {
    var keys = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "", "0", "⌫"];
    var pad = $("pinpad");
    keys.forEach(function (k) {
      var b = document.createElement("button");
      b.textContent = k;
      if (!k) b.style.visibility = "hidden";
      b.onclick = function () {
        haptic(8);
        $("pinErr").textContent = "";
        if (k === "⌫") pinBuf = pinBuf.slice(0, -1);
        else if (pinBuf.length < 6) pinBuf += k;
        drawPin();
        if (pinBuf.length === 6) {
          store.set("pin", pinBuf);
          pinTyped = true;
          send({ type: "auth", pin: pinBuf });
        }
      };
      pad.appendChild(b);
    });
  })();
  $("pinCancel").onclick = function () { $("pinBox").classList.add("hidden"); };
  $("lockPill").onclick = openPin;

  // ------------------------------------------------------------ settings
  function fillProfileSelect() {
    var sel = $("setProfile");
    sel.innerHTML = "";
    var o = document.createElement("option"); o.value = "auto"; o.textContent = t("auto"); sel.appendChild(o);
    Object.keys(S.profiles).forEach(function (id) {
      var op = document.createElement("option"); op.value = id; op.textContent = S.profiles[id].name; sel.appendChild(op);
    });
    sel.value = S.profiles[prefs.profile] ? prefs.profile : "auto";
  }
  function renderAbout() {
    var h = S.hello || {};
    $("about").innerHTML = esc(h.app || "V987 Wheel Studio") + " · " + t("version") + " " + esc(h.version || "?") + "<br>" + t("pc") + ": " + esc(location.host);
  }
  function applyPrefs() {
    document.body.setAttribute("data-size", prefs.size);
    document.body.classList.toggle("night", !!prefs.night);
    document.body.classList.remove("layout-both", "layout-buttons", "layout-radio");
    document.body.classList.add("layout-" + prefs.layout);
    $("setLang").value = prefs.lang; $("setUnits").value = prefs.units; $("setSize").value = prefs.size; $("setLayout").value = prefs.layout;
    $("setNight").checked = !!prefs.night; $("setHaptics").checked = !!prefs.haptics; $("setAwake").checked = !!prefs.awake;
  }
  function bindPref(id, key, isCheck, after) {
    $(id).onchange = function () {
      prefs[key] = isCheck ? $(id).checked : $(id).value;
      store.set(key, prefs[key]);
      applyPrefs();
      if (after) after();
    };
  }
  bindPref("setLang", "lang", false, function () { applyText(); fillProfileSelect(); renderProfile(true); onState(S.game); renderAbout(); });
  bindPref("setUnits", "units", false, function () { onState(S.game); });
  bindPref("setSize", "size", false, fitGrid);
  bindPref("setLayout", "layout", false, fitGrid);
  bindPref("setNight", "night", true);
  bindPref("setHaptics", "haptics", true);
  bindPref("setAwake", "awake", true, function () { prefs.awake ? startAwake() : stopAwake(); });
  bindPref("setProfile", "profile", false, function () { renderProfile(true); onState(S.game); });
  $("setBtn").onclick = function () { $("settings").classList.remove("hidden"); };
  $("gameChip").onclick = function () { $("settings").classList.remove("hidden"); };
  $("setClose").onclick = function () { $("settings").classList.add("hidden"); };
  $("settings").addEventListener("pointerdown", function (e) { if (e.target === $("settings")) $("settings").classList.add("hidden"); });
  $("pairBtn").onclick = function () { $("settings").classList.add("hidden"); openPin(); };
  $("unpairBtn").onclick = function () { store.del("pin"); S.paired = !S.pinRequired; updateLock(); location.reload(); };
  $("reloadBtn").onclick = function () { location.reload(); };

  // ------------------------------------------------------------ misc
  function haptic(ms) { if (prefs.haptics && navigator.vibrate) { try { navigator.vibrate(ms); } catch (e) {} } }
  var toastTimer = null;
  function toast(msg, err) {
    var el = $("toast");
    el.textContent = msg;
    el.classList.toggle("err", !!err);
    el.classList.add("show");
    clearTimeout(toastTimer);
    toastTimer = setTimeout(function () { el.classList.remove("show"); }, 2600);
  }
  function tick() {
    var d = new Date();
    $("clock").textContent = (d.getHours() < 10 ? "0" : "") + d.getHours() + ":" + (d.getMinutes() < 10 ? "0" : "") + d.getMinutes();
  }
  setInterval(tick, 5000); tick();
  // The game went quiet (Wheel Studio sends state at least once a second).
  setInterval(function () { if (S.connected && S.game && S.game.ageMs > 60000) onState(null); }, 5000);

  // A PIN in the URL (from the QR code) pairs this tablet, then leaves the address bar.
  var m = /pin=(\d{6})/.exec(location.hash || "");
  if (m) { store.set("pin", m[1]); pinTyped = true; history.replaceState(null, "", location.pathname); }

  applyText();
  applyPrefs();
  showConn(true);
  connect();
})();
