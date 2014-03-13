/* driver::mod.rs */

use super::cpu::interrupt;
use super::io;
use core::option::{Option, None};
use kernel;

// See http://static.rust-lang.org/doc/master/rust.html#conditional-compilation
#[cfg(target_chip = "arm1176jzf-s")]
pub use chip = self::arm1176jzf_s;
#[cfg(target_chip = "arm926ej-s")]
pub use chip = self::arm926ej_s;

#[cfg(target_chip = "arm1176jzf-s")]
mod arm1176jzf_s;
#[cfg(target_chip = "arm926ej-s")]
mod arm926ej_s;

pub fn init() {

    
    // removed 926-specific initialization
    /*
     * unsafe {
        kernel::int_table.map(|t| {
	// See http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0235c/index.html
            t.enable(interrupt::IRQ, keypress);
        });
    }
    */
    unsafe {
        chip::init(kernel::screen::Resolutions::VGA);
    }
}

pub static mut keydown: Option<extern unsafe fn(char)> = None;
pub static mut read_char: Option<extern fn()->char> = None;

/*
#[no_mangle]
pub unsafe fn keypress() {
	keydown.map(|f| {
		let x = (*self::arm926ej_s::serial::UART0.base) as u8 as char;
		f(x)
	}
	);
    // Exception return instruction. [8]
    // TODO: better interrupt handler. r11 could change
    asm!("pop {r11, lr}
          subs pc, r14, #4") // pc = lr - 4
}*/
