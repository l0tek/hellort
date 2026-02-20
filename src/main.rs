#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output, OutputConfig},
    main,
    time::{Duration, Instant},
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let mut led = Output::new(peripherals.GPIO25, Level::Low, OutputConfig::default());

    println!("Boot: Heltec Wireless Stick V1 (ESP32)");

    loop {
        led.toggle();
        println!("tick");
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(500) {}
    }
}
