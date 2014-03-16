/* kernel::serial */
/* Serial API for UART devices */

pub mod font;

//#[deriving(FromPrimative)]
pub enum ColorDepth{
    NoColor = 0,
    BW = 1,
    HighColor = 16,
    TrueColor = 24,
    ARGB = 32,
}

impl ColorDepth{
    pub fn from_uint(x : uint) -> ColorDepth{
        match x{
            1   => BW,
            16  => HighColor,
            24  => TrueColor,
            32  => ARGB,
            _   => NoColor
        }
    }
}

pub struct Resolution {w : uint, h : uint}

pub mod Resolutions{
    use super::Resolution;
    pub static NO_DISPLAY : Resolution = Resolution{w:0,	h:0};

    pub static QQVGA 	: Resolution = Resolution{w:160,	h:120};
    pub static HQVGA 	: Resolution = Resolution{w:240,	h:160};
    pub static QVGA 	: Resolution = Resolution{w:320,	h:240};
    pub static WQVGA1 	: Resolution = Resolution{w:360,	h:240};
    pub static WQVGA2 	: Resolution = Resolution{w:400,	h:240};
    pub static HVGA 	: Resolution = Resolution{w:480,	h:320};
    pub static VGA 	    : Resolution = Resolution{w:640,	h:480};
    pub static WVGA1 	: Resolution = Resolution{w:720,	h:480};
    pub static WVGA2 	: Resolution = Resolution{w:800,	h:480};
    pub static FWVGA    : Resolution = Resolution{w:854,	h:480};
    pub static SVGA 	: Resolution = Resolution{w:800,	h:600};
    pub static DVGA 	: Resolution = Resolution{w:960,	h:640};
    pub static WSVGA1   : Resolution = Resolution{w:1024,	h:576};
    pub static WSVGA2 	: Resolution = Resolution{w:1024,	h:600};

    pub static XGA 	    : Resolution = Resolution{w:1024,	h:768};
    pub static WXGA1 	: Resolution = Resolution{w:1152,	h:768};
    pub static WXGA2 	: Resolution = Resolution{w:1280,	h:720};
    pub static WXGA3 	: Resolution = Resolution{w:1280,	h:768};
    pub static WXGA4 	: Resolution = Resolution{w:1280,	h:800};
    pub static WXGA5 	: Resolution = Resolution{w:1360,	h:768};
    pub static WXGA6 	: Resolution = Resolution{w:1366,	h:768};
    pub static XGAp 	: Resolution = Resolution{w:1152,	h:864};
    pub static WXGAp 	: Resolution = Resolution{w:1440,	h:900};
    pub static WSXGA 	: Resolution = Resolution{w:1440,	h:960};
    pub static SXGA 	: Resolution = Resolution{w:1280,	h:1024};
    pub static SXGAp 	: Resolution = Resolution{w:1400,	h:1050};
    pub static WSXGAp 	: Resolution = Resolution{w:1680,	h:1050};
    pub static UXGA 	: Resolution = Resolution{w:1600,	h:1200};
    pub static WUXGA 	: Resolution = Resolution{w:1920,	h:1200};

    pub static nHD 	: Resolution = Resolution{w:640,	h:360};
    pub static qHD 	: Resolution = Resolution{w:960,	h:540};
    pub static HD 	: Resolution = Resolution{w:1280,	h:720};
    pub static HDp 	: Resolution = Resolution{w:1600,	h:900};
    pub static FHD 	: Resolution = Resolution{w:1920,	h:1080};
    pub static QHD 	: Resolution = Resolution{w:2560,	h:1440};
    pub static WQXGAp :Resolution = Resolution{w:3200,	h:1800};
    pub static UHD 	: Resolution = Resolution{w:3840,	h:2160};

    pub static Pebble : Resolution = Resolution{w:144,	h:168};
}

pub enum Pixel{
    NoColorPixel(),
    BWPixel(bool),
    HighColorPixel(u8, u8, u8),
    TrueColorPixel(u8, u8, u8),
    ARGBPixel(u8,u8,u8,u8)
}

