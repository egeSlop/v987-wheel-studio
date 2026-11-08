import { i18n } from "./i18n.svelte";

/** Text for the Tablet dashboard page. */
const en = {
  intro: "Open the dashboard on a tablet or phone on the same Wi-Fi: a car-radio screen with live data from the game, and buttons that press the game's keys on this PC. Each game has its own button layout.",
  running: "Running", stopped: "Stopped", start: "Start", stop: "Stop",
  openOn: "Open this on the tablet", scan: "Or scan the code with the tablet's camera. It pairs the tablet too.",
  copy: "Copy", copied: "Copied",
  pin: "PIN", newPin: "New PIN", pinHelp: "Tablets enter this PIN once (the QR code fills it in). A new PIN unpairs every tablet.",
  requirePin: "Ask for a PIN", requirePinHelp: "Without it, anyone on your network can press keys on this PC.",
  autostart: "Start with Wheel Studio", autostartHelp: "Also starts with the headless FFB server that game launchers use.",
  port: "Port", portHelp: "Changing it restarts the dashboard.",
  tablets: "Connected tablets", noTablets: "None yet.", paired: "Paired", notPaired: "Not paired", presses: "presses",
  now: "Right now", noGame: "No game is sending data.", live: "Live", paused: "Paused or in the menu", via: "via",
  sources: "Data sources",
  src_ffb: "FFB server stream (FS22 and other FFB mods)", src_udp: "UDP 47822 (ETS2/ATS plugin, community tools)", src_dr2: "DiRT Rally 2.0 (UDP 20777)", src_ac: "Assetto Corsa (UDP 9996)", src_feed: "WebSocket /feed (community mods)",
  listening: "listening", connected: "connected", down: "not available", lastData: "last data",
  keyboard: "Key presses", kbReady: "Virtual keyboard ready.", kbLazy: "The virtual keyboard is created on the first press.",
  games: "Games", setUp: "Set up", reinstall: "Reinstall", ready: "Ready", missing: "Needs setup", absent: "Not installed", manual: "Manual",
  fs22Manual: "Install FS22_WheelFFB 1.7 or newer.",
  dr2Missing: "Start DiRT Rally 2.0 once so it writes its settings, then press Set up.",
  done: "Done", untested: "untested in the game",
  profiles: "Profiles", profilesHelp: "One TOML file per game decides the radio, the readouts and the buttons. Put your own files in the profiles folder; a file with a built-in id replaces it.",
  openFolder: "Open profiles folder", reload: "Reload", buttons: "buttons", builtIn: "built-in", custom: "custom",
  test: "Test", testHelp: "Press a profile button from here. Click into the game first: the key goes to the focused window, in 3 seconds.",
  testIn: "Pressing in 3 s…", pressed: "Pressed",
  docs: "Mod authors: how to send data and write profiles",
  firewall: "Tablet can't connect? Allow TCP port {port} in your firewall.",
};

type Key = keyof typeof en;

const tr: Record<Key, string> = {
  intro: "Paneli aynı Wi-Fi'daki bir tablette ya da telefonda açın: oyundan canlı veri gösteren bir oto teyp ekranı ve bu PC'de oyunun tuşlarına basan butonlar. Her oyunun kendi buton düzeni var.",
  running: "Çalışıyor", stopped: "Durdu", start: "Başlat", stop: "Durdur",
  openOn: "Tablette bunu açın", scan: "Ya da kodu tabletin kamerasıyla okutun. Tableti de eşleştirir.",
  copy: "Kopyala", copied: "Kopyalandı",
  pin: "PIN", newPin: "Yeni PIN", pinHelp: "Tabletler bu PIN'i bir kez girer (QR kod kendisi doldurur). Yeni PIN bütün tabletlerin eşleşmesini kaldırır.",
  requirePin: "PIN iste", requirePinHelp: "Kapatırsanız ağınızdaki herkes bu PC'de tuşlara basabilir.",
  autostart: "Wheel Studio ile başlat", autostartHelp: "Oyun başlatıcılarının kullandığı arka plan FFB sunucusuyla da başlar.",
  port: "Port", portHelp: "Değiştirince panel yeniden başlar.",
  tablets: "Bağlı tabletler", noTablets: "Henüz yok.", paired: "Eşleşti", notPaired: "Eşleşmedi", presses: "basış",
  now: "Şu an", noGame: "Veri gönderen bir oyun yok.", live: "Canlı", paused: "Duraklatıldı ya da menüde", via: "kaynak:",
  sources: "Veri kaynakları",
  src_ffb: "FFB sunucusu akışı (FS22 ve diğer FFB modları)", src_udp: "UDP 47822 (ETS2/ATS eklentisi, topluluk araçları)", src_dr2: "DiRT Rally 2.0 (UDP 20777)", src_ac: "Assetto Corsa (UDP 9996)", src_feed: "WebSocket /feed (topluluk modları)",
  listening: "dinleniyor", connected: "bağlı", down: "kullanılamıyor", lastData: "son veri",
  keyboard: "Tuş basma", kbReady: "Sanal klavye hazır.", kbLazy: "Sanal klavye ilk basışta oluşturulur.",
  games: "Oyunlar", setUp: "Kur", reinstall: "Yeniden kur", ready: "Hazır", missing: "Kurulum gerekli", absent: "Yüklü değil", manual: "Elle",
  fs22Manual: "FS22_WheelFFB 1.7 ya da üstünü kurun.",
  dr2Missing: "Ayarlarını yazması için DiRT Rally 2.0'ı bir kez açın, sonra Kur'a basın.",
  done: "Tamam", untested: "oyunda denenmedi",
  profiles: "Profiller", profilesHelp: "Her oyun için bir TOML dosyası radyoyu, göstergeleri ve butonları belirler. Kendi dosyalarınızı profil klasörüne koyun; hazır bir profille aynı id'ye sahip dosya onun yerine geçer.",
  openFolder: "Profil klasörünü aç", reload: "Yenile", buttons: "buton", builtIn: "hazır", custom: "sizin",
  test: "Dene", testHelp: "Bir profil butonuna buradan basın. Önce oyuna tıklayın: tuş 3 saniye sonra o an açık olan pencereye gider.",
  testIn: "3 sn sonra basılıyor…", pressed: "Basıldı",
  docs: "Mod yazarları için: veri gönderme ve profil yazma",
  firewall: "Tablet bağlanamıyor mu? Güvenlik duvarında TCP {port} portuna izin verin.",
};

export function td(k: Key, vars?: Record<string, string | number>): string {
  let s = (i18n.lang === "tr" ? tr : en)[k] ?? en[k];
  if (vars) for (const [a, b] of Object.entries(vars)) s = s.replace(`{${a}}`, String(b));
  return s;
}
