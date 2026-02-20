#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    main,
    spi::{
        master::{Config as SpiConfig, Spi},
        Mode,
    },
    time::{Duration, Instant, Rate},
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

const REG_FIFO: u8 = 0x00;
const REG_OP_MODE: u8 = 0x01;
const REG_FRF_MSB: u8 = 0x06;
const REG_FRF_MID: u8 = 0x07;
const REG_FRF_LSB: u8 = 0x08;
const REG_PA_CONFIG: u8 = 0x09;
const REG_FIFO_ADDR_PTR: u8 = 0x0D;
const REG_FIFO_TX_BASE_ADDR: u8 = 0x0E;
const REG_FIFO_RX_CURRENT_ADDR: u8 = 0x10;
const REG_IRQ_FLAGS: u8 = 0x12;
const REG_RX_NB_BYTES: u8 = 0x13;
const REG_PKT_RSSI_VALUE: u8 = 0x1A;
const REG_MODEM_CONFIG_1: u8 = 0x1D;
const REG_MODEM_CONFIG_2: u8 = 0x1E;
const REG_PREAMBLE_MSB: u8 = 0x20;
const REG_PREAMBLE_LSB: u8 = 0x21;
const REG_PAYLOAD_LENGTH: u8 = 0x22;
const REG_MODEM_CONFIG_3: u8 = 0x26;
const REG_DIO_MAPPING_1: u8 = 0x40;
const REG_VERSION: u8 = 0x42;

const MODE_LONG_RANGE: u8 = 0x80;
const MODE_SLEEP: u8 = 0x00;
const MODE_STDBY: u8 = 0x01;
const MODE_TX: u8 = 0x03;
const MODE_RX_CONTINUOUS: u8 = 0x05;

const IRQ_TX_DONE: u8 = 0x08;
const IRQ_RX_DONE: u8 = 0x40;
const IRQ_CRC_ERR: u8 = 0x20;

fn sx_write(spi: &mut Spi<'_, esp_hal::Blocking>, nss: &mut Output<'_>, reg: u8, val: u8) {
    let frame = [reg | 0x80, val];
    nss.set_low();
    let _ = spi.write(&frame);
    nss.set_high();
}

fn sx_read(spi: &mut Spi<'_, esp_hal::Blocking>, nss: &mut Output<'_>, reg: u8) -> u8 {
    let mut frame = [reg & 0x7f, 0];
    nss.set_low();
    let _ = spi.transfer(&mut frame);
    nss.set_high();
    frame[1]
}

