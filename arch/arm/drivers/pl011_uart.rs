/* platform::drivers::pl011_uart */
/* Implementation of Serial for ARM's PL011 UART chip (PL011) */
// See http://infocenter.arm.com/help/topic/com.arm.doc.ddi0183f/DDI0183.pdf
// See also https://code.google.com/p/xv6-rpi/source/browse/src/device/uart.c?spec=svnc73095f6b10f30786b6786732d66532f7fce1988&r=c73095f6b10f30786b6786732d66532f7fce1988
// (RPi implementation of PL011 in C)
use kernel::serial::*;
use platform::drivers::chip;
use platform::io;
use platform::cpu::interrupt;
use kernel;

// TODO resizable buffers
pub static UART_BUFF_SZ : uint = 1024;
// TODO Does not use FIFO
// TODO Does not use error checking

pub struct PL011 
{
    // NB: Base addresses should be mutable, to allow for re-mapping 
    base : u32,
    IRQ : u32,
    
    rate : baud,

    receiver : unsafe fn (),

    buffer : [u8, .. UART_BUFF_SZ],
    buf_head : uint,
    buf_count : uint,

    // TODO proper receive handlers

}

impl Serial for PL011
{

    /// Initialize device and begin transmission. Returns true if device successfully opened.
    fn open(&mut self, rate : u32) -> bool
    {
        unsafe{
            // Set baud rate
            let int_divisor : u16 = (chip::UART_CLK / (16 * rate as uint)) as u16;
            // f_divisor is only a 6-bit quantity;
            // "...taking the fractional part of the required baud rate divisor and multiplying it by 64
            // and adding 0.5 to account for rounding errors..."
            let temp : uint = (8 * (chip::UART_CLK % (16 * rate as uint))) / (rate as uint);
            let f_divisor : u8 = (((temp >> 1) + (temp & 1)) & 0x3F) as u8;
            

            if(int_divisor == 0 || int_divisor == (1 << 16) - 1)
            {
                return false;
            }
            
            io::wh(self.base + IBRD, int_divisor as u32);
            io::wh(self.base + FBRD, f_divisor as u32);

            // Set to 8 bits, 1 stop bit, no parity, no FIFO
            io::wh(self.base + LCR_H, 
                LCR_WLEN_8 as u32
                // | LCR_FIFOEN
                );

            // Enable UART
            io::wh(self.base + CR,
                CR_EN
                | CR_RXE
                | CR_TXE
                );

            // Default interrupts for transmit / receive come when FIFO is 1/2 full; FIFO disables

            // enable PL0110 IRQ [4]
            *chip::VIC_INT_ENABLE = 1 << self.IRQ;
            
            // enable RXIM interrupt (interrupt on receive)
            /*
             * See
             * http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0183f/I54603.html
             */
            // We use WS here because writing a 0 will clear the bit on the mask. 
            io::ws(self.base + IMSC, IMSC_RXIM);

            // TODO: Add IRQ handler for 
            
            kernel::int_table.map(|t| {
                t.enable(interrupt::IRQ, self.receiver);
            }); 
        }
        self.buf_head = 0;
        self.buf_count = 0;
        false
    }

    fn isOpen(&self) -> bool
    {
        self.rate == 0
    }

    /// End transmission, close device. Returns true if device is closed after operation.
    fn close(&mut self) -> bool
    {
        self.rate = 0;
        self.buf_head = 0;
        self.buf_count = 0;
        true
    }

    /// Number of bytes available to read
    fn available(&self) -> uint
    {
        self.buf_count
    }
    
    /// Read up to length bytes into buffer. Return number of bytes read.
    fn readBuf(&mut self, buffer : &mut [u8], length : uint) -> uint
    {
        let mut i = 0;
        while (i < length && self.buf_count > 0)
        {
            self.read(&mut buffer[i]);
            i += 1;
        }
        i
    }

    /// Read one character into buffer. Return number of bytes read.
    fn read(&mut self, c : &mut u8) -> uint
    {
        if self.buf_count == 0 
        {
            return 0;
        }
        else
        {
            *c = self.buffer[self.buf_head];
            self.buf_head += 1;
            self.buf_count -= 1;
            return 1;
        }
    }

    /// Write a single byte. Return number of bytes written.
    fn write(&self, c : u8) -> uint
    {
        unsafe {
            /*
             * We need to include a blank asm call to prevent rustc
             * from optimizing this part out
             */
            asm!("");
            io::wh(self.base + DR, c as u32);
        }
        1
    }

