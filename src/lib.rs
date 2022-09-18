#![feature(allocator_api)]
#![feature(layout_for_ptr)]
#![feature(new_uninit)]
#![feature(pointer_is_aligned)]

pub mod errors;
pub mod fs;
pub mod handle;
mod buffer;