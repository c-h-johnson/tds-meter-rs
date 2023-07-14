# tds-meter-rs

[![](https://badgers.space/crates/info/tds-meter)](https://crates.io/crates/tds-meter)
[![](https://badgers.space/github/open-issues/c-h-johnson/tds-meter-rs)](https://github.com/c-h-johnson/tds-meter-rs/issues)

An embedded-hal driver for TDS Meter (total disolved solids sensor)

# Device

Links:

- [product page](https://whadda.com/product/tds-total-dissolved-solids-water-quality-sensor-wpm356/)
- [manual](https://cdn.velleman.eu/downloads/25/prototyping/manual_wpm356.pdf)

Also available under different brands (look for "water conductivity sensor" on ebay/amazon).

# Example

```rust
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use tds_meter::TdsMeter;

use rp_pico as bsp;

use bsp::hal::{
    adc::Adc,
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let adc_pin_0 = pins.gpio28.into_floating_input();
    // pico has 12-bit ADC and has 3.3V reference voltage
    let mut tds: TdsMeter<_, _, u16, _, _> = TdsMeter::new(adc, 4096, 3.3, adc_pin_0);


    loop {
        match tds.measure(25., &mut delay) {
            Ok(tds_value) => info!("tds: {} ppm", tds_value),
            Err(_) => error!("error occurred"),
        }
        delay.delay_ms(1000);
    }
}
```

# Results

Using pi pico:

| Water | PPM |
| ----- | --- |
| none/dry | <4.5 |
| de-ionised | 5 |
| rain | 11 |
| hard tap water | 310  |
| saline | 1290 |