fn sx_write_fifo(spi: &mut Spi<'_, esp_hal::Blocking>, nss: &mut Output<'_>, data: &[u8]) {
    nss.set_low();
    let _ = spi.write(&[REG_FIFO | 0x80]);
    let _ = spi.write(data);
    nss.set_high();
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let mut led = Output::new(peripherals.GPIO25, Level::Low, OutputConfig::default());

    let mut lora_nss = Output::new(peripherals.GPIO18, Level::High, OutputConfig::default());
    let mut lora_rst = Output::new(peripherals.GPIO14, Level::High, OutputConfig::default());
    let lora_dio0 = Input::new(peripherals.GPIO26, InputConfig::default());

    let spi_cfg = SpiConfig::default()
        .with_frequency(Rate::from_mhz(8))
        .with_mode(Mode::_0);
    let mut spi = Spi::new(peripherals.SPI2, spi_cfg)
        .expect("SPI init failed")
        .with_sck(peripherals.GPIO5)
        .with_mosi(peripherals.GPIO27)
        .with_miso(peripherals.GPIO19);

    println!("Boot: Heltec Wireless Stick V1 (ESP32)");
    println!("LoRa test: SCK=5 MISO=19 MOSI=27 NSS=18 RST=14 DIO0=26");

    // Hardware reset SX1276.
    lora_rst.set_low();
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_millis(20) {}
    lora_rst.set_high();
    let t1 = Instant::now();
    while t1.elapsed() < Duration::from_millis(20) {}

    let version = sx_read(&mut spi, &mut lora_nss, REG_VERSION);
    println!("SX127x version reg = 0x{:02x} (expect 0x12)", version);

    // Basic LoRa config, 915.125 MHz, SF7/BW125, CR 4/5, CRC on.
    sx_write(
        &mut spi,
        &mut lora_nss,
        REG_OP_MODE,
        MODE_LONG_RANGE | MODE_SLEEP,
    );
    sx_write(
        &mut spi,
        &mut lora_nss,
        REG_OP_MODE,
        MODE_LONG_RANGE | MODE_STDBY,
    );
    sx_write(&mut spi, &mut lora_nss, REG_FRF_MSB, 0xE4);
    sx_write(&mut spi, &mut lora_nss, REG_FRF_MID, 0xC8);
    sx_write(&mut spi, &mut lora_nss, REG_FRF_LSB, 0x00);
    sx_write(&mut spi, &mut lora_nss, REG_PA_CONFIG, 0x8F);
    sx_write(&mut spi, &mut lora_nss, REG_MODEM_CONFIG_1, 0x72);
    sx_write(&mut spi, &mut lora_nss, REG_MODEM_CONFIG_2, 0x74);
    sx_write(&mut spi, &mut lora_nss, REG_MODEM_CONFIG_3, 0x04);
    sx_write(&mut spi, &mut lora_nss, REG_PREAMBLE_MSB, 0x00);
    sx_write(&mut spi, &mut lora_nss, REG_PREAMBLE_LSB, 0x08);
    sx_write(&mut spi, &mut lora_nss, REG_DIO_MAPPING_1, 0x00); // DIO0=RxDone/TxDone.
    sx_write(&mut spi, &mut lora_nss, REG_FIFO_TX_BASE_ADDR, 0x00);
    sx_write(&mut spi, &mut lora_nss, REG_IRQ_FLAGS, 0xFF);

    let mut counter: u32 = 0;

    loop {
        led.toggle();
        counter = counter.wrapping_add(1);

        let msg = [
            b'p',
            b'i',
            b'n',
            b'g',
            b' ',
            b'#',
            b'0' + ((counter / 100) % 10) as u8,
            b'0' + ((counter / 10) % 10) as u8,
            b'0' + (counter % 10) as u8,
        ];

        sx_write(
            &mut spi,
            &mut lora_nss,
            REG_OP_MODE,
            MODE_LONG_RANGE | MODE_STDBY,
        );
        sx_write(&mut spi, &mut lora_nss, REG_IRQ_FLAGS, 0xFF);
        sx_write(&mut spi, &mut lora_nss, REG_FIFO_ADDR_PTR, 0x00);
        sx_write_fifo(&mut spi, &mut lora_nss, &msg);
        sx_write(&mut spi, &mut lora_nss, REG_PAYLOAD_LENGTH, msg.len() as u8);

        println!("LoRa TX: ping #{:03}", counter % 1000);
        sx_write(
            &mut spi,
            &mut lora_nss,
            REG_OP_MODE,
            MODE_LONG_RANGE | MODE_TX,
        );

        let tx_wait = Instant::now();
        while !lora_dio0.is_high() && tx_wait.elapsed() < Duration::from_millis(1500) {}
        let irq = sx_read(&mut spi, &mut lora_nss, REG_IRQ_FLAGS);
        sx_write(&mut spi, &mut lora_nss, REG_IRQ_FLAGS, 0xFF);
        println!("TX done, IRQ=0x{:02x}", irq);

        // Switch to RX and listen for a short response packet.
        sx_write(
            &mut spi,
            &mut lora_nss,
            REG_OP_MODE,
            MODE_LONG_RANGE | MODE_RX_CONTINUOUS,
        );
        let rx_wait = Instant::now();
        while !lora_dio0.is_high() && rx_wait.elapsed() < Duration::from_millis(1200) {}
        let rx_irq = sx_read(&mut spi, &mut lora_nss, REG_IRQ_FLAGS);

        if (rx_irq & IRQ_RX_DONE) != 0 && (rx_irq & IRQ_CRC_ERR) == 0 {
            let len = sx_read(&mut spi, &mut lora_nss, REG_RX_NB_BYTES);
            let addr = sx_read(&mut spi, &mut lora_nss, REG_FIFO_RX_CURRENT_ADDR);
            sx_write(&mut spi, &mut lora_nss, REG_FIFO_ADDR_PTR, addr);

            let mut first = 0u8;
            if len > 0 {
                first = sx_read(&mut spi, &mut lora_nss, REG_FIFO);
            }
            let rssi = sx_read(&mut spi, &mut lora_nss, REG_PKT_RSSI_VALUE);
            println!(
                "LoRa RX: len={} first=0x{:02x} rssi_raw={} irq=0x{:02x}",
                len, first, rssi, rx_irq
            );
        } else if (rx_irq & IRQ_TX_DONE) == 0 {
            println!("LoRa RX timeout/none irq=0x{:02x}", rx_irq);
        }
        sx_write(&mut spi, &mut lora_nss, REG_IRQ_FLAGS, 0xFF);

        let pause = Instant::now();
        while pause.elapsed() < Duration::from_millis(1000) {}
    }
}
