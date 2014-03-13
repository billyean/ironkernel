/* drivers::arm1176jzf_s.rs */

use kernel::screen::Resolution;

pub fn init(_ : Resolution){
    unsafe{
        let p = gpio::Pin::get(16).get();
        p.setMode(gpio::OUTPUT);
        p.write(false);
    }

}

/// driver::arm1176jzf_s::mailman 
/// used for GPU communication
mod mailman{
    use super::super::super::io;

    pub static BASE : u32 = 0x2000B880;
    static READ_OFFSET	: u32	= 0x0;
    static POLL_OFFSET	: u32	= 0x10;
    static SENDER_OFFSET	: u32	= 0x14;
    static STATUS_OFFSET	: u32	= 0x18;
    static CONFIG_OFFSET	: u32	= 0x1C;
    static WRITE_OFFSET	: u32	= 0x20;

    static STATUS_READY_MASK	: u32	= 0x80000000;
    static STATUS_MSG_PENDING_MASK	: u32	= 0x40000000;

    type MailmanBoxNum = u8;

    static PWR_MGMT	    : MailmanBoxNum	= 0;
    static GPU	        : MailmanBoxNum	= 1;
    static VIRT_UART	: MailmanBoxNum	= 2;
    static VCHIQ	    : MailmanBoxNum	= 3;
    static LEDS	        : MailmanBoxNum	= 4;
    static BUTTONS	    : MailmanBoxNum	= 5;
    static TOUCH	    : MailmanBoxNum	= 6;
    static NOTHING	    : MailmanBoxNum	= 7;
    static PROP_TAGS_SEND	: MailmanBoxNum	= 8;
    static PROP_TAGS_RECV	: MailmanBoxNum	= 9;


    /// Reads from the appropriate mailbox. Discards other mailboxes' messages and will hang until message received.
    /// Returns (message, true) on success, (0, false) on user error.
    unsafe fn postman_read(mailbox : MailmanBoxNum) -> (u32, bool){
        let mut message : u32 = 0;
        loop{
            while(io::read(BASE + STATUS_OFFSET) & STATUS_MSG_PENDING_MASK != 0) {};
            message = io::read(BASE + READ_OFFSET);
            if((message & (0xF)) as u8 == (mailbox as u8)){ break; };
        }
        
        return (message ^ (message & 0xF), true); // equivalent to message & (not 0xF)
    }

    /// Reads from the appropriate mailbox. Will hang until message received.
    /// Returns true on success, false on user error.
    unsafe fn postman_write(mailbox : MailmanBoxNum, message : u32) -> bool{
        if(message & 0xF != 0){
            return false; //Error: message bigger than mailbox number
        }
        let sendmsg : u32 = (message ^ (message & 0xF)) | (mailbox as u32);

        while(io::read(BASE + STATUS_OFFSET) & STATUS_READY_MASK != 0) {};
        io::wh(BASE + WRITE_OFFSET, sendmsg);

        return true;
    }
}

/// driver::arm1176jzf_s::screen
/// Driver for communicating with GPU, drawing to screen
pub mod screen{
    use kernel::screen::*;
    use super::mailman;

    struct screen_buffer_info{
        /// requested / provided width of screen
        width       : u32,
        /// requested / provided width of screen
        height      : u32,
        v_width     : u32, 
        v_height    : u32,
        /// pitch between screen rows; provdided by graphics driver
        pitch       : u32,
        /// color depth. 16 for hicolor, 24 for truecolor, 32 for RGBA32
        depth       : u32, 
        /// x offset of display
        x           : u32, 
        /// y offset of screen
        y           : u32, 
        /// pointer to graphics area of memory
        pointer     : *mut u32, 
        /// Size of graphics buffer (bytes)
        size        : u32  
    }

    impl ScreenCanvas for screen_buffer_info{
        fn sync(&mut self) -> bool{
            true
        }

        fn getResolution(&self) -> Resolution  
        { 
            Resolution{w: self.width as uint, h :self.height as uint} 
        }

        fn setResolution(&mut self, res : Resolution) -> Resolution
        {
            let prev = self.getResolution();
            self.width      = res.w as u32;
            self.height     = res.h as u32;
            self.v_width    = res.w as u32;
            self.v_height   = res.h as u32;
            
            if(!self.sync()){
                self.width      = prev.w as u32;
                self.height     = prev.h as u32;
                self.v_width    = prev.w as u32;
                self.v_height   = prev.h as u32;

                self.sync();
            }
            self.getResolution()
        }

