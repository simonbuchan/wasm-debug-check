#![deny(unsafe_op_in_unsafe_fn)]
#![deny(rust_2018_idioms)]

mod ffi;

use ffi::IntoNative;

#[no_mangle]
unsafe fn alloc(layout: ffi::Layout) -> *mut u8 {
    unsafe { std::alloc::alloc(layout.into_native()) }
}

#[no_mangle]
unsafe fn dealloc(layout: ffi::Layout, ptr: *mut u8) {
    unsafe { std::alloc::dealloc(ptr, layout.into_native()) };
}

mod console {
    use crate::ffi::IntoFfi;

    extern "C" {
        fn console_log(message: crate::ffi::Str);
        fn console_error(message: crate::ffi::Str);
    }

    pub fn log(msg: &str) {
        unsafe { console_log(msg.into_ffi()) }
    }

    pub fn error(msg: &str) {
        unsafe { console_error(msg.into_ffi()) }
    }
}

#[no_mangle]
fn init() {
    std::panic::set_hook(Box::new(|info| {
        console::error(&info.to_string());
    }));
}

#[no_mangle]
fn parse(data: &[u8]) {
    console::log(&format!("received {} bytes!", data.len()));
    let pe = goblin::pe::PE::parse(data).unwrap();
    for export in pe.exports {
        console::log(export.name.unwrap_or("<unnamed>"));
    }
}
