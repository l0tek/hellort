# Rust ESP Boilerplate - Heltec Wireless Stick

Minimaler Startpunkt fuer den Heltec Wireless Stick auf Basis von `esp-hal`.

## Voraussetzungen

- `espup` installiert und ausgefuehrt (stellt Toolchain `esp` bereit)
- `espflash` installiert

Beispiel:

```bash
cargo install espup espflash
espup install
source "$HOME/export-esp.sh"
```

## Build & Flash

```bash
cargo +esp run
```

Die Runner-Config nutzt standardmaessig:

- Target: `xtensa-esp32-none-elf`
- Runner: `espflash flash --monitor`

## Wireless Stick V1 Pins

Bekannte Zuordnung fuer Heltec Wireless Stick V1:

- LED: `GPIO25`
- LoRa SPI: `SCK=GPIO5`, `MISO=GPIO19`, `MOSI=GPIO27`, `NSS=GPIO18`
- LoRa Control: `RST=GPIO14`, `DIO0=GPIO26`, `DIO1=GPIO35`, `BUSY=GPIO34`
- OLED I2C: `SDA=GPIO4`, `SCL=GPIO15`, `RST=GPIO16`
- Vext Control: `GPIO21` (wird oft fuer externe Versorgung/OLED verwendet)

Das Beispiel in `src/main.rs` blinkt die LED auf `GPIO25`.

## Flash-Hinweis

Falls `espflash` nicht automatisch in den Bootloader kommt:

1. `PRG` gedrueckt halten
2. kurz `RST` tippen
3. `PRG` loslassen
4. `cargo +esp run` erneut ausfuehren
