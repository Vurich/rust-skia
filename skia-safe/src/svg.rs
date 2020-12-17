pub mod canvas;

use crate::{
    interop::RustStream,
    prelude::{NativeAccess, NativeDrop, NativeRefCounted},
    RCHandle,
};
use std::{error::Error, fmt, io};

pub use self::canvas::Canvas;

use skia_bindings as sb;

pub type SvgDom = RCHandle<sb::SkSVGDOM>;

impl NativeDrop for sb::SkSVGDOM {
    fn drop(&mut self) {}
}

impl NativeRefCounted for sb::SkSVGDOM {
    fn _ref(&self) {
        unsafe { sb::C_SkSVGDOM_ref(self) }
    }

    fn _unref(&self) {
        unsafe { sb::C_SkSVGDOM_unref(self) }
    }

    fn unique(&self) -> bool {
        unsafe { sb::C_SkSVGDOM_unique(self) }
    }
}

/// Error when something goes wrong when loading an SVG file. Sadly, Skia doesn't give further
/// details so we can't return a more expressive error type, but we still use this instead of
/// `Option` to express the intent and allow for `Try`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SvgLoadError;

impl fmt::Display for SvgLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to load animation (reason unknown)")
    }
}

impl Error for SvgLoadError {
    fn description(&self) -> &str {
        "Failed to load animation (reason unknown)"
    }
}

impl From<SvgLoadError> for io::Error {
    fn from(other: SvgLoadError) -> Self {
        io::Error::new(io::ErrorKind::Other, other)
    }
}

impl SvgDom {
    pub fn read<R: io::Read>(mut reader: R) -> Result<Self, SvgLoadError> {
        let mut reader = RustStream::new(&mut reader);

        let stream = reader.stream_mut();

        let out = unsafe { sb::C_SkSVGDOM_MakeFromStream(stream) };

        Self::from_ptr(out).ok_or(SvgLoadError)
    }

    /// Render this animation to a canvas, optionally specifying the location on the canvas that
    /// the animation should be rendered to.
    pub fn render(&self, canvas: &mut crate::Canvas) {
        unsafe { sb::SkSVGDOM::render(self.native() as &_, canvas.native_mut()) }
    }
}
