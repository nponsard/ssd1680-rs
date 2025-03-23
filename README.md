# SSD1680 Rust Driver

This is a low level driver for the SSD1680 e-paper display controller.

The goal is to make the use of the SSD1680 controller simpler by providing some checks and types while still having full control over the chip.

The interface is subject to change.

## Example usage with esp-rs

```rust
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

    let mut delay = esp_hal::delay::Delay::new();

    let ssd1680_spi = AtomicDevice::new(&spi_bus, cs, delay).unwrap();
    let mut ssd1680_display = ssd1680_rs::Ssd1680Display::new(
    rst,
    dc,
    busy,
    delay,
    ssd1680_spi,
    ssd1680_rs::DisplayConfig::epd_290_t94()
    );

    ssd1680_display.hw_init().unwrap();
    ssd1680_display.write_bw_byte(0xFF).unwrap();
    ssd1680_display.full_refresh().unwrap();
    ssd1680_display.enter_deep_sleep().unwrap();

    loop{}
}
```
