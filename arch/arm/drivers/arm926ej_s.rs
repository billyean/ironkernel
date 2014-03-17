/* drivers::arm926ej_s */

use core::mem;

use kernel;
use kernel::screen::*;
use core::mem::transmute;

/* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0225d/BBABEGGE.html */
/* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0225d/BBABEGGE.html */
pub static VIC_INT          : *mut u32 = (0x10140000) as *mut u32;
pub static VIC_INT_ENABLE   : *mut u32 = (0x10140000 + 0x10) as *mut u32;
pub static VIC_INT_DISABLE  : *mut u32 = (0x10140000 + 0x14) as *mut u32; // "enable clear"

pub mod screen
{
    use kernel::screen::*;
    use kernel::screen::font;
    use super::super::io::*;

    pub struct canvas{
        CURSOR : cursor     ,
        CURSOR_BUFFER: [u32, ..8*16],
        SAVE_X: u32         ,
        SAVE_Y: u32         ,
        START_ADDR: u32     ,
        SCREEN_WIDTH: u32   ,
        SCREEN_HEIGHT: u32  ,
    }

    pub static mut Screen0 : canvas = canvas{
            CURSOR : cursor{
                x      : 0,
                y      : 0,
                height : 8,
                width  : 16,
                cursor_color  : ARGBPixel(0, 0, 0, 0xFF),
                fg_color      : ARGBPixel(0, 0, 0, 0),
                bg_color      : ARGBPixel(0, 0xFF, 0xFF, 0xFF),
            },
            CURSOR_BUFFER   : [0x00FF0000, ..8*16],
            SAVE_X          : 0,
            SAVE_Y          : 0,
            START_ADDR      : 1024*1024,
            SCREEN_WIDTH    : 0,
            SCREEN_HEIGHT   : 0, 
    };

    impl ScreenCanvas for canvas
    {
        fn sync(&mut self) -> bool 
        {
            true 
        }

        fn setResolution(&mut self, res : Resolution) -> Resolution
        {
            self.SCREEN_WIDTH = res.w as u32;
            self.SCREEN_HEIGHT = res.h as u32;
            /*unsafe {
                sgash::init()
            };*/
            /* For the following magic values, see 
             * http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0225d/CACHEDGD.html
             */

            match res {
                WVGA2 => unsafe {
                    // 800x600
                    ws(0x10000010, 0x2CAC);
                    ws(0x10120000, 0x1313A4C4);
                    ws(0x10120004, 0x0505F657);
                    ws(0x10120008, 0x071F1800);

                    /* See http://forum.osdev.org/viewtopic.php?p=195000 */
                    ws(0x10120010, self.START_ADDR);
                    
                    /* See http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0161e/I911024.html */
                    ws(0x10120018, 0x82B);
                },               
                /*#[allow(unreachable_code)]                      
                VGA => unsafe {
                    // 640x480
                    // See http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0225d/CACCCFBF.html
                    ws(0x10000010, 0x2C77);
                    ws(0x10120000, 0x3F1F3F9C);
                    ws(0x10120004, 0x090B61DF);
                    ws(0x10120008, 0x067F1800);

                    /* See http://forum.osdev.org/viewtopic.php?p=195000 */
                    ws(0x10120010, self.START_ADDR);

                    /* See http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0161e/I911024.html */
                    ws(0x10120018, 0x82B);
                },
                _ => abort() 
                    */
            } // match resolution                

            self.getResolution()
        } // setResolution

        fn getResolution(&self) -> Resolution
        {
            Resolution{w: self.SCREEN_WIDTH as uint, h: self.SCREEN_HEIGHT as uint}
        }
        
        fn getColorDepth(&self) -> ColorDepth
        {
            ARGB
        }

        #[allow(unused_variable)]
        fn setColorDepth(&mut self, d : ColorDepth) -> ColorDepth
        {
            self.getColorDepth()
        }
        
        #[allow(unused_variable)]
        fn drawPixel(&mut self, color : &Pixel, coords : &(uint, uint)) -> bool
        {
            match *color {
                ARGBPixel(r, g, b, a) => unsafe { // Allow it: direct write
                    let (xx, yy) = *coords;
                    let (x, y) = (xx as u32, yy as u32);
                    let addr = self.START_ADDR + 4 * (x + y * self.SCREEN_WIDTH);
                    wh(addr, color.word());
                    true
                },
                _ => false
            }
        }
                
