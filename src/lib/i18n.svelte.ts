export type Lang = "tr" | "en";

function initialLang(): Lang {
  try {
    const s = localStorage.getItem("lang");
    if (s === "tr" || s === "en") return s;
  } catch {}
  return navigator.language?.toLowerCase().startsWith("tr") ? "tr" : "en";
}

export const i18n = $state({ lang: initialLang() as Lang });

export function setLang(l: Lang) {
  i18n.lang = l;
  try { localStorage.setItem("lang", l); } catch {}
}

type Dict = Record<string, string>;

const en: Dict = {
  appName: "V987 Wheel Studio",
  connect: "Connect", connecting: "Connecting…", disconnect: "Disconnect", scan: "Search", scanning: "Searching…",
  noWheel: "No wheel found. Check that it's switched on and not connected to your phone.",
  scanHint: "The wheel keeps its settings in its own memory, and the app changes them over Bluetooth. Close the vendor's phone app first: the wheel only accepts one connection.",
  disconnected: "Disconnected", reconnect: "Reconnect",
  firmware: "Firmware", mode: "Mode", pedals: "Pedals", shifter: "Shifter", detected: "detected", notDetected: "not detected",
  tabTuning: "Force feedback", tabLive: "Live test", tabButtons: "Buttons", tabPedals: "Pedal curves", tabPresets: "Presets", tabDevice: "Device", tabLabs: "Labs",
  stSending: "Sending", stAcked: "Wheel confirmed", stVerified: "Saved on wheel", stFailed: "Not confirmed", stUnchanged: "",
  stVerifiedLong: "The wheel confirmed the change, and reading it back shows the new value.",
  stAckedLong: "The wheel confirmed the change. Waiting for the read-back.",
  stFailedLong: "The wheel didn't confirm this change, so it may not have been applied.",
  revert: "Revert", revertAll: "Back to the values from when you connected", original: "When connected",
  examples: "Examples", tip: "Tip",
  tuningIntro: "These settings are stored in the wheel and apply to every game. Each change is sent right away, and the app reads it back to check the wheel kept it.",
  reverseOn: "Reversed", reverseOff: "Normal",
  liveIntro: "Press a button, turn the wheel or push a pedal and watch it light up. Nothing on the wheel changes.",
  liveStart: "Start live view", liveStop: "Stop", steering: "Steering", throttle: "Throttle", brake: "Brake", clutch: "Clutch", gear: "Gear", neutral: "N",
  buttonsIntro: "Click a control in the drawing, or press it on the wheel while live view is on, then pick what it sends. A wheel button can also send a keyboard key or a mouse click, which helps in games without wheel support.",
  pickControl: "Pick a control", sends: "Sends", remapTo: "Make it send", groupPad: "Wheel / gamepad", groupKeyboard: "Keyboard key", groupMouse: "Mouse", groupOff: "Disable", resetMapping: "Restore default", remapped: "remapped",
  perMode: "D-Input and X-Input keep separate mappings.",
  pedalsIntro: "A curve sets how far the pedal has to move for a given input in the game. Drag the points. With live view on, a dot shows where your pedal is.",
  linear: "Linear", progressive: "Progressive", aggressive: "Aggressive", sCurve: "S-curve", noCurve: "No curve set (linear)",
  pedalTravel: "Pedal travel", output: "Game input", write: "Write to wheel",
  curveHint: "Progressive is soft at the start. It gives finer throttle in trucks and tractors and makes the brakes harder to lock. Aggressive reacts early, which suits arcade games or a brake pedal that feels too light.",
  platform: "Platform", apply: "Apply", applied: "Applied", loading: "Loading…", offline: "Couldn't reach the LiteStar server.", myPresets: "My presets", saveCurrent: "Save current settings as…", presetName: "Preset name", delete: "Delete",
  modeTitle: "Input mode",
  modeHint: "D-Input is the normal PC wheel mode with force feedback. Use it for sims like ETS2, Assetto Corsa, rFactor and Farming Simulator. X-Input makes the wheel pretend to be an Xbox controller, so some games without wheel support will accept it, but you lose force feedback and a few buttons.",
  modeWarn: "The wheel reconnects over USB when you switch modes. Games and Steam may need a restart to find it again.",
  resetTitle: "Factory reset (this mode)",
  resetHint: "Puts the button mapping, angle, forces and pedal curves for this mode back to factory values.",
  resetConfirm: "Reset every setting for this mode? You can't undo it, but you can apply the \"When connected\" values again afterwards.",
  reset: "Reset", cancel: "Cancel", confirm: "Confirm",
  info: "Device", protocolLog: "Protocol log", clear: "Clear", copied: "Copied",
  labsIntro: "These features exist in the vendor's firmware kit, but the vendor app doesn't show them for this wheel. \"Check\" sends one harmless request. If the wheel answers, it has the feature. If it stays silent, it doesn't, and nothing changes.",
  check: "Check", supported: "Supported", unsupported: "Not on this wheel", unknown: "Not checked",
  rawTitle: "Raw command", rawHint: "For tinkering: send hex bytes and see the reply. The read commands (50, 51) are safe.", send: "Send", noReply: "No reply (unsupported)",
  language: "Language", theme: "Theme",

  tabFfb: "FFB server",
  tabTablet: "Tablet dashboard",
  ffbIntro: "Some games can't drive your wheel's force feedback on Linux. For those, a small mod sends the game's vehicle data to this server, and the server turns it into centering, steering weight, bumps and vibration. Farming Simulator 22 works this way with the FS22_WheelFFB mod. You can write a mod for another game using the same connection.",
  ffbRunning: "Running", ffbStopped: "Stopped", ffbStart: "Start server", ffbStop: "Stop",
  ffbExternal: "Another FFB server is running",
  ffbExternalHint: "This is usually the background server your game launcher started. The live data below comes from it, and it picks up any settings you change here. Close the game if you want this app to take over.",
  ffbTest: "Test wheel", ffbTestHint: "Takes 12 seconds. The wheel pushes left, then right, centers itself, gets heavy, then vibrates. Hold it loosely.",
  ffbWheel: "Wheel", ffbNoWheel: "No force feedback wheel found", ffbGame: "Game", ffbNoGame: "Waiting for a game",
  ffbNoGameHint: "Start a game that has an FFB mod. It connects by itself.",
  ffbModeTelemetry: "Driving · from telemetry", ffbModeForces: "Driving · game's own forces", ffbModeTest: "Testing", ffbModeIdle: "Idle",
  ffbIdleHint: "No forces while you're in a menu, paused, or out of the vehicle.",
  fSpring: "Centering", fDamper: "Weight", fConstant: "Push", fTexture: "Texture", fEngine: "Engine",
  speed: "Speed", rpm: "RPM", surface: "Surface", engineOn: "Engine", on: "on", off: "off",
  ffbFeel: "Feel", ffbFeelHint: "Changes apply immediately, so you can adjust them while driving. Games that send their own forces ignore these.",
  ffbGeneral: "General", autostart: "Start with the app", autostartHint: "The server runs whenever Wheel Studio is open.",
  ffbForModders: "Writing a mod", ffbModdersHint: "Connect to the address below and send JSON: one hello message, then telemetry or forces about 60 times a second. FFB-SERVER.md has the full format and examples in Lua, Python and JavaScript.",
  endpoint: "Send to", watchEndpoint: "Watch live data", copy: "Copy", savedLive: "Saved and applied", saving: "Saving…",
  clients: "Connections", frames: "frames", errors: "errors", settingsFile: "Settings file", docs: "Protocol docs",
  resetFeel: "Default",
  presetsIntro2: "All presets are stored on this computer: LiteStar's official ones for PC, PS4 and Xbox (with their Chinese tips translated), plus Wheel Studio's own. Applying one sends its wheel values. If it also has FFB server settings, those are loaded too. Your button mapping isn't touched.",
  refreshVendor: "Update from LiteStar", refreshed: "Updated", presetFolder: "Open folder",
  srcVendor: "LiteStar", srcStudio: "Wheel Studio", srcUser: "Mine", all: "All",
  needsWheel: "Connect the wheel to apply wheel values.", ffbPart: "Also sets the FFB server feel", angleOnly: "Angle only. LiteStar stores no force values for this one.",
  tutorial: "Guide / video", appliedFfb: "FFB feel loaded", search: "Search games…", noResults: "No presets match.",
  notConnected: "Not connected", connectWheel: "Connect wheel", connectTitle: "Connect your wheel",
  connectBody: "You can use the FFB server and presets without connecting.",
  brokenTitle: "The wheel holds impossible values",
  brokenBody: "Rotation is set to {angle}° and FFB strength to {ffb}. LiteStar's phone app writes some of its presets two bytes off, which moves every value into the wrong slot. That can make the wheel stiff or weak. Repair sets 900° and FFB 100; check the other forces afterwards.",
  brokenFix: "Repair: 900°, FFB 100",
};

