// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! PL011 UART driver.

use core::ops;
use register::{mmio::{WriteOnly, ReadOnly, ReadWrite}, register_bitfields, register_structs};

use crate::{ bsp::{self, mmio }, kernel::{io, result::{ KernelError, KernelResult}, driver} };

// PL011 UART registers.
//
// Descriptions taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
register_bitfields! {
    u32,
    /// Data Register
    DR [
        /// Overrun error. This bit is set to 1 if data is received and the receive FIFO is already
        /// full.
        /// 
        /// This is cleared to 0 once there is an empty space in the FIFO and a new character can
        /// be written to it.
        OE OFFSET(11) NUMBITS(1) [],
        
        /// Break error. This bit is set to 1 if a break condition was detected, indicating that
        /// the received data input was held LOW for longer than a full-word transmission time
        /// (defined as start, data, parity and stop bits).
        /// 
        /// In FIFO mode, this error is associated with the character at the top of the FIFO. When
        /// a break occurs, only one 0 character is loaded into the FIFO. The next character is
        /// only enabled after the receive data input goes to a 1 (marking state), an
        BE OFFSET(10) NUMBITS(1) [],

        /// Parity error. When set to 1, it indicates that the parity of the received data
        /// character does not match the parity that the EPS and SPS bits in the Line Control
        /// Register, UART_LCRH select. In FIFO mode, this error is associated with the character
        /// at the top of the FIFO.
        PE OFFSET(9) NUMBITS(1) [],

        /// Framing error. When set to 1, it indicates that the received character did not have a
        /// valid stop bit (a valid stop bit is 1). In FIFO mode, this error is associated with the
        /// character at the top of the FIFO.
        FE OFFSET(8) NUMBITS(1) [],

        /// Receive (read) data character.
        /// Transmit (write) data character.
        DATA OFFSET(0) NUMBITS(8) []
    ],

    /// Flag Register
    FR [
        /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// Line Control Register, UARTLCR_ LCRH.
        ///
        /// If the FIFO is disabled, this bit is set when the transmit holding register is empty. If
        /// the FIFO is enabled, the TXFE bit is set when the transmit FIFO is empty. This bit does
        /// not indicate if there is data in the transmit shift register.
        TXFE OFFSET(7) NUMBITS(1) [],

        /// Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit in the
        /// UARTLCR_ LCRH Register.
        ///
        /// If the FIFO is disabled, this bit is set when the transmit holding register is full. If
        /// the FIFO is enabled, the TXFF bit is set when the transmit FIFO is full.
        TXFF OFFSET(5) NUMBITS(1) [],

        /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// UARTLCR_H Register.
        ///
        /// If the FIFO is disabled, this bit is set when the receive holding register is empty. If
        /// the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
        RXFE OFFSET(4) NUMBITS(1) []
    ],

    /// Integer Baud rate divisor
    IBRD [
        /// Integer Baud rate divisor
        IBRD OFFSET(0) NUMBITS(16) []
    ],

    /// Fractional Baud rate divisor
    FBRD [
        /// Fractional Baud rate divisor
        FBRD OFFSET(0) NUMBITS(6) []
    ],

    /// Line Control register
    LCRH [
        /// Word length. These bits indicate the number of data bits transmitted or received in a
        /// frame.
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit = 0b00,
            SixBit = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11
        ],

        /// Enable FIFOs:
        ///
        /// 0 = FIFOs are disabled (character mode) that is, the FIFOs become 1-byte-deep holding
        /// registers
        ///
        /// 1 = transmit and receive FIFO buffers are enabled (FIFO mode).
        FEN OFFSET(4) NUMBITS(1) [
            FifosDisabled = 0,
            FifosEnabled = 1
        ]
    ],

    /// Control Register
    CR [
        /// Receive enable. If this bit is set to 1, the receive section of the UART is enabled.
        /// Data reception occurs for UART signals. When the UART is disabled in the middle of
        /// reception, it completes the current character before stopping.
        RXE OFFSET(9) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// Transmit enable. If this bit is set to 1, the transmit section of the UART is enabled.
        /// Data transmission occurs for UART signals. When the UART is disabled in the middle of
        /// transmission, it completes the current character before stopping.
        TXE OFFSET(8) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// UART enable
        UARTEN OFFSET(0) NUMBITS(1) [
            /// If the UART is disabled in the middle of transmission or reception, it completes the
            /// current character before stopping.
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// Interrupt Clear Register
    ICR [
        /// Meta field for all pending interrupts
        ALL OFFSET(0) NUMBITS(11) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => DR: ReadWrite<u32, DR::Register>),
        (0x04 => _reserved1),
        (0x18 => FR: ReadOnly<u32, FR::Register>),
        (0x1c => _reserved2),
        (0x24 => IBRD: WriteOnly<u32, IBRD::Register>),
        (0x28 => FBRD: WriteOnly<u32, FBRD::Register>),
        (0x2c => LCRH: WriteOnly<u32, LCRH::Register>),
        (0x30 => CR: WriteOnly<u32, CR::Register>),
        (0x34 => _reserved3),
        (0x44 => ICR: WriteOnly<u32, ICR::Register>),
        (0x48 => @END),
    }
}

pub struct PL011Uart;

impl ops::Deref for PL011Uart {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl PL011Uart {
    /// Return a pointer to the register block.
    fn ptr(&self) -> *const RegisterBlock {
        mmio::UART_BASE as *const _
    }
}

impl driver::Driver for PL011Uart {
    fn name(&self) -> &str {
        "PL011Uart"
    }

    /// Set up baud rate and characteristics.
    ///
    /// Results in 8N1 and 230400 baud (if the clk has been previously set to 48 MHz by the
    /// firmware).
    fn init(&self) -> KernelResult {
        // UART init state
        self.CR.set(0);
        self.ICR.write(ICR::ALL::CLEAR);
        
        // Set baud rate
        self.IBRD.write(IBRD::IBRD.val(13));
        self.FBRD.write(FBRD::FBRD.val(2));
        
        // Set 8-bit as data size and enable FIFO
        self.LCRH
            .write(LCRH::WLEN::EightBit + LCRH::FEN::FifosEnabled); // 8N1 + Fifo on
        
        // Enable UART, enable RW
        self.CR
            .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);

        Ok(())
    }
}

impl io::Write for PL011Uart {
    type Err = KernelError;

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Err> {

        for byte in buf {
            while self.FR.matches_all(FR::TXFF::SET) {
                bsp::nop();
            }

            self.DR.write(DR::DATA.val(*byte as u32));
        }

        Ok(buf.len())
    }
}

impl io::Read for PL011Uart {
    type Err = KernelError;
    
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Err> {
        for byte in buf { 
            while self.FR.matches_all(FR::RXFE::SET) {
                bsp::nop();
            }
    
            *byte = self.DR.read(DR::DATA) as u8;
        }

        Ok(buf.len())
    }
}
