pub mod gpio;
pub mod uart;
pub mod mmio;

use cortex_a::asm;
use crate::kernel::{self, driver::Driver, io};

pub use asm::nop;

static mut UART: uart::PL011Uart = uart::PL011Uart{};
static mut GPIO: gpio::GPIO = gpio::GPIO{};

pub fn console() -> &'static mut impl io::Console {
    unsafe { &mut UART }
} 

pub fn drivers() -> [&'static dyn Driver; 2] {
    unsafe { [&GPIO, &UART] }
}

pub fn post_init() {
    unsafe { GPIO.map_pl011_uart() }
}

#[no_mangle]
extern "C" fn _start() -> ! {
    kernel::init()
}

pub fn spin_for_cycles(cycles: usize) {
    for _ in 0..cycles {
        asm::nop();
    }
}