impl Pixel{
    // TODO allow other color modes
    pub fn new(c : ColorDepth) -> Pixel{
        match c {
            BWColor   => BWPixel(false),
            /*HighColor  => HighColorPixel(0,0,0),
            TrueColor  => TrueColorPixel(0,0,0),
            ARGB  => ARGBPixel(0,0,0,0),
            _   => NoColorPixel*/
        }
    }
    // TODO : rgb as methods or as named properties
    
    pub fn word(&self) -> u32 {
        match self {
            &BWPixel(v)   => if v { -1 } else { 0 },
            &HighColorPixel(r,g,b) => ((r & 31) as u32 << 11) | ((g & 63) as u32 << 5) | ((b & 31) as u32),
            &TrueColorPixel(r,g,b) => (r as u32 << 16) | (g as u32 << 8) | (b as u32),
            &ARGBPixel(a,r,g,b) => (a as u32 << 24) | (r as u32 << 16) | (g as u32 << 8) | (b as u32),
            &NoColorPixel() => 0 as u32,
        }

    }
}

pub struct cursor{
    x        : u32,
    y        : u32,
    height   : u32,
    width    : u32,
    cursor_color    : Pixel,
    fg_color        : Pixel,
    bg_color        : Pixel
}

pub trait ScreenCanvas{
    /// Sync with hardware device.
    fn sync(&mut self) -> bool;
    /// Set Resolution. Returns actual Resolution available.
    fn setResolution(&mut self, Resolution) -> Resolution;
    fn getResolution(&self) -> Resolution;
    /// Set color depth. Returns actual color depth available.
    fn setColorDepth(&mut self, ColorDepth) -> ColorDepth;
    fn getColorDepth(&self) -> ColorDepth;

    fn drawPixel(&mut self, &Pixel, &(uint,uint)) -> bool;

    fn clear(&mut self, c : &Pixel) -> bool{
        let res = self.getResolution();
        let mut j = 0;
        let mut ok = true;
        while j < res.h
        {
            let mut i = 0;
            while i < res.w
            {
                ok = ok && self.drawPixel(c, &(i, j));
                i += 1;
            }
            j += 1;
        }
        ok
    }

    /// Check if the device is available
    fn ready(&mut self) -> bool;
}



pub trait TerminalCanvas : ScreenCanvas {
    fn getCursor(&self) -> cursor;
    fn setCursor(&mut self, &cursor) -> cursor;
    unsafe fn scrollup(&mut self);
    unsafe fn drawCharacter(&mut self, c : char) -> bool
    {
        let font_offset = (c as u8) - 0x20; // ' ' in ASCII
        let res = self.getResolution();
        let cursor = self.getCursor();
        // TODO this was based on the 976 implementation - reevaluate validity
        if cursor.x+(res.w as u32 *cursor.y) >= (res.w*res.h) as u32
        {
            self.scrollup();
        }
        let map = self::font::bitmaps[font_offset];

        let mut i = 0;
        let mut j = 0;
        let mut ok = true;
        while j < cursor.height
        {
            while i < cursor.width
            {
                //let addr = START_ADDR + 4*(CURSOR.x + CURSOR_WIDTH - i + SCREEN_WIDTH*(CURSOR.y + j));
                //let addr = START_ADDR + 4*(CURSOR.x + CURSOR_WIDTH + SCREEN_WIDTH*CURSOR.y) - 4*i + 4*SCREEN_WIDTH*j
                let color : Pixel = 
                    if ((map[j] >> 4*i) & 1) == 1
                    {
                        cursor.fg_color
                    }
                    else
                    {
                        cursor.bg_color
                    };
                ok = ok && self.drawPixel(&color, &((cursor.x + i) as uint, (cursor.y + j) as uint));
                i += 1;
            }
            i = 0;
            j += 1;
        }
        ok
    }


    unsafe fn backup(&mut self);
    unsafe fn restore(&mut self);
    unsafe fn drawCursor(&mut self);
}


