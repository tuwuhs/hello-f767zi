#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;

use panic_halt as _;

use cortex_m::peripheral::syst;
use cortex_m_rt::exception;
use stm32f7::stm32f7x7;

use core::num::Wrapping;

fn yeah() {
    asm::nop();
}

#[exception]
fn SysTick() {
    static mut MILLIS: Wrapping<usize> = Wrapping(0);

    *MILLIS += Wrapping(1);
}

#[entry]
fn main() -> ! {
    asm::nop(); // To not have main optimize to abort in release mode, remove when you add code

    let cp = cortex_m::Peripherals::take().unwrap();
    let p = stm32f7x7::Peripherals::take().unwrap();
    
    let rcc = p.RCC;
    let pwr = p.PWR;
    
    // Enable PWR, set voltage scale
    rcc.apb1enr.modify(|_, w| w.pwren().set_bit());
    pwr.cr1.modify(|_, w| w.vos().scale1());

    // Enable HSE
    rcc.cr.modify(|_, w| w.hsebyp().set_bit().hseon().set_bit());
    while rcc.cr.read().hserdy().is_not_ready() {};

    // Configure PLL
    rcc.cr.modify(|_, w| w.pllon().clear_bit());
    rcc.pllcfgr.write(|w| unsafe {
        w.pllsrc().hse().
        pllm().bits(4).
        plln().bits(216).
        pllp().div2().
        pllq().bits(9)
    });
    rcc.cr.modify(|_, w| w.pllon().set_bit());
    while rcc.cr.read().pllrdy().is_not_ready() {};

    // Enable overdrive
    pwr.cr1.modify(|_, w| w.oden().set_bit());
    while pwr.csr1.read().odrdy().bit_is_clear() {};
    pwr.cr1.modify(|_, w| w.odswen().set_bit());
    while pwr.csr1.read().odswrdy().bit_is_clear() {};

    // Configure clock and flash latency
    let flash = p.FLASH;
    let flash_latency: u8 = 7;
    if flash_latency > flash.acr.read().latency().bits() {
        flash.acr.modify(|_, w| { w.latency().bits(flash_latency) });
    }
    rcc.cfgr.modify(|_, w| w.
        ppre1().div16().
        ppre2().div16().
        hpre().div1().
        sw().pll()
    );
    while !rcc.cfgr.read().sw().is_pll() {};
    if flash_latency < flash.acr.read().latency().bits() {
        flash.acr.modify(|_, w| { w.latency().bits(flash_latency) });
    }
    rcc.cfgr.modify(|_, w| w.
        ppre1().div4().
        ppre2().div2()
    );

    // SysTick initialization
    let mut systick = cp.SYST;
    systick.set_clock_source(syst::SystClkSource::Core);
    systick.set_reload(8_000_000 - 1);
    systick.clear_current();
    systick.enable_counter();
    systick.enable_interrupt();

    // GPIO initialization
    rcc.ahb1enr.modify(|_, w| w.gpioben().set_bit().gpiocen().set_bit());

    let gpiob = p.GPIOB;
    gpiob.ospeedr.modify(|_, w| w.
        ospeedr0().low_speed().
        ospeedr7().low_speed().
        ospeedr14().low_speed());
    gpiob.pupdr.modify(|_, w| w.
        pupdr0().floating().
        pupdr7().floating().
        pupdr14().floating());
    gpiob.moder.modify(|_, w| w.
        moder0().output().
        moder7().output().
        moder14().output());

    let gpioc = p.GPIOC;
    gpioc.pupdr.modify(|_, w| w.pupdr13().floating());
    gpioc.moder.modify(|_, w| w.moder13().input());

    yeah();

    while !systick.has_wrapped() {

    }
    
    gpiob.bsrr.write(|w| w.bs7().set_bit());

    loop {
        match gpioc.idr.read().idr13().bit() {
            false => gpiob.bsrr.write(|w| w.br14().set_bit()),
            true => gpiob.bsrr.write(|w| w.bs14().set_bit())
        }

        if systick.has_wrapped() {
            match gpiob.odr.read().odr0().bit() {
                false => gpiob.bsrr.write(|w| w.bs0().set_bit()),
                true => gpiob.bsrr.write(|w| w.br0().set_bit())
            }

            if gpiob.odr.read().odr7().bit_is_set() {
                gpiob.bsrr.write(|w| w.br7().set_bit());
            } else {
                gpiob.bsrr.write(|w| w.bs7().set_bit());
            }
        }

        // // while !systick.has_wrapped() {};
        // cortex_m::asm::delay(24_000_000);
        // gpiob.bsrr.write(|w| w.bs0().set_bit());
        // // while !systick.has_wrapped() {};
        // cortex_m::asm::delay(24_000_000);
        // gpiob.bsrr.write(|w| w.br0().set_bit());        
    }
}
