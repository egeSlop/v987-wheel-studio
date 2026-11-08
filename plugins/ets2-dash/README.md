# v987-dash (ETS2 / ATS telemetry plugin)

A small SCS Telemetry SDK plugin for Euro Truck Simulator 2 and American Truck Simulator. It reads truck state from the game and sends it as JSON over UDP to `127.0.0.1:47822`, one JSON object per datagram. V987 Wheel Studio listens on that port and feeds the tablet dashboard.

**UNTESTED in the real games.** Neither game was installed when this was written. The FFI declarations were written by hand from the SDK headers, and the plugin was only checked against a fake host (a C program that loads the library and calls the callbacks with made-up values).

This plugin was vibecoded with Claude.

## Install

Copy the library into the game's `plugins` folder (create the folder if it is missing):

| Platform | File | Folder |
|---|---|---|
| Windows | `v987_dash.dll` | `<game>/bin/win_x64/plugins/` |
| Linux (native) | `libv987_dash.so` (or `v987_dash.so`) | `<game>/bin/linux_x64/plugins/` |

On start the game shows a prompt about "advanced SDK features" being used by a plugin. You have to accept it or the plugin is not loaded. The game log (`game.log.txt` in the documents folder) then contains a line `v987-dash: streaming to udp 127.0.0.1:47822`.

If you play the Windows version through Proton, use the `.dll`.

## Datagram format

Sent at about 20 Hz while driving, at 2 Hz while paused, and right away when any on/off value (lights, blinker switches, engine, parking brake and so on) or the paused state changes. The blinker lamp fields are the exception: they flip on and off while blinking, so they don't trigger an extra send. One last datagram with `"active":false` is sent when the game unloads the plugin.

```json
{"type":"dash","game":"ets2","name":"Euro Truck Simulator 2","source":"v987-dash-ets2 1.0",
 "active":true,"speedKmh":82.4,"rpm":1320,"maxRpm":2500,"gear":"12","gearsForward":12,"gearsReverse":2,
 "engine":true,"electric":true,"fuel":0.620,"fuelLiters":372.0,"fuelRange":1450,"cruise":80,"speedLimit":90,
 "navDistance":12.4,"navTime":540,"odometer":123456.7,"parkingBrake":false,"lightsParking":true,
 "lightsLow":true,"lightsHigh":false,"beacon":false,"blinkerLeft":false,"blinkerRight":false,
 "blinkerLampLeft":false,"blinkerLampRight":false,
 "hazard":false,"wipers":false,"retarder":0,"vehicle":"Scania S"}
```

| Field | Meaning |
|---|---|
| `game`, `name` | `ets2` / `Euro Truck Simulator 2` or `ats` / `American Truck Simulator` |
| `active` | false while the game is paused (menus, map) and in the final datagram |
| `speedKmh` | speedometer speed in km/h, always positive (also when reversing) |
| `rpm`, `maxRpm` | engine rpm; `maxRpm` is the truck's rpm limit, left out until the truck config is known |
| `gear` | `"N"`, `"1"`..`"n"`, or `"R1"`..`"Rn"` (the gear shown on the dashboard) |
| `gearsForward`, `gearsReverse` | gear counts from the truck config (0 until known) |
| `engine`, `electric` | engine running, electrics on |
| `fuel` | tank fill 0..1; left out until the tank capacity is known |
| `fuelLiters`, `fuelRange` | fuel in litres, estimated range in km |
| `cruise` | cruise control set speed in km/h, 0 when off |
| `speedLimit` | route advisor speed limit in km/h, 0 when there is none |
| `navDistance`, `navTime` | distance to route end in km, ETA in seconds |
| `odometer` | km |
| `parkingBrake`, `lightsParking`, `lightsLow`, `lightsHigh`, `beacon`, `wipers` | booleans |
| `blinkerLeft`, `blinkerRight` | blinker switch state (`truck.lblinker` / `truck.rblinker`), stays true as long as the indicator is on |
| `blinkerLampLeft`, `blinkerLampRight` | the blinker lamp itself (`truck.light.lblinker` / `rblinker`), toggles while blinking |
| `hazard` | hazard switch state; only present when the game has the `truck.hazard.warning` channel (newer SDK versions) |
| `retarder` | retarder step, 0 = off |
| `vehicle` | truck brand and model, left out until known |

## Build

```sh
cargo build --release                                   # target/release/libv987_dash.so
cargo build --release --target x86_64-pc-windows-gnu    # target/x86_64-pc-windows-gnu/release/v987_dash.dll
```

The Windows build needs the mingw-w64 gcc (`x86_64-w64-mingw32-gcc`), set as linker in `.cargo/config.toml`. Only 64-bit builds are supported.

## Türkçe

ETS2 ve ATS için küçük bir telemetri eklentisi. Oyundan tır bilgilerini (hız, devir, vites, yakıt, farlar, sinyaller, hız sabitleyici vb.) okuyup JSON olarak UDP ile `127.0.0.1:47822` adresine gönderiyor. Her pakette tek bir JSON nesnesi var. V987 Wheel Studio bu portu dinleyip tabletteki göstergeye aktarıyor.

**Gerçek oyunda TEST EDİLMEDİ.** Yazılırken iki oyun da kurulu değildi. FFI tanımları SDK başlık dosyalarına bakılarak elle yazıldı; eklenti sadece sahte bir host programıyla denendi. Eklenti Claude ile vibecode'landı.

### Kurulum

- Windows: `v987_dash.dll` dosyasını `<oyun klasörü>/bin/win_x64/plugins/` içine kopyala.
- Linux (yerel sürüm): `.so` dosyasını `<oyun klasörü>/bin/linux_x64/plugins/` içine kopyala.
- Proton ile Windows sürümünü oynuyorsan `.dll` dosyasını kullan.

`plugins` klasörü yoksa kendin oluştur. Oyun açılırken "advanced SDK features" diye bir uyarı çıkar, bunu kabul etmen gerekiyor, yoksa eklenti yüklenmez. Her şey yolundaysa oyunun log dosyasında (`game.log.txt`) `v987-dash: streaming to udp 127.0.0.1:47822` satırını görürsün.

### Veri

Sürüş sırasında saniyede yaklaşık 20, oyun duraklatılmışken saniyede 2 paket gidiyor. Bir far, sinyal kolu, motor ya da el freni gibi açık/kapalı bir değer değişince paket hemen gönderiliyor. Alanların anlamı yukarıdaki İngilizce tabloda. Kısaca: hızlar km/s, mesafeler km, süreler saniye. `blinkerLeft`/`blinkerRight` sinyal kolunun durumunu gösterir ve sinyal açık olduğu sürece `true` kalır. `blinkerLampLeft`/`blinkerLampRight` ise yanıp sönen lambanın kendisidir. Oyun kapanırken son olarak `"active":false` içeren bir paket gönderiliyor.