const tr: Dict = {
  appName: "V987 Wheel Studio",
  connect: "Bağlan", connecting: "Bağlanıyor…", disconnect: "Bağlantıyı kes", scan: "Ara", scanning: "Aranıyor…",
  noWheel: "Direksiyon bulunamadı. Açık olduğundan ve telefona bağlı olmadığından emin olun.",
  scanHint: "Direksiyon ayarlarını kendi hafızasında tutar, uygulama da bunları Bluetooth üzerinden değiştirir. Önce üreticinin telefon uygulamasını kapatın, çünkü direksiyon aynı anda tek bağlantı kabul ediyor.",
  disconnected: "Bağlantı koptu", reconnect: "Yeniden bağlan",
  firmware: "Yazılım", mode: "Mod", pedals: "Pedallar", shifter: "Vites kolu", detected: "takılı", notDetected: "takılı değil",
  tabTuning: "FFB ayarları", tabLive: "Canlı test", tabButtons: "Tuşlar", tabPedals: "Pedal eğrileri", tabPresets: "Profiller", tabDevice: "Cihaz", tabLabs: "Deneysel",
  stSending: "Gönderiliyor", stAcked: "Direksiyon aldı", stVerified: "Direksiyona kaydedildi", stFailed: "Onay gelmedi", stUnchanged: "",
  stVerifiedLong: "Direksiyon değişikliği onayladı. Geri okunan değer de yeni değerle aynı.",
  stAckedLong: "Direksiyon değişikliği onayladı. Geri okuma bekleniyor.",
  stFailedLong: "Direksiyon bu değişikliği onaylamadı. Uygulanmamış olabilir.",
  revert: "Geri al", revertAll: "Bağlandığımdaki değerlere dön", original: "Bağlanınca",
  examples: "Örnekler", tip: "İpucu",
  tuningIntro: "Bu ayarlar direksiyonun hafızasında durur ve her oyunda geçerlidir. Her değişiklik hemen gönderilir, sonra direksiyondan geri okunarak kaydedildiği kontrol edilir.",
  reverseOn: "Ters", reverseOff: "Normal",
  liveIntro: "Bir tuşa basın, direksiyonu çevirin ya da pedala basın, ekranda yansın. Direksiyonda hiçbir ayar değişmez.",
  liveStart: "Canlı görünümü aç", liveStop: "Durdur", steering: "Direksiyon", throttle: "Gaz", brake: "Fren", clutch: "Debriyaj", gear: "Vites", neutral: "N",
  buttonsIntro: "Çizimde bir kontrole tıklayın (canlı görünüm açıkken direksiyonda basmanız da yeter), sonra ne göndereceğini seçin. Direksiyon tuşu klavye tuşu ya da fare tıklaması da gönderebilir. Direksiyon desteklemeyen oyunlarda işe yarar.",
  pickControl: "Bir kontrol seçin", sends: "Gönderdiği", remapTo: "Bunu göndersin", groupPad: "Direksiyon / gamepad", groupKeyboard: "Klavye tuşu", groupMouse: "Fare", groupOff: "Kapalı", resetMapping: "Varsayılana dön", remapped: "değişti",
  perMode: "D-Input ve X-Input tuş atamalarını ayrı tutar.",
  pedalsIntro: "Eğri, pedalı ne kadar bastığınızda oyuna ne kadar gideceğini belirler. Noktaları sürükleyin. Canlı görünüm açıkken pedalın yeri nokta olarak görünür.",
  linear: "Doğrusal", progressive: "Yumuşak başlangıç", aggressive: "Sert başlangıç", sCurve: "S eğrisi", noCurve: "Eğri yok (doğrusal)",
  pedalTravel: "Pedal yolu", output: "Oyuna giden", write: "Direksiyona yaz",
  curveHint: "Yumuşak başlangıç, pedalın ilk kısmını hassaslaştırır. Tır ve traktörde gazı ince ayarlamak ya da frende teker kilitlememek için iyidir. Sert başlangıç ilk anda tepki verir. Arcade oyunlara ya da fazla hafif gelen fren pedalına uyar.",
  platform: "Platform", apply: "Uygula", applied: "Uygulandı", loading: "Yükleniyor…", offline: "LiteStar sunucusuna ulaşılamadı.", myPresets: "Profillerim", saveCurrent: "Şu anki ayarları kaydet…", presetName: "Profil adı", delete: "Sil",
  modeTitle: "Giriş modu",
  modeHint: "D-Input, FFB'li normal PC direksiyon modudur. ETS2, Assetto Corsa, rFactor, Farming Simulator gibi simülasyonlarda bunu kullanın. X-Input direksiyonu Xbox kolu gibi gösterir. Direksiyon desteklemeyen bazı oyunlar böylece tanır ama FFB ve birkaç tuş gider.",
  modeWarn: "Mod değişince direksiyon USB'den kopup yeniden bağlanır. Oyunların ve Steam'in direksiyonu tekrar görmesi için yeniden başlatmanız gerekebilir.",
  resetTitle: "Fabrika ayarları (bu mod)",
  resetHint: "Bu moddaki tuş atamalarını, açıyı, kuvvetleri ve pedal eğrilerini fabrika değerlerine döndürür.",
  resetConfirm: "Bu moddaki bütün ayarlar sıfırlansın mı? Geri alınamaz, ama \"Bağlanınca\" değerlerini sonra yeniden uygulayabilirsiniz.",
  reset: "Sıfırla", cancel: "Vazgeç", confirm: "Onayla",
  info: "Cihaz", protocolLog: "Protokol kaydı", clear: "Temizle", copied: "Kopyalandı",
  labsIntro: "Bu özellikler üreticinin yazılım altyapısında var ama üreticinin uygulaması bu direksiyonda göstermiyor. \"Dene\" zararsız tek bir istek gönderir. Direksiyon cevap verirse özellik var demektir. Sessiz kalırsa yoktur ve hiçbir şey değişmez.",
  check: "Dene", supported: "Var", unsupported: "Bu direksiyonda yok", unknown: "Denenmedi",
  rawTitle: "Ham komut", rawHint: "Kurcalamak isteyenler için: hex bayt gönderip cevabı görün. Okuma komutları (50, 51) güvenlidir.", send: "Gönder", noReply: "Cevap yok (desteklenmiyor)",
  language: "Dil", theme: "Tema",

  tabFfb: "FFB sunucusu",
  tabTablet: "Tablet paneli",
  ffbIntro: "Bazı oyunlar Linux'ta direksiyonun FFB'sini çalıştıramıyor. Bu oyunlarda küçük bir mod aracın verilerini bu sunucuya gönderir, sunucu da bunları merkezlemeye, direksiyon ağırlığına, tümseklere ve titreşime çevirir. Farming Simulator 22, FS22_WheelFFB moduyla böyle çalışıyor. Aynı bağlantıyı kullanarak başka bir oyun için de mod yazabilirsiniz.",
  ffbRunning: "Çalışıyor", ffbStopped: "Kapalı", ffbStart: "Sunucuyu başlat", ffbStop: "Durdur",
  ffbExternal: "Başka bir FFB sunucusu çalışıyor",
  ffbExternalHint: "Genelde oyun başlatıcısının arka planda açtığı sunucudur. Aşağıdaki canlı veriler ondan geliyor, burada değiştirdiğiniz ayarları da o alır. Bu uygulamanın devralması için oyunu kapatın.",
  ffbTest: "Direksiyonu test et", ffbTestHint: "12 saniye sürer. Direksiyon önce sola, sonra sağa iter, ortalanır, ağırlaşır, sonra titrer. Gevşek tutun.",
  ffbWheel: "Direksiyon", ffbNoWheel: "FFB destekli direksiyon bulunamadı", ffbGame: "Oyun", ffbNoGame: "Oyun bekleniyor",
  ffbNoGameHint: "FFB modu olan bir oyunu açın, kendisi bağlanır.",
  ffbModeTelemetry: "Sürüşte · telemetriden", ffbModeForces: "Sürüşte · oyunun kendi kuvvetleri", ffbModeTest: "Test ediliyor", ffbModeIdle: "Boşta",
  ffbIdleHint: "Menüde, oyun duraklatılmışken ya da araçta değilken kuvvet yok.",
  fSpring: "Merkezleme", fDamper: "Ağırlık", fConstant: "İtme", fTexture: "Zemin", fEngine: "Motor",
  speed: "Hız", rpm: "Devir", surface: "Zemin", engineOn: "Motor", on: "açık", off: "kapalı",
  ffbFeel: "His", ffbFeelHint: "Değişiklikler anında uygulanır, sürerken ayarlayabilirsiniz. Kendi kuvvetlerini gönderen oyunlar bunları kullanmaz.",
  ffbGeneral: "Genel", autostart: "Uygulamayla birlikte başlat", autostartHint: "Wheel Studio açık olduğu sürece sunucu çalışır.",
  ffbForModders: "Mod yazmak", ffbModdersHint: "Aşağıdaki adrese bağlanıp JSON gönderin: önce bir hello mesajı, sonra saniyede 60 kadar telemetri ya da kuvvet mesajı. Tam biçim ve Lua, Python, JavaScript örnekleri FFB-SERVER.md dosyasında.",
  endpoint: "Gönderilecek adres", watchEndpoint: "Canlı veriyi izle", copy: "Kopyala", savedLive: "Kaydedildi ve uygulandı", saving: "Kaydediliyor…",
  clients: "Bağlantılar", frames: "mesaj", errors: "hata", settingsFile: "Ayar dosyası", docs: "Protokol belgesi",
  resetFeel: "Varsayılan",
  presetsIntro2: "Bütün profiller bu bilgisayarda duruyor: LiteStar'ın PC, PS4 ve Xbox için resmi profilleri (Çince ipuçları Türkçeye çevrildi) ve Wheel Studio'nun kendi profilleri. Uyguladığınızda profildeki direksiyon değerleri gönderilir. Profilde FFB sunucusu ayarı da varsa o da yüklenir. Tuş atamalarınıza dokunulmaz.",
  refreshVendor: "LiteStar'dan güncelle", refreshed: "Güncellendi", presetFolder: "Klasörü aç",
  srcVendor: "LiteStar", srcStudio: "Wheel Studio", srcUser: "Benim", all: "Hepsi",
  needsWheel: "Direksiyon değerlerini uygulamak için direksiyonu bağlayın.", ffbPart: "FFB sunucusunun hissini de ayarlar", angleOnly: "Sadece açı. LiteStar bu profil için kuvvet değeri vermiyor.",
  tutorial: "Rehber / video", appliedFfb: "FFB hissi yüklendi", search: "Oyun ara…", noResults: "Eşleşen profil yok.",
  notConnected: "Bağlı değil", connectWheel: "Direksiyonu bağla", connectTitle: "Direksiyonu bağlayın",
  connectBody: "FFB sunucusunu ve profilleri bağlanmadan da kullanabilirsiniz.",
  brokenTitle: "Direksiyonda olmaması gereken değerler var",
  brokenBody: "Dönüş açısı {angle}°, FFB gücü {ffb} olarak kayıtlı. LiteStar'ın telefon uygulaması bazı profillerini iki bayt kaydırarak yazıyor ve bütün değerler yanlış yere düşüyor. Direksiyon bu yüzden sert ya da güçsüz gelebilir. Onar düğmesi 900° ve FFB 100 ayarlar; sonra diğer kuvvetlere de göz atın.",
  brokenFix: "Onar: 900°, FFB 100",
};

