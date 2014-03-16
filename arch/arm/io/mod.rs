/* io::mod.rs */

use core::mem::{volatile_store, volatile_load};

pub unsafe fn read(addr: u32)	->	u32
{
    volatile_load(addr as *u32)
    //*(addr as *mut u32)
}

/// io::ws - write-set, set value's bits in ws
pub unsafe fn ws(addr: u32, value: u32)
{
    volatile_store(addr as *mut u32, volatile_load(addr as *u32) | value)
    //*(addr as *mut u32) = *(addr as *mut u32) | value;
}

/// io::wh - write-hard, overwrite existing address 
pub unsafe fn wh(addr: u32, value: u32)
{
    volatile_store(addr as *mut u32, value);
    //*(addr as *mut u32) = value;
}


