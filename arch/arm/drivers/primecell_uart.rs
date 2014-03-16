/* platform::drivers::primecell_uart */
/* Implementation of Serial for ARM's PrimeCell UART chip (PL011) */
// See http://infocenter.arm.com/help/topic/com.arm.doc.ddi0183f/DDI0183.pdf
//
use kernel::serial;
use platform::drivers::chip;
use platform::io;

// TODO resizable buffers
static PrimeCell_BUF_SZ : uint = 1024

struct PrimeCell {
    // NB: Base addresses should be mutable, to allow for re-mapping 
    base : u32,
    IRQ : u32,
    
    rate : baud,

    priv buffer : [u8, .. PrimeCell_BUF_SZ],
    priv buf_head : uint,
    priv buf_count,

    // TODO proper receive handlers
}

impl Serial for PrimeCell{

    /// Initialize device and begin transmission. Returns true if device successfully opened.
    fn open(&mut self, r : u32) -> bool
    {
        unsafe{
            // Set baud rate
            // Using tables on page 3-11 of PrimeCell ref for typical baud rates
            let int_divisor : u16 = chip::UART_CLK / (16 * r);
            // f_divisor is actually only a 6-bit quantity
            let f_component : u64 = (chip::UART_CLK as u64 << 7) / ((16 * r) as u64 << 7) - int_divisor << 7 as u64;
            let f_divisor : u8 = f_component >> 1 + f_component & 1;

            if(int_divisor == 0 || int_divisor == (1 << 16) - 1 as u16))
            {
                return false;
            }

            // enable PrimeCell0 IRQ [4]
            *chip::VIC_INT_ENABLE = 1 << self.IRQ;
            // enable RXIM interrupt (interrupt on receive)
            /*
             * See
             * http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0183f/I54603.html
             */
            io::ws(self.base + IMSC, 1 << 4);
            // TODO: Force IRQ enable, with appropriate interrupt handler
            /*
            kernel::int_table.map(|t| {
                t.enable(interrupt::IRQ, PrimeCell0_receiveInterrupt);
            });
            */
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
            volatile_store(self.base + DR, c as u32);
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

impl PrimeCell
{
    fn receive(&mut self, c : u8) -> bool
    {
        if(self.buf_count == PrimeCell_BUF_SZ)
        {
            false
        }else
        {
            self.buffer[(self.buf_head + self.buf_count) % PrimeCell_BUF_SZ] = c;
            self.buf_count += 1;
            true
        }
    }
}

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


