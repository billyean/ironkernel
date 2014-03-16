/* kernel::sgash.rs */

use core::*;
use core::str::*;
use core::option::{Some, Option, None}; // Match statement
use core::iter::Iterator;
use kernel::*;
use kernel::screen::*;
use kernel::memory::Allocator;
use kernel::serial::*;

use kernel::shell::*;

//use super::super::platform::drivers::arm926ej_s;
//use super::super::platform::drivers::arm926ej_s::serial;

// TODO Make useable for non-static-lifetime'd variables.

pub struct SGASH{
    buffer : cstr,
    serial : Option<&'static mut Serial>,
    screen : Option<&'static mut TerminalCanvas>,
}

// TODO a proper impl
impl Shell for SGASH
{
    fn init(&mut self)
    {
        unsafe{
            self.buffer = cstr::new(256);
        }
        self.serial = None;
        self.screen = None;
    }

    fn attachToSerial(&mut self, s : &'static mut Serial) -> bool
    {
        match self.serial 
        {
            Some(_) => false,
            None => {
                let mut success = true;

                success = success && s.open(9600);
                success = success && s.addReceiveHandler(|c| {
                    self.input(c);
                });
                if(success){
                    self.serial = Some(s);
                };
                success
            }
        }
    }
    
    fn attachToScreen(&mut self, s : &'static mut TerminalCanvas) -> bool
    {
        match self.screen
        {
            Some(_) => false,
            None => {
                self.splash();
                self.screen = Some(s);
                true
            }
        }
    }
    
    fn input(&mut self, x : char) -> bool
    {
        let c = x as u8;
        // Set this to false to learn the keycodes of various keys!
        // Key codes are printed backwards because life is hard
            
        if (true) {
            match c { 
                13		=>	{ 
                            self.parse();
                            self.prompt(); 
                },
                127		=>	unsafe { 
                    if (self.buffer.delete_char()) { 
                        self.txStr(&"^H ^H");
                        self.backspace();
                    }
                },
                _		=>	unsafe{ 
                    if (self.buffer.add_char(c)) { 
                        //self.txChar(c as char);
                        self.drawchar(c as char);
                    }
                },
            }
        }
        else {
            self.keycode(x as u8);
        }
    	true
    }
    
    fn output(&mut self, s : &str) -> bool
    {
        self.drawStr(s);
        self.txStr(s);
        // TODO call outputHandlers
    	true
    }
   
    // TODO implement
    #[allow(unused_variable)]
    fn addInputHandler(&mut self, ih : shellInputHandler) -> bool
    {
    	false
    }

    // TODO implement
    #[allow(unused_variable)]
    fn addOutputHandler(&mut self, oh : shellOutputHandler) -> bool
    {
    	false
    }
}

impl SGASH
{
    fn txChar(&self, x : char)
    {
        match self.serial
        {
            None => (),  
            Some(ref uart) => { uart.write(x as u8);},
        };

    }
    fn txStr(&self, msg: &str)
    {
        match self.serial
        {
            Some(ref uart) => for c in slice::iter(as_bytes(msg)) {
            	uart.write(*c);
            },
            _ => ()
        }
    }

    // TODO evaluate use
    #[allow(dead_code)]
    fn txCstr(&self, s: cstr)
    {
        match self.serial
        {
            Some(ref uart) => unsafe {
                let mut p = s.p as uint;
                while *(p as *char) != '\0'
                {
                    uart.write(*(p as *u8));
                    p += 1;
                }
            },
            _ => {}
        }
    }

    fn drawStr(&self, msg: &str)
    {
        match self.screen
        {
            Some(ref scr) => {
                // TODO Why the awkward color changing thing going on here?
                // Just to indicate what function it's going through?
                let old_fg = scr.getCursor().fg_color;
                let mut x : u32 = 0x6699AAFF;
                for c in slice::iter(as_bytes(msg)) {
                    x = (x << 8) + (x >> 24); 
                    let mut cur = scr.getCursor();
                    cur.fg_color = screen::ARGBPixel(
                                (x >> 24) as u8,
                                (x >> 16) as u8,
                                (x >> 8)  as u8,
                                x       as u8
                        );
                    scr.setCursor(&cur);
                    self.drawchar(*c as char);
                }
                let mut cur = scr.getCursor();
                cur.fg_color = old_fg;
                scr.setCursor(&cur);
            },
            _ => ()
        }
    }

    fn drawchar(&self, x: char)
    {
        match self.screen
        {
            Some(ref scr) => {
                let res = scr.getResolution();
                let mut cur = scr.getCursor();
                if x == '\n' {
                    cur.y += cur.height;
                    cur.x = 0;
                    scr.setCursor(&cur);
                    return;
                } else if x == '\t' {
                    cur.x += cur.width * 4;
                    if cur.x >= res.w as u32 
                    {
                        cur.x -= res.w as u32;
                        cur.y += cur.height;
                    }
                    scr.setCursor(&cur);
                    return;
                }
                unsafe{
                    scr.restore();

                    scr.drawCharacter(x);
                    cur.x += cur.width;
                    if cur.x >= res.w as u32 
                    {
                        cur.x -= res.w as u32;
                        cur.y += cur.height;
                    }
                    scr.setCursor(&cur);
                    scr.backup();
                    scr.drawCursor();
                }
            },
            _ => ()
        }
    }
    
    fn backspace(&self)
    {
        match self.screen 
        {
            Some(ref scr) => unsafe {
                scr.restore();
                let mut cur = scr.getCursor();
                cur.x -= cur.width;
                scr.setCursor(&cur);
                scr.drawCharacter(' ');
                scr.backup();
                scr.drawCursor();
            },
            None => ()
        }
    }

    fn splash(&mut self) 
    {	
        self.output(&"\n                                                               "); 
        self.output(&"\n                                                               ");
        self.output(&"\n                       7=..~$=..:7                             "); 
        self.output(&"\n                  +$: =$$$+$$$?$$$+ ,7?                        "); 
        self.output(&"\n                  $$$$$$$$$$$$$$$$$$Z$$                        ");
        self.output(&"\n              7$$$$$$$$$$$$. .Z$$$$$Z$$$$$$                    ");
        self.output(&"\n           ~..7$$Z$$$$$7+7$+.?Z7=7$$Z$$Z$$$..:                 ");
        self.output(&"\n          ~$$$$$$$$7:     :ZZZ,     :7ZZZZ$$$$=                ");
        self.output(&"\n           Z$$$$$?                    .+ZZZZ$$                 ");
        self.output(&"\n       +$ZZ$$$Z7                         7ZZZ$Z$$I.            "); 
        self.output(&"\n        $$$$ZZZZZZZZZZZZZZZZZZZZZZZZI,    ,ZZZ$$Z              "); 
        self.output(&"\n      :+$$$$ZZZZZZZZZZZZZZZZZZZZZZZZZZZ=    $ZZ$$+~,           "); 
        self.output(&"\n     ?$Z$$$$ZZZZZZZZZZZZZZZZZZZZZZZZZZZZI   7ZZZ$ZZI           "); 
        self.output(&"\n      =Z$$+7Z$$7ZZZZZZZZ$$$$$$$ZZZZZZZZZZ  ~Z$?$ZZ?            ");	 
        self.output(&"\n    :$Z$Z...$Z  $ZZZZZZZ~       ~ZZZZZZZZ,.ZZ...Z$Z$~          "); 
        self.output(&"\n    7ZZZZZI$ZZ  $ZZZZZZZ~       =ZZZZZZZ7..ZZ$?$ZZZZ$          "); 
        self.output(&"\n      ZZZZ$:    $ZZZZZZZZZZZZZZZZZZZZZZ=     ~$ZZZ$:           "); 
        self.output(&"\n    7Z$ZZ$,     $ZZZZZZZZZZZZZZZZZZZZ7         ZZZ$Z$          "); 
        self.output(&"\n   =ZZZZZZ,     $ZZZZZZZZZZZZZZZZZZZZZZ,       ZZZ$ZZ+         "); 
        self.output(&"\n     ,ZZZZ,     $ZZZZZZZ:     =ZZZZZZZZZ     ZZZZZ$:           "); 
        self.output(&"\n    =$ZZZZ+     ZZZZZZZZ~       ZZZZZZZZ~   =ZZZZZZZI          "); 
        self.output(&"\n    $ZZ$ZZZ$$Z$$ZZZZZZZZZ$$$$   IZZZZZZZZZ$ZZZZZZZZZ$          "); 
        self.output(&"\n      :ZZZZZZZZZZZZZZZZZZZZZZ   ~ZZZZZZZZZZZZZZZZZ~            "); 
        self.output(&"\n     ,Z$$ZZZZZZZZZZZZZZZZZZZZ    ZZZZZZZZZZZZZZZZZZ~           "); 
        self.output(&"\n     =$ZZZZZZZZZZZZZZZZZZZZZZ     $ZZZZZZZZZZZZZZZ$+           "); 
        self.output(&"\n        IZZZZZ:.                        . ,ZZZZZ$              "); 
        self.output(&"\n       ~$ZZZZZZZZZZZ                 ZZZZ$ZZZZZZZ+             "); 
        self.output(&"\n           Z$ZZZ. ,Z~               =Z:.,ZZZ$Z                 "); 
        self.output(&"\n          ,ZZZZZ..~Z$.             .7Z:..ZZZZZ:                ");
        self.output(&"\n          ~7+:$ZZZZZZZZI=:.   .,=IZZZZZZZ$Z:=7=                ");
        self.output(&"\n              $$ZZZZZZZZZZZZZZZZZZZZZZ$ZZZZ                    ");
        self.output(&"\n              ==..$ZZZ$ZZZZZZZZZZZ$ZZZZ .~+                    "); 			
        self.output(&"\n                  I$?.?ZZZ$ZZZ$ZZZI =$7                        ");
        self.output(&"\n                       $7..I$7..I$,                            ");
        self.output(&"\n"); 
        self.output(&"\n _                     _     _                         _  ");
        self.output(&"\n| |                   (_)   | |                       | | ");
        self.output(&"\n| | ____ ___  ____     _____| |_____  ____ ____  _____| | ");
        self.output(&"\n| |/ ___) _ \\|  _ \\   |  _   _) ___ |/ ___)  _ \\| ___ | | ");
        self.output(&"\n| | |  | |_| | | | |  | |  \\ \\| ____| |   | | | | ____| | ");
        self.output(&"\n|_|_|  \\____/|_| |_|  |_|   \\_\\_____)_|   |_| |_|_____)__)\n\n");
    }
    
    fn prompt(&mut self) 
    {
        self.output(&"\nsgash > ");
        unsafe{ self.buffer.reset(); }
    }

    fn parse(&mut self) 
    {
        unsafe{
            if (self.buffer.streq(&"ls")) { 
                self.output( &"\na\tb");
            };
            match self.buffer.getarg(' ', 0) {
                Some(y)        => {
                    if(y.streq(&"cat")) {
                        match self.buffer.getarg(' ', 1) {
                        Some(x)        => {
                            if(x.streq(&"a")) { 
                                self.output( &"\nHello"); 
                            }
                            if(x.streq(&"b")) {
                                self.output( &"\nworld!");
                            }
                        }
                        None        => { }
                        };
                    }
                    if(y.streq(&"open")) {
                        self.output(&"\nTEST YO");
                    }
                }
                None        => { }
            };
            self.buffer.reset();
        }
    }

    fn keycode(&self, x: u8) 
    {
        let mut x = x;
        while  x != 0 {
            self.txChar((x%10+ ('0' as u8) ) as char);
            x = x/10;
        }
        self.txChar(' ');
    }
}

/* BUFFER MODIFICATION FUNCTIONS */

struct cstr {
	p: *mut u8,
	p_cstr_i: uint,
	max: uint 
}

impl cstr {
	pub unsafe fn new(size: uint) -> cstr 
    {
		// Sometimes this doesn't allocate enough memory and gets stuck...
		let (x, y) = heap.alloc(size);
		let this = cstr {
			p: x,
			p_cstr_i: 0,
			max: y
		};
		*(((this.p as uint)+this.p_cstr_i) as *mut char) = '\0';
		this
	}

	// TODO evaluate usefulness
    #[allow(dead_code)]
    unsafe fn from_str(s: &str) -> cstr 
    {
		let mut this = cstr::new(256);
		for c in slice::iter(as_bytes(s)) {
			this.add_char(*c);
		};
		this
	}

	#[allow(dead_code)]
    fn len(&self) -> uint { self.p_cstr_i }

	// HELP THIS DOESN'T WORK THERE IS NO GARBAGE COLLECTION!!!
	// -- TODO: exchange_malloc, exchange_free
    #[allow(dead_code)]
	unsafe fn destroy(&self) { heap.free(self.p); }

	unsafe fn add_char(&mut self, x: u8) -> bool
    {
		if (self.p_cstr_i == self.max) { return false; }
		*(((self.p as uint)+self.p_cstr_i) as *mut u8) = x;
		self.p_cstr_i += 1;
		*(((self.p as uint)+self.p_cstr_i) as *mut char) = '\0';
		true
	}

	unsafe fn delete_char(&mut self) -> bool 
    {
		if (self.p_cstr_i == 0) { return false; }
		self.p_cstr_i -= 1;
		*(((self.p as uint)+self.p_cstr_i) as *mut char) = '\0';
		true
	}

	unsafe fn reset(&mut self) 
    {
		self.p_cstr_i = 0; 
		*(self.p as *mut char) = '\0';
	}

    #[allow(dead_code)]
	unsafe fn eq(&self, other: &cstr) -> bool 
    {
		if (self.len() != other.len()) { return false; }
		else {
			let mut x = 0;
			let mut selfp: uint = self.p as uint;
			let mut otherp: uint = other.p as uint;
			while x < self.len() {
				if (*(selfp as *char) != *(otherp as *char)) { return false; }
				selfp += 1;
				otherp += 1;
				x += 1;
			}
			true
		}
	}

	unsafe fn streq(&self, other: &str) -> bool 
    {
		let mut selfp: uint = self.p as uint;
		for c in slice::iter(as_bytes(other)) {
			if( *c != *(selfp as *u8) ) { return false; }
			selfp += 1;
		};
		*(selfp as *char) == '\0'
	}

	unsafe fn getarg(&self, delim: char, mut k: uint) -> Option<cstr> 
    {
		let mut ind: uint = 0;
		let mut found = k == 0;
		let mut selfp: uint = self.p as uint;
		let mut s = cstr::new(256);
		loop {
			if (*(selfp as *char) == '\0') { 
				// End of string
				if (found) { return Some(s); }
				else { return None; }
			};
			if (*(selfp as *u8) == delim as u8) { 
				if (found) { return Some(s); }
				k -= 1;
			};
			if (found) {
				s.add_char(*(selfp as *u8));
			};
			found = k == 0;
			selfp += 1;
			ind += 1;
			if (ind == self.max) { 
				//self.output(&"\nSomething broke!");
				return None; 
			}
		}
	}

    #[allow(dead_code)]
	unsafe fn split(&self, delim: char) -> (cstr, cstr) 
    {
		let mut selfp: uint = self.p as uint;
		let mut beg = cstr::new(256);
		let mut end = cstr::new(256);
		let mut found = false;
		loop {
			if (*(selfp as *char) == '\0') { 
				return (beg, end);
			}
			else if (*(selfp as *u8) == delim as u8) {
				found = true;
			}
			else if (!found) {
				beg.add_char(*(selfp as *u8));
			}
			else if (found) {
				end.add_char(*(selfp as *u8));
			};
			selfp += 1;
		}
	}


}


// TODO
// pub fn new() -> ~Shell
