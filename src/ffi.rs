pub trait IntoNative {
    type Output<'a>;

    unsafe fn into_native<'a>(self) -> Self::Output<'a>;
}

pub trait IntoFfi {
    type Output;

    fn into_ffi(self) -> Self::Output;
}

#[repr(C)]
pub struct Slice<T>(pub *const T, pub usize);

impl<T> IntoNative for Slice<T>
where
    T: 'static,
{
    type Output<'a> = &'a [T];

    unsafe fn into_native<'a>(self) -> Self::Output<'a> {
        unsafe { std::slice::from_raw_parts(self.0, self.1) }
    }
}

impl<T> IntoFfi for &[T] {
    type Output = Slice<T>;

    fn into_ffi(self) -> Self::Output {
        Slice(self.as_ptr(), self.len())
    }
}

#[repr(transparent)]
pub struct Str(pub Slice<u8>);

impl IntoNative for Str {
    type Output<'a> = &'a str;

    unsafe fn into_native<'a>(self) -> Self::Output<'a> {
        std::str::from_utf8(unsafe { self.0.into_native() }).unwrap()
    }
}

impl<'a> IntoFfi for &'a str {
    type Output = Str;

    fn into_ffi(self) -> Self::Output {
        Str(self.as_bytes().into_ffi())
    }
}

#[repr(C)]
pub struct Layout {
    size: usize,
    align: usize,
}

impl IntoNative for Layout {
    type Output<'a> = std::alloc::Layout;

    unsafe fn into_native<'a>(self) -> Self::Output<'a> {
        std::alloc::Layout::from_size_align(self.size, self.align).unwrap()
    }
}

impl IntoFfi for std::alloc::Layout {
    type Output = Layout;

    fn into_ffi(self) -> Self::Output {
        Layout {
            size: self.size(),
            align: self.align(),
        }
    }
}