        fn ready(&mut self) -> bool
        {
            true
        }

    } // impl ScreenCanvas

    impl TerminalCanvas for canvas
    {
        fn getCursor(&self) -> cursor 
        {
            self.CURSOR
        }
        fn setCursor(&mut self, c : &cursor) -> cursor
        {
            self.CURSOR = *c;
            self.CURSOR
        }
        unsafe fn scrollup(&mut self)
        {
            let curHeight = self.CURSOR.height;
            let mut i = curHeight * self.SCREEN_WIDTH;
            while i < (self.SCREEN_WIDTH*self.SCREEN_HEIGHT)
            {
                *((self.START_ADDR + ((i-16*self.SCREEN_WIDTH)*4)) as *mut u32) = *((self.START_ADDR+(i*4)) as *u32); 
                i += 1;
            }
            i = 4*(self.SCREEN_WIDTH*self.SCREEN_HEIGHT - curHeight*self.SCREEN_WIDTH);
            while i < 4*self.SCREEN_WIDTH*self.SCREEN_HEIGHT
            {
                *((self.START_ADDR + (i)) as *mut u32) = self.CURSOR.bg_color.word();
                i += 4;
            }
            self.CURSOR.x = 0x0;
            self.CURSOR.y -= curHeight;
        }
        unsafe fn drawCharacter(&mut self, c: char) -> bool
        {
            if self.CURSOR.x +(self.SCREEN_WIDTH* (self.CURSOR.y)) >= self.SCREEN_WIDTH*self.SCREEN_HEIGHT
            {
                self.scrollup();
            }
            let font_offset = (c as u8) - 0x20;
            let map = font::bitmaps[font_offset];

            let mut i = -1;
            let mut j = 0;
            let mut addr = self.START_ADDR + 4*(self.CURSOR.x + self.CURSOR.width + 1 + self.SCREEN_WIDTH* (self.CURSOR.y));
            while j < self.CURSOR.height
            {
                while i < self.CURSOR.width
                {
                    //let addr = START_ADDR + 4*(CURSOR.x + CURSOR_WIDTH - i + SCREEN_WIDTH*(CURSOR.y + j));
                    //let addr = START_ADDR + 4*(CURSOR.x + CURSOR_WIDTH + SCREEN_WIDTH*CURSOR.y) - 4*i + 4*SCREEN_WIDTH*j
                    *(addr as *mut u32) = {
                        if ((map[j] >> 4*i) & 1) == 1
                        {
                            self.CURSOR.fg_color.word()
                        }
                        else
                        {
                            self.CURSOR.bg_color.word()
                        }
                    }; 
                    
                    addr-= 4;
                    i += 1;
                }
                addr += 4u32*(i + self.SCREEN_WIDTH);
                i = 0;
                j += 1;
            }
            true
        }
        
        unsafe fn backup(&mut self)
        {
            let mut i = 0;
            let mut j = 0;
            while j < self.CURSOR.height
            {
                while i < self.CURSOR.width
                {
                    let addr = self.START_ADDR + 4*((self.CURSOR.x + i) + self.SCREEN_WIDTH*((self.CURSOR.y + j)));
                    self.CURSOR_BUFFER[i + j*8] = *(addr as *mut u32);
                    i += 1;
                }
            i = 0;
            j += 1;
            }
            self.SAVE_X = self.CURSOR.x;
            self.SAVE_Y = self.CURSOR.y;
        }

        unsafe fn restore(&mut self)
        {
            let mut i = 0;
            let mut j = 0;
            while j < self.CURSOR.height
            {
                while i < self.CURSOR.width
                {
                    let addr = self.START_ADDR + 4*(self.SAVE_X + i + self.SCREEN_WIDTH*(self.SAVE_Y + j));
                    *(addr as *mut u32) = self.CURSOR_BUFFER[i + j*8];
                    i += 1;
                }
                i = 0;
                j += 1;
            }
        }

        unsafe fn drawCursor(&mut self)
        {
            let mut i = 0;
            let mut j = 0;

            while j < self.CURSOR.height
            {
                while i < self.CURSOR.width
                {
                    let addr = self.START_ADDR + 4*(self.CURSOR.x + i + self.SCREEN_WIDTH*(self.CURSOR.y + j));
                    *(addr as *mut u32) = self.CURSOR.cursor_color.word();
                    i += 1;
                }
                i = 0;
                j += 1;
            }

        }

    }

    impl canvas
    {
        pub unsafe fn paint(&mut self, color: u32)
        {
            let mut i = 0; 
            while i < self.SCREEN_WIDTH*self.SCREEN_HEIGHT
            {
                *((self.START_ADDR as u32 + i*4) as *mut u32) = color;
                i+=1;
            }
        }

        pub unsafe fn fill_bg(&mut self)
        {
            let word : u32 = self.CURSOR.bg_color.word();
            self.paint(word);
        }

        pub unsafe fn set_fg(&mut self, color: Pixel)
        {
            self.CURSOR.fg_color = color;
        }

        pub unsafe fn set_bg(&mut self, color: Pixel)
        {
            self.CURSOR.bg_color = color;
        }

        pub unsafe fn set_cursor_color(&mut self, color: Pixel)
        {
            self.CURSOR.cursor_color = color;
        }
    }
}


pub unsafe fn init(r : Resolution)
{
    let cv = &mut screen::Screen0;
    cv.sync();
    cv.setResolution(r);
    cv.set_bg(kernel::screen::ARGBPixel(0x00, 0x22, 0x2C, 0x38));
    cv.set_fg(kernel::screen::ARGBPixel(0x00, 0xFA, 0xFC, 0xFF));
    cv.set_cursor_color(kernel::screen::ARGBPixel(0x00, 0xFA, 0xFC, 0xFF));
    cv.fill_bg();
}

pub static UART_CLK : uint = 24_000_000; // 24 MHz

pub mod serial
{
    use kernel::serial::*;
    use kernel;
    use platform::cpu::interrupt;
    use core::mem::{volatile_load, volatile_store};
    use platform::io;
    use platform::drivers::pl011_uart::PL011;
    use platform::drivers::pl011_uart::UART_BUFF_SZ;

    pub static mut UART0 : PL011 = PL011 {
        base : 0x101f1000,
        IRQ : 12,
        receiver : UART0_receiveInterrupt,

        rate : 0,
        buffer : [0, .. UART_BUFF_SZ],
        buf_head : 0,
        buf_count : 0,
    }; 

#[no_mangle]
    unsafe fn UART0_receiveInterrupt() 
    { 
        let x = io::read(UART0.base as u32) as u8;
        UART0.receive(x);
        asm!("  pop {r11, lr}
                subs pc, r14, #4") // pc = lr - 4

    }
}
