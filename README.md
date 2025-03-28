# SSD1680 Rust Driver

This is a low level driver for the SSD1680 e-paper display controller.

The goal is to make the use of the SSD1680 controller simpler by providing some checks and types while still having full control over the chip.

The interface is subject to change.

## Example usage with esp-rs

```rust
#![no_std]
#![no_main]

use embedded_hal_bus::spi::AtomicDevice;
use embedded_hal_bus::util::AtomicCell;
use esp_backtrace as _;
use esp_hal::gpio::{Input, InputConfig, Output, OutputConfig};
use esp_hal::spi::master::Spi;
use esp_hal::time::Rate;
use esp_hal::{main, spi};

#[main]
fn main() -> ! {
    let peripherals =
        esp_hal::init(esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::_160MHz));

    let rst = Output::new(
        peripherals.GPIO0,
        esp_hal::gpio::Level::Low,
        OutputConfig::default(),
    );
    let dc = Output::new(
        peripherals.GPIO21,
        esp_hal::gpio::Level::High,
        OutputConfig::default(),
    );
    let cs = Output::new(
        peripherals.GPIO1,
        esp_hal::gpio::Level::Low,
        OutputConfig::default(),
    );
    let busy = Input::new(
        peripherals.GPIO23,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::None),
    );

    let sck = peripherals.GPIO19;
    let mosi = peripherals.GPIO18;
    let miso = peripherals.GPIO20;

    let spi_bus = AtomicCell::new(
        Spi::new(
            peripherals.SPI2,
            spi::master::Config::default()
                .with_frequency(Rate::from_khz(10000))
                .with_write_bit_order(spi::BitOrder::MsbFirst)
                .with_read_bit_order(spi::BitOrder::MsbFirst)
                .with_mode(spi::Mode::_0),
        )
        .unwrap()
        .with_mosi(mosi)
        .with_sck(sck)
        .with_miso(miso),
    );

    let delay = esp_hal::delay::Delay::new();

    let ssd1680_spi = AtomicDevice::new(&spi_bus, cs, delay).unwrap();
    let mut ssd1680_display = ssd1680_rs::SSD1680::new(
        rst,
        dc,
        busy,
        delay,
        ssd1680_spi,
        ssd1680_rs::config::DisplayConfig::epd_290_t94(),
    );

    let mut y = 0;

    loop {
        ssd1680_display.hw_init().unwrap();
        ssd1680_display.set_ram_counter_y(y).unwrap();
        ssd1680_display.write_bw_byte(0xFF).unwrap();
        ssd1680_display.full_refresh().unwrap();
        ssd1680_display.enter_deep_sleep().unwrap();

        y += 1;
        if y > 295 {
            y = 0;
        }

        delay.delay_millis(1000);
    }
}
```

Cargo.toml:

```toml
[package]
name = "ssd1680-driver-test"
version = "0.1.0"
edition = "2021"

[dependencies]
esp-backtrace = { version = "0.15.1", features = [
    "esp32c6",
    "exception-handler",
    "panic-handler",
    "println",
] }

esp-hal = { version = "1.0.0-beta.0", features = ["esp32c6", "unstable"] }
esp-println = { version = "0.13.1", features = ["esp32c6", "log"] }
log = { version = "0.4.21" }
critical-section = "1.2.0"
embedded-hal-bus = "0.3.0"
embedded-hal = "1.0.0"
ssd1680-rs = { branch = "main", git = "https://github.com/nponsard/ssd1680-rs" }

[profile.dev]
opt-level = "s"

[profile.release]
codegen-units = 1
debug = 2
debug-assertions = false
incremental = false
lto = 'fat'
opt-level = 's'
overflow-checks = false
```
