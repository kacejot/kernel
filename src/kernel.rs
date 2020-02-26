pub mod io;
pub mod driver;
pub mod result;

use crate::{bsp, kernel::io::{Read, Write}};

pub fn init() -> ! {
    for driver in bsp::drivers().iter() {
        if let Err(_) = driver.init() {
            panic!("failed to load driver: {:?}", driver.name())
        }
    }
    bsp::post_init();
    
    kernel_main()
}

fn kernel_main() -> ! {
    let mut data = [0u8];

    // wait until user hits Enter
    loop {
        bsp::console().read(&mut data);
        if data[0] as char == '\n' {
            break;
        }
    }

    // echo the input
    loop {
        bsp::console().read(&mut data);
        bsp::console().write(&data);
    }
}
