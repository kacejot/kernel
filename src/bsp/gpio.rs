// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! GPIO driver.

use core::ops;
use register::{mmio::ReadWrite, register_bitfields, register_structs};

use crate::kernel::driver;

// GPIO registers.
//
// Descriptions taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
register_bitfields! {
    u32,

    /// GPIO Function Select 1
    GPFSEL1 [
        /// Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100  // PL011 UART RX

        ],

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100  // PL011 UART TX
        ]
    ],

    /// GPIO Pull-up/down Clock Register 0
    GPPUDCLK0 [
        /// Pin 15
        PUDCLK15 OFFSET(15) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 14
        PUDCLK14 OFFSET(14) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ]
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => GPFSEL0: ReadWrite<u32>),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => GPFSEL2: ReadWrite<u32>),
        (0x0C => GPFSEL3: ReadWrite<u32>),
        (0x10 => GPFSEL4: ReadWrite<u32>),
        (0x14 => GPFSEL5: ReadWrite<u32>),
        (0x18 => _reserved1),
        (0x94 => GPPUD: ReadWrite<u32>),
        (0x98 => GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>),
        (0x9C => GPPUDCLK1: ReadWrite<u32>),
        (0xA0 => @END),
    }
}

/// GPIO driver
pub struct GPIO;

impl GPIO {
    fn ptr(&self) -> *const RegisterBlock {
        self.base_addr as *const _
    }

    pub fn map_pl011_uart(&self) {
        use crate::bsp;

        self
            .GPFSEL1
            .modify(GPFSEL1::FSEL14::AltFunc0 + GPFSEL1::FSEL15::AltFunc0);

        // Enable pins 14 and 15.
        self.GPPUD.set(0);
        bsp::spin_for_cycles(150);

        self
            .GPPUDCLK0
            .write(GPPUDCLK0::PUDCLK14::AssertClock + GPPUDCLK0::PUDCLK15::AssertClock);
            bsp::spin_for_cycles(150);

        self.GPPUDCLK0.set(0);
    }
}

impl ops::Deref for GPIO {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

//--------------------------------------------------------------------------------------------------
// OS interface implementations
//--------------------------------------------------------------------------------------------------

impl driver::Driver for GPIO {
    fn name(&self) -> &str {
        "GPIO"
    }
}
