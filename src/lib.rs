#![deny(unsafe_op_in_unsafe_fn)]
#![deny(rust_2018_idioms)]

mod ffi;

use ffi::{IntoFfi, IntoNative};

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
fn parse<'a>(data: ffi::Slice<u8>) -> ffi::Box<goblin::pe::PE<'a>> {
    let data = unsafe { data.into_native() };
    Box::new(goblin::pe::PE::parse(data).unwrap()).into_ffi()
}

#[no_mangle]
fn parse_drop(data: ffi::Box<goblin::pe::PE<'_>>) {
    drop(unsafe { data.into_native() });
}

#[no_mangle]
fn parse_exports(pe: ffi::Box<goblin::pe::PE<'_>>) -> ffi::Box<ffi::Vec<ffi::Str>> {
    let vec = unsafe { pe.0.as_ref() }
        .unwrap()
        .exports
        .iter()
        .map(|export| export.name.unwrap_or("<none>").into_ffi())
        .collect::<Vec<_>>()
        .into_ffi();
    Box::new(vec).into_ffi()
}

#[no_mangle]
fn parse_imports(pe: ffi::Box<goblin::pe::PE<'_>>) -> ffi::Box<ffi::Vec<ffi::Str>> {
    let vec = unsafe { pe.0.as_ref() }
        .unwrap()
        .imports
        .iter()
        .flat_map(|import| [import.dll.into_ffi(), import.name.into_ffi()])
        .collect::<Vec<_>>()
        .into_ffi();
    Box::new(vec).into_ffi()
}

#[no_mangle]
fn box_vec_str_drop(data: ffi::Box<ffi::Vec<ffi::Str>>) {
    drop(unsafe { data.into_native().into_native() });
}
