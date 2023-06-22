pub trait IntoNative {
    type Output<'a>;

    unsafe fn into_native<'a>(self) -> Self::Output<'a>;
}

pub trait IntoFfi {
    type Output;

    fn into_ffi(self) -> Self::Output;
}

#[repr(C)]
pub struct Slice<T> {
    pub ptr: *const T,
    pub len: usize,
}

impl<T> IntoNative for Slice<T>
where
    T: 'static,
{
    type Output<'a> = &'a [T];

    unsafe fn into_native<'a>(self) -> Self::Output<'a> {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<T> IntoFfi for &[T] {
    type Output = Slice<T>;

    fn into_ffi(self) -> Self::Output {
        Slice {
            ptr: self.as_ptr(),
            len: self.len(),
        }
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
    pub size: usize,
    pub align: usize,
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

type StdBox<T> = std::boxed::Box<T>;

#[repr(transparent)]
pub struct Box<T: ?Sized>(pub *mut T);

impl<T: ?Sized> IntoNative for Box<T> {
    type Output<'a> = StdBox<T>;

    unsafe fn into_native<'a>(self) -> Self::Output<'a> {
        unsafe { StdBox::from_raw(self.0) }
    }
}

impl<T: ?Sized> IntoFfi for StdBox<T> {
    type Output = Box<T>;

    fn into_ffi(self) -> Self::Output {
        Box(StdBox::into_raw(self))
    }
}

type StdString = std::string::String;

#[repr(transparent)]
pub struct String(pub Box<str>);

impl IntoNative for String {
    type Output<'a> = StdString;

    unsafe fn into_native<'a>(self) -> Self::Output<'a> {
        unsafe { StdString::from(self.0.into_native()) }
    }
}

impl IntoFfi for StdString {
    type Output = String;

    fn into_ffi(self) -> Self::Output {
        String(self.into_boxed_str().into_ffi())
    }
}

type StdVec<T> = std::vec::Vec<T>;

#[repr(transparent)]
pub struct Vec<T>(pub Box<[T]>);

impl<T> IntoNative for Vec<T> {
    type Output<'a> = StdVec<T>;

    unsafe fn into_native<'a>(self) -> Self::Output<'a> {
        unsafe { StdVec::from(self.0.into_native()) }
    }
}

impl<T> IntoFfi for StdVec<T> {
    type Output = Vec<T>;

    fn into_ffi(self) -> Self::Output {
        Vec(self.into_boxed_slice().into_ffi())
    }
}
