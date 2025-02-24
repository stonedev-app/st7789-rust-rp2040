#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor, prelude::*};
use embedded_hal::{digital::OutputPin, spi::MODE_0};
use panic_probe as _;

use rp_pico::{
    self as bsp,
    hal::{fugit::RateExtU32, gpio::FunctionSpi, Spi},
};

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

use embedded_hal_bus::spi::ExclusiveDevice;
use mipidsi::{interface::SpiInterface, models::ST7789, options::ColorInversion, Builder};

// Display
const W: i32 = 135;
const H: i32 = 240;
const X_OFFSET: u16 = 52;
const Y_OFFSET: u16 = 40;

#[entry]
fn main() -> ! {
    info!("Program start");
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

    // Define the delay struct, needed for the display driver
    let mut delay = DelayCompat(cortex_m::delay::Delay::new(
        core.SYST,
        clocks.system_clock.freq().to_Hz(),
    ));

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    /* Define the DC digital output pin as the variable `dc` */
    let dc = pins.gpio7.into_push_pull_output();

    /* Define the SPI interface as the variable `spi` */
    let sclk = pins.gpio2.into_function::<FunctionSpi>();
    let mosi = pins.gpio3.into_function::<FunctionSpi>();

    /* Define the CS digital output pin as the variable `cs` */
    let cs = pins.gpio5.into_push_pull_output();

    /* Define the BL digital output pin as the variable `bl` */
    let mut bl = pins.gpio6.into_push_pull_output();

    let spi_device = pac.SPI0;
    let spi_pin_layout = (mosi, sclk);

    let spi = Spi::<_, _, _, 8>::new(spi_device, spi_pin_layout).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16_000_000u32.Hz(),
        MODE_0,
    );

    let spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    // Create a buffer
    let mut buffer = [0_u8; 512];

    // Create a DisplayInterface from SPI and DC pin, with no manual CS control
    let di = SpiInterface::new(spi_device, dc, &mut buffer);

    // Define the display from the display interface and initialize it
    let mut display = Builder::new(ST7789, di)
        .display_size(W as u16, H as u16)
        .display_offset(X_OFFSET, Y_OFFSET)
        .invert_colors(ColorInversion::Inverted)
        .init(&mut delay)
        .unwrap();

    // Make the display all black
    display.clear(Rgb565::BLACK).unwrap();

    // Turn on backlight
    bl.set_high().unwrap();

    loop {
        // Do nothing
    }
}

/// Wrapper around `Delay` to implement the embedded-hal 1.0 delay.
///
/// This can be removed when a new version of the `cortex_m` crate is released.
struct DelayCompat(cortex_m::delay::Delay);

impl embedded_hal::delay::DelayNs for DelayCompat {
    fn delay_ns(&mut self, mut ns: u32) {
        while ns > 1000 {
            self.0.delay_us(1);
            ns = ns.saturating_sub(1000);
        }
    }

    fn delay_us(&mut self, us: u32) {
        self.0.delay_us(us);
    }
}
