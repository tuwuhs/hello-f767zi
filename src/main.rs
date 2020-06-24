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
use stm32f7::stm32f7x7;

fn yeah() {
    asm::nop();
}

#[entry]
fn main() -> ! {
    asm::nop(); // To not have main optimize to abort in release mode, remove when you add code

    let cp = cortex_m::Peripherals::take().unwrap();
    let p = stm32f7x7::Peripherals::take().unwrap();

    let mut systick = cp.SYST;
    systick.set_clock_source(syst::SystClkSource::Core);
    systick.set_reload(8_000_000);
    systick.clear_current();
    systick.enable_counter();

    let rcc = p.RCC;
    rcc.ahb1enr.modify(|_, w| w.
        gpioben().set_bit().
        gpiocen().set_bit()
    );

    let gpiob = p.GPIOB;
    gpiob.ospeedr.modify(|_, w| w.
        ospeedr0().low_speed().
        ospeedr7().low_speed().
        ospeedr14().low_speed()
    );
    gpiob.pupdr.modify(|_, w| w.
        pupdr0().floating().
        pupdr7().floating().
        pupdr14().floating()
    );
    gpiob.moder.modify(|_, w| w.
        moder0().output().
        moder7().output().
        moder14().output()
    );

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

        // while !systick.has_wrapped() {};
        // gpiob.bsrr.write(|w| w.bs0().set_bit());
        // while !systick.has_wrapped() {};
        // gpiob.bsrr.write(|w| w.br0().set_bit());        
    }
}
