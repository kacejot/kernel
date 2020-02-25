pub mod gpio;
pub mod uart;

use crate::kernel;

#[no_mangle]
extern "C" fn _start() -> ! {
    kernel::init()
}

pub fn spin_for_cycles(cycles: usize) {
    use cortex_a::asm;
    for _ in 0..cycles {
        asm::nop();
    }
}
