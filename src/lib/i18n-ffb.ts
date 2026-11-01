import { i18n, type Lang, type SettingHelp } from "./i18n.svelte";

/** Help for the FFB server's settings, same shape as the wheel settings' help. */
const help: Record<Lang, Record<string, SettingHelp>> = {
  en: {
    master_gain: {
      title: "Overall strength",
      short: "Scales every force the server makes.",
      body: "Applied on top of the wheel's own force feedback strength setting.",
      examples: ["0.8: strong, with plenty of detail", "0.5 to 0.6: long sessions, or a wheel that knocks", "1.0: full strength"],
    },
    invert: {
      title: "Reverse pushes",
      short: "Flips the direction of bumps and impacts.",
      body: "Turn this on if a bump under the left front wheel twists the wheel the wrong way. Centering and steering weight stay the same.",
      examples: ["Off: normal", "On: if hitting a kerb on the right turns the wheel left"],
    },
    centering: {
      title: "Centering at speed",
      short: "How hard the wheel returns to straight while you're moving.",
      body: "Grows with speed, the way caster works on a real vehicle. On fields and snow the tyres grip less, so it gets weaker there.",
      examples: ["0.55: a tractor on the road, a clear but relaxed pull", "0.75: snappy, closer to a car", "0.3: light, for long days in the field"],
    },
    centering_parked: {
      title: "Centering when stopped",
      short: "A small pull to the middle while standing still.",
      body: "A real tractor hardly centers itself when it isn't moving. A little of it stops the wheel flopping around.",
      examples: ["0.08: barely there", "0: completely free", "0.2: the wheel always finds the middle"],
    },
    damping: {
      title: "Steering weight",
      short: "Resistance when you turn, like power steering.",
      body: "Fast turns feel heavier than slow ones, which keeps the wheel calm.",
      examples: ["0.3: power steering", "0.5: heavy machinery", "0.15: light and quick"],
    },
    damping_engine_off: {
      title: "Weight with the engine off",
      short: "Heavy steering when there's no power assist.",
      body: "With the engine off and the vehicle stopped, the power steering pump isn't running and the wheel gets much heavier, as on a real tractor.",
      examples: ["0.65: clearly heavy", "0.9: hard work to turn", "Same as steering weight: no difference"],
    },
    bumps: {
      title: "Bumps",
      short: "The twist you feel when one front wheel hits something.",
      body: "When the left and right front wheels move differently over a hole, rut or kerb, the wheel twists in your hands.",
      examples: ["0.6: easy to feel", "0.9: rough ground becomes a workout", "0.3: subtle"],
    },
    impacts: {
      title: "Impacts",
      short: "A jolt for collisions and hard landings.",
      body: "Sudden stops and landings send a short kick through the wheel.",
      examples: ["0.8: you notice every hit", "0.4: softer", "0: no jolts"],
    },
    texture: {
      title: "Surface texture",
      short: "Fine vibration from the ground.",
      body: "Stronger on fields and gravel than on asphalt, and faster as you speed up.",
      examples: ["0.25: there, but not tiring", "0.5: you feel every furrow", "0: smooth"],
    },
    soft_lock: {
      title: "Soft lock",
      short: "The wheel pushes back where the in-game steering stops.",
      body: "A game that knows its steering lock can tell the server when you turn past it. FS22_WheelFFB does this, scaled for each vehicle, so the end of the in-game steering feels like a real stop.",
      examples: ["0.8: a firm stop", "0.4: a gentle hint", "0: off; the wheel just keeps turning"],
    },
    engine: {
      title: "Engine vibration",
      short: "A hum that follows the rpm.",
      body: "You feel it most when slow or stopped. It fades as you drive faster.",
      examples: ["0.1: a faint diesel hum", "0.25: an old tractor", "0: off"],
    },
  },
  tr: {
    master_gain: {
      title: "Genel güç",
      short: "Sunucunun ürettiği bütün kuvvetleri ölçekler.",
      body: "Direksiyonun kendi FFB gücü ayarının üstüne uygulanır.",
      examples: ["0.8: güçlü ve detaylı", "0.5 ile 0.6 arası: uzun sürüşler ya da vuran direksiyon", "1.0: tam güç"],
    },
    invert: {
      title: "İtmeleri ters çevir",
      short: "Tümsek ve darbelerin yönünü değiştirir.",
      body: "Sol ön tekerin altındaki tümsek direksiyonu yanlış yöne çeviriyorsa açın. Merkezleme ve direksiyon ağırlığı değişmez.",
      examples: ["Kapalı: normal", "Açık: sağdaki bordüre çarpınca direksiyon sola dönüyorsa"],
    },
    centering: {
      title: "Hızda merkezleme",
      short: "Giderken direksiyonun düze ne kadar sert döndüğü.",
      body: "Gerçek araçtaki kaster açısı gibi hızla artar. Tarlada ve karda lastik daha az tuttuğu için orada zayıflar.",
      examples: ["0.55: yolda giden traktör, belirgin ama rahat", "0.75: otomobile yakın, keskin", "0.3: hafif, tarlada uzun mesai için"],
    },
    centering_parked: {
      title: "Dururken merkezleme",
      short: "Araç dururken ortaya doğru hafif bir çekiş.",
      body: "Gerçek traktör dururken kendini neredeyse hiç toplamaz. Biraz olması direksiyonun boşta savrulmasını engeller.",
      examples: ["0.08: zar zor hissedilir", "0: tamamen serbest", "0.2: direksiyon hep ortayı bulur"],
    },
    damping: {
      title: "Direksiyon ağırlığı",
      short: "Çevirirken gelen direnç, hidrolik direksiyon gibi.",
      body: "Hızlı çevirmek yavaş çevirmekten ağır gelir, direksiyon da sakin kalır.",
      examples: ["0.3: hidrolik direksiyon", "0.5: ağır iş makinesi", "0.15: hafif ve çevik"],
    },
    damping_engine_off: {
      title: "Motor kapalıyken ağırlık",
      short: "Hidrolik destek yokken direksiyon ağırlaşır.",
      body: "Motor kapalı ve araç dururken hidrolik pompa çalışmaz, direksiyon da gerçek traktördeki gibi epey ağırlaşır.",
      examples: ["0.65: belirgin şekilde ağır", "0.9: çevirmek zor", "Direksiyon ağırlığıyla aynı: fark olmaz"],
    },
    bumps: {
      title: "Tümsekler",
      short: "Ön tekerlerden biri bir şeye çarpınca gelen burulma.",
      body: "Sol ve sağ ön teker bir çukurda, izde ya da bordürde farklı hareket edince direksiyon elinizde döner.",
      examples: ["0.6: rahatça hissedilir", "0.9: engebeli arazi spor gibi olur", "0.3: hafif"],
    },
    impacts: {
      title: "Darbeler",
      short: "Çarpışmada ve sert inişte gelen sarsıntı.",
      body: "Ani duruşlar ve inişler direksiyona kısa bir vuruş verir.",
      examples: ["0.8: her çarpmayı anlarsınız", "0.4: daha yumuşak", "0: sarsıntı yok"],
    },
    texture: {
      title: "Zemin dokusu",
      short: "Zeminden gelen ince titreşim.",
      body: "Tarlada ve çakılda asfalttan güçlüdür, hız arttıkça sıklaşır.",
      examples: ["0.25: hissedilir ama yormaz", "0.5: her saban izini hissedersiniz", "0: pürüzsüz"],
    },
    soft_lock: {
      title: "Yumuşak kilit",
      short: "Oyundaki direksiyon dayandığı yerde direksiyon geri iter.",
      body: "Direksiyonunun nerede kilitlendiğini bilen bir oyun, o noktayı geçtiğinizi sunucuya söyleyebilir. FS22_WheelFFB bunu her araç için ayrı hesaplıyor, böylece oyundaki direksiyonun sonu gerçek bir dayanak gibi hissedilir.",
      examples: ["0.8: sert bir dayanak", "0.4: hafif bir uyarı", "0: kapalı, direksiyon dönmeye devam eder"],
    },
    engine: {
      title: "Motor titreşimi",
      short: "Devre göre değişen bir uğultu.",
      body: "En çok yavaşken ya da dururken hissedilir, hızlandıkça azalır.",
      examples: ["0.1: hafif bir dizel uğultusu", "0.25: eski bir traktör", "0: kapalı"],
    },
  },
};

export function ffbHelp(key: string): SettingHelp {
  return help[i18n.lang][key] ?? help.en[key];
}
