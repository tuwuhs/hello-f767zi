#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;

use cortex_m::peripheral::syst;
use cortex_m_rt::exception;

use stm32f7xx_hal::{device, prelude::*};

use core::num::Wrapping;
use core::sync::atomic::{AtomicUsize, Ordering};

static MILLIS: AtomicUsize = AtomicUsize::new(0);

#[exception]
fn SysTick() {
    MILLIS.fetch_add(1, Ordering::Relaxed);
}

fn get_millis() -> usize {
    MILLIS.load(Ordering::Relaxed)
}

fn has_elapsed(start: usize, timeout_msec: usize) -> bool {
    Wrapping(get_millis()) - Wrapping(start) >= Wrapping(timeout_msec)
}

fn delay_ms(msec: usize) {
    let start = get_millis();
    while !has_elapsed(start, msec) {};
}

#[entry]
fn main() -> ! {
    asm::nop(); // To not have main optimize to abort in release mode, remove when you add code
    
    let p = device::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    
    // Clock initialization
    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.mhz()).freeze();

    // SysTick initialization
    let mut systick = cp.SYST;
    systick.set_clock_source(syst::SystClkSource::Core);
    systick.set_reload(clocks.hclk().0 / 1000 - 1);
    systick.clear_current();
    systick.enable_counter();
    systick.enable_interrupt();

    // GPIO initialization
    let gpiob = p.GPIOB.split();
    let gpioc = p.GPIOC.split();
    
    let mut led_green = gpiob.pb0.into_push_pull_output();
    let mut led_blue = gpiob.pb7.into_push_pull_output();
    let mut led_red = gpiob.pb14.into_push_pull_output();
    let button = gpioc.pc13.into_floating_input();
    
    led_blue.set_high().unwrap();

    let mut heartbeat_timer = get_millis();

    loop {
        match button.is_high() {
            Ok(false) => led_red.set_low().unwrap(),
            Ok(true) => led_red.set_high().unwrap(),
            _ => ()
        }

        if has_elapsed(heartbeat_timer, 500) {
            heartbeat_timer = get_millis();

            match led_green.is_high() {
                Ok(false) => led_green.set_high().unwrap(),
                Ok(true) => led_green.set_low().unwrap(),
                _ => ()
            }

            if led_blue.is_high().unwrap() {
                led_blue.set_low().unwrap();
            } else {
                led_blue.set_high().unwrap();
            }
        }   
    }
}