        fn getColorDepth(&self) -> ColorDepth
        {
            ColorDepth::from_uint(self.depth as uint)
        }

        fn setColorDepth(&mut self, newDepth : ColorDepth) -> ColorDepth
        {
            let previous : u32 = self.depth;
            self.depth = newDepth as u32;
            if(!self.sync()){
                self.depth = previous;
                self.sync();
            }
            self.getColorDepth()
        }

        fn drawPixel(&mut self, color: &Pixel, coords : &(uint, uint)) -> bool{ false }

        fn ready(&mut self) -> bool { true }
    }

        // Need screenbuf instance acquisition
}

pub mod gpio{
    use core::option::{Option, Some, None};
    use super::super::io::*;

    pub static mut BASE : u32 = 0x20200000;
    static PINID_MAX : uint = 54;
    static FSEL : u32 = 0;
    static OUTPUT_SET : u32 = 0x1C;
    static OUTPUT_CLR : u32 = 0x28;
    static OUTPUT_LVL : u32 = 0x34;

    pub struct Pin {
        id : uint
    }

//trait SARTPin : Pin;
//trait SPIPin : Pin;
    impl Pin{
        pub fn get(no : uint) -> Option<Pin>{
            match no {
                0 .. PINID_MAX => Some(Pin{id : no}),
                _ => None
            }
        }
       
        pub unsafe fn setMode(&self, mode : pin_mode) -> bool{
            // TODO: Validate modes on a per-pin basis

            let bank : u32 = BASE + FSEL + (self.id as u32 / 10) * 4; // offset from GPIO base
            let bank_shift : u32 = (self.id as u32 % 10) * 3;  // amt to shift within bank

            let prev_reg = read(bank);
            wh(bank, (prev_reg ^ (prev_reg & (7 << bank_shift))) | (mode as u32 << bank_shift));

            true
        }
        pub unsafe fn getMode(&self) -> Option<pin_mode>{
            let bank : u32 = BASE + FSEL + (self.id as u32 / 10) * 4; // offset from GPIO base
            let bank_shift : u32 = (self.id as u32 % 10) * 3;  // amt to shift within bank

            pin_mode::from_uint(((read(bank) >> bank_shift) & 7) as uint)
        }

        pub unsafe fn write(&self, value : bool) -> Option<bool>{
            match self.getMode().get() {
                OUTPUT => Some({
                    let bank = BASE // base GPIO address
                        + if(value){ OUTPUT_SET }else{ OUTPUT_CLR } // Which bank: set or clear
                        + if(self.id > 31){ 4 }else{ 0 }              // Second register if pin is in second bank
                        ;
                    let mask = 1 << (self.id % 32);
                    wh(bank, mask);
                    true
                }),
                _ => None
            }
        }

        pub unsafe fn read(&self) -> Option<bool>{
            match self.getMode().get() {
                OUTPUT | INPUT => Some({
                    let bank = BASE // base GPIO address
                        + OUTPUT_LVL
                        + if(self.id > 31){ 4 }else{ 0 }              // Second register if pin is in second bank
                        ;
                    let mask = 1 << (self.id % 32);
                    
                    if(0 == mask & read(bank)){
                        false
                    }else{
                        true
                    }
                    }),
                _ => None
            } // match
        } // read
    } //impl Pin

    /// IO mode for GPIO pins
    #[repr(C)]
    pub enum pin_mode{
        INPUT = 0,
        OUTPUT = 1,
        ALT0 = 4,
        ALT1 = 5,
        ALT2 = 6,
        ALT3 = 7,
        ALT4 = 3,
        ALT5 = 2
    }
    impl pin_mode{
        pub fn from_uint(x : uint) -> Option<pin_mode>{
            match x {
                0 => Some(INPUT),
                1 => Some(OUTPUT),
                4 => Some(ALT0),
                5 => Some(ALT1),
                6 => Some(ALT2),
                7 => Some(ALT3),
                3 => Some(ALT4),
                2 => Some(ALT5),
                _ => None
            }
        }
    }
}