    /// Write a buffer of bytes. Return number of bytes written.
    fn writeBuf(&self, buffer : &[u8], length : uint) -> uint
    {
        let mut i = 0;
        while (i < length)
        {
            self.write(buffer[i]);
            i += 1;
        }
        return length;
    }

    fn flush(&self) -> uint
    {
        0
    }

    /// Callback on new data available.
    fn addReceiveHandler(&self, newHandler : serialReceiveHandler) -> bool
    {
        false
    }

    /// Remove all receive handlers
    fn clearReceiveHandlers(&self)
    {
        ()
    }
}

impl PL011
{
    fn new(_base : u32, _IRQ : u32, _receiver : unsafe fn() ) -> PL011
    {
        PL011 {
            base : _base,
            IRQ : _IRQ,
            receiver : _receiver,

            rate : 0,
            buffer : [0, .. UART_BUFF_SZ],
            buf_head : 0,
            buf_count : 0,
        } 
    }

    pub fn receive(&mut self, c : u8) -> bool
    {
        if(self.buf_count == UART_BUFF_SZ)
        {
            false
        }else
        {
            self.buffer[(self.buf_head + self.buf_count) % UART_BUFF_SZ] = c;
            self.buf_count += 1;
            true
        }
    }
}

// CONSTANTS

// Registers
static DR       : u32 = 0x000; // Data register, UARTDR on page 3-5
static RSR_ECR  : u32 = 0x004; // Receive status register/error clear register, UARTRSR/UARTECR on page 3-6
static FR       : u32 = 0x014; // Flag register, UARTFR on page 3-8
static ILPR     : u32 = 0x01C; // IrDA low-power counter register, UARTILPR on page 3-9
static IBRD     : u32 = 0x024; // Integer baud rate register, UARTIBRD on page 3-10
static FBRD     : u32 = 0x028; // Fractional baud rate register, UARTFBRD on page 3-10
static LCR_H    : u32 = 0x02C; // Line control register, UARTLCR_H on page 3-12
static CR       : u32 = 0x030; // Control register, UARTCR on page 3-15
static IFLS     : u32 = 0x034; // Interrupt FIFO level select register, UARTIFLS on page 3-17
static IMSC     : u32 = 0x038; // Interrupt mask set/clear register, UARTIMSC on page 3-17
static RIS      : u32 = 0x03C; // Raw interrupt status register, UARTRIS on page 3-19
static MIS      : u32 = 0x040; // Masked interrupt status register, UARTMIS on page 3-20
static ICR      : u32 = 0x044; // Interrupt cler register, UARTICR on page 3-21
static DMACR    : u32 = 0x048; // DMA control register, UARTDMACR on page 3-22
static PeriphID0: u32 = 0xFE0; // UARTPeriphID0 register on page 3-23
static PeriphID1: u32 = 0xFE4; // UARTPeriphID1 register on page 3-24
static PeriphID2: u32 = 0xFE8; // UARTPeriphID2 register on page 3-24
static PeriphID3: u32 = 0xFEC; // UARTPeriphID3 register on page 3-24
static PCellID0 : u32 = 0xFF0; // UARTPCellID0 register on page 3-25
static PCellID1 : u32 = 0xFF4; // UARTPCellID1 register on page 3-26
static PCellID2 : u32 = 0xFF8; // UARTPCellID2 register on page 3-26
static PCellID3 : u32 = 0xFFC; // UARTPCellID3 register on page 3-26

// Register masks
static FR_TXFE      : u32 = 1 << 7; // flag register; transmit FIFO empty
static FR_RXFF      : u32 = 1 << 6; // flag register; receive FIFO full
static FR_TXFF      : u32 = 1 << 5; // flag register; transmit FIFO full
static FR_RXFE      : u32 = 1 << 4; // flag register; receive FIFO empty
static CR_RXE       : u32 = 1 << 9; // control register; receive enable
static CR_TXE       : u32 = 1 << 8; // control register; transmit enable
static CR_EN        : u32 = 1 << 0; // control register; UART enable
static LCR_FIFOEN   : u32 = 1 << 4; // line control register; FIFO enable
static LCR_PEN      : u32 = 1 << 1; // line control register; parity enable
static LCR_EPS      : u32 = 1 << 2; // line control register; even parity select (odd default)
static IMSC_TXIM    : u32 = 1 << 5; // interrupt mask set/clear; transmit interrupt
static IMSC_RXIM    : u32 = 1 << 4; // interrupt mask set/clear; receive interrupt

enum WLEN
{
    LCR_WLEN_8 = 11 << 5,
    LCR_WLEN_7 = 10 << 5,
    LCR_WLEN_6 = 01 << 5,
    LCR_WLEN_5 = 00 << 5,
}