const dicts: Record<Lang, Dict> = { en, tr };

export function t(key: string): string {
  return dicts[i18n.lang][key] ?? en[key] ?? key;
}

/** Long-form help for each force feedback setting. */
export interface SettingHelp { title: string; short: string; body: string; examples: string[]; tip?: string }

const help: Record<Lang, Record<string, SettingHelp>> = {
  en: {
    angle: {
      title: "Rotation angle",
      short: "How far the wheel turns from lock to lock.",
      body: "The total rotation from full left to full right. Set the same value in the game, or the car will turn faster or slower than your hands.",
      examples: ["Trucks (ETS2): 900°, same as LiteStar's ETS2 preset", "Road cars (Assetto Corsa, Forza): 900°", "Rally (WRC): about 460°", "F1: about 360°", "Tractors: 900° to 1080°"],
      tip: "If the car turns before the wheel does, or the other way round, the game's angle doesn't match this one.",
    },
    ffb: {
      title: "Force feedback strength",
      short: "Overall strength of what the game sends.",
      body: "Scales every force the game sends: the road, kerbs, crashes and the pull of the steering.",
      examples: ["100: full strength, used by every LiteStar preset", "60 to 80: for long sessions, or if the wheel knocks", "0: no forces from the game"],
      tip: "If strong moments all feel the same and the detail disappears (clipping), lower this before lowering the game's setting.",
    },
    reverse: {
      title: "Reverse force direction",
      short: "Flips which way the wheel pushes.",
      body: "A few games send forces with the wrong sign. The wheel then pushes into the corner and feels unstable.",
      examples: ["Off: almost every game", "On: only if the wheel pulls away from center in a corner"],
    },
    spring: {
      title: "Spring (centering)",
      short: "How hard the wheel pulls back to the middle.",
      body: "LiteStar's Chinese label is 回中弹力, the return-to-center spring. A high value snaps the wheel back to straight. A low value lets it stay where you leave it.",
      examples: ["Trucks (ETS2): 30, from LiteStar's preset", "Most sims (Assetto Corsa, F1): 50 to 60", "Games without their own force feedback: 70 to 80", "Drifting: 20 to 40, for fast countersteer"],
    },
    damping: {
      title: "Damping",
      short: "Resistance that grows the faster you turn.",
      body: "Slow turns stay light and fast turns get heavy, a bit like turning through oil. It calms the steering and stops it wobbling on straights.",
      examples: ["Trucks and buses: 70 to 80", "Road cars: 40 to 60", "Rally (WRC): 20 to 30", "Drifting: as low as you can"],
    },
    friction: {
      title: "Friction / road detail",
      short: "How much road texture and impact comes through.",
      body: "LiteStar describes this as the detail games use for the road surface and collisions: bumps, gravel, kerbs and hits. Higher lets more of it through. Lower gives a smoother, quieter wheel.",
      examples: ["Sims (ETS2, Assetto Corsa): 70 to 80", "Gravel rally: 40 to 60, or it gets tiring", "Arcade or long relaxed drives: 20 to 40"],
    },
    inertia: {
      title: "Inertia",
      short: "How heavy the wheel feels when it starts and stops moving.",
      body: "Imitates the weight of the steering column. With high inertia the wheel is slow to get going and keeps turning a little after you let go, like a big truck. Low inertia feels light and direct.",
      examples: ["Trucks and tractors: 70 to 80", "Road cars: 50 to 70", "Racing and rally: 20 to 40"],
    },
  },
  tr: {
    angle: {
      title: "Dönüş açısı",
      short: "Direksiyonun bir uçtan öbür uca kaç derece döndüğü.",
      body: "Tam soldan tam sağa toplam dönüş. Oyunda da aynı değeri seçin, yoksa araç ellerinizden hızlı ya da yavaş döner.",
      examples: ["Tır (ETS2): 900°, LiteStar'ın ETS2 profiliyle aynı", "Otomobil (Assetto Corsa, Forza): 900°", "Ralli (WRC): 460° civarı", "F1: 360° civarı", "Traktör: 900° ile 1080° arası"],
      tip: "Araç direksiyondan önce dönüyorsa ya da geç kalıyorsa oyundaki açı bununla aynı değil.",
    },
    ffb: {
      title: "FFB gücü",
      short: "Oyundan gelen kuvvetlerin genel gücü.",
      body: "Oyunun gönderdiği bütün kuvvetleri ölçekler: yol hissi, bordürler, çarpmalar ve direksiyonun çekişi.",
      examples: ["100: tam güç, LiteStar'ın bütün profillerinde böyle", "60 ile 80 arası: uzun sürüşler ya da direksiyon vuruyorsa", "0: oyundan hiç kuvvet gelmez"],
      tip: "Sert anların hepsi aynı hissettiriyor ve detay kayboluyorsa (clipping), oyunun ayarından önce bunu düşürün.",
    },
    reverse: {
      title: "Kuvvet yönünü ters çevir",
      short: "Direksiyonun ittiği yönü değiştirir.",
      body: "Birkaç oyun kuvveti ters işaretle gönderir. O zaman direksiyon virajda içeri doğru iter ve araç oynak hissettirir.",
      examples: ["Kapalı: neredeyse bütün oyunlar", "Açık: sadece direksiyon virajda ortadan kaçıyorsa"],
    },
    spring: {
      title: "Yay (merkezleme)",
      short: "Direksiyonun ortaya ne kadar sert döndüğü.",
      body: "LiteStar'ın Çince adı 回中弹力, yani merkeze döndüren yay. Yüksek değerde direksiyon hemen düze gelir. Düşük değerde bıraktığınız yerde kalır.",
      examples: ["Tır (ETS2): 30, LiteStar profilinden", "Çoğu simülasyon (Assetto Corsa, F1): 50 ile 60 arası", "Kendi FFB'si olmayan oyunlar: 70 ile 80 arası", "Drift: 20 ile 40 arası, hızlı kontra için"],
    },
    damping: {
      title: "Sönümleme",
      short: "Ne kadar hızlı çevirirseniz o kadar artan direnç.",
      body: "Yavaş çevirince hafif, hızlı çevirince ağır gelir, yağın içinde çevirmek gibi. Direksiyonu sakinleştirir, düz yolda sağa sola yalpalamasını önler.",
      examples: ["Tır ve otobüs: 70 ile 80 arası", "Otomobil: 40 ile 60 arası", "Ralli (WRC): 20 ile 30 arası", "Drift: olabildiğince düşük"],
    },
    friction: {
      title: "Sürtünme / yol detayı",
      short: "Yol dokusunun ve darbelerin ne kadarının hissedildiği.",
      body: "LiteStar bunu oyunların yol yüzeyi ve çarpmalar için kullandığı detay olarak tanımlıyor: tümsek, çakıl, bordür, darbe. Yüksek değerde bunları daha çok hissedersiniz. Düşük değerde direksiyon daha pürüzsüz ve sessiz olur.",
      examples: ["Simülasyon (ETS2, Assetto Corsa): 70 ile 80 arası", "Çakılda ralli: 40 ile 60 arası, yoksa yorar", "Arcade ya da uzun, rahat sürüşler: 20 ile 40 arası"],
    },
    inertia: {
      title: "Atalet",
      short: "Direksiyonun harekete geçerken ve dururken ne kadar ağır geldiği.",
      body: "Direksiyon milinin ağırlığını taklit eder. Atalet yüksekse direksiyon zor harekete geçer ve bıraktıktan sonra biraz daha döner, büyük bir tırdaki gibi. Düşükse hafif ve dolaysız gelir.",
      examples: ["Tır ve traktör: 70 ile 80 arası", "Otomobil: 50 ile 70 arası", "Yarış ve ralli: 20 ile 40 arası"],
    },
  },
};

export function settingHelp(key: string): SettingHelp {
  return help[i18n.lang][key] ?? help.en[key];
}
