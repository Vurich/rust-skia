use std::{
    ffi::{CStr, CString, OsStr},
    ops::{Deref, DerefMut},
    path::Path,
};

use crate::{prelude::*, Canvas, RCHandle, Rect};
use skia_bindings as sb;

#[repr(transparent)]
pub struct Builder(sb::skottie_Animation_Builder);

impl NativeTransmutable<sb::skottie_Animation_Builder> for Builder {}

impl Deref for Builder {
    type Target = sb::skottie_Animation_Builder;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Builder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

unsafe impl Send for Builder {}
unsafe impl Sync for Builder {}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe { self.destruct() }
    }
}

/// A [Lottie](https://lottiefiles.com/) animation.
pub type Animation = RCHandle<sb::skottie_Animation>;

bitflags::bitflags! {
    #[derive(Default)]
    pub struct RenderFlags: u32 {
        const SKIP_TOP_LEVEL_ISOLATION = sb::skottie_Animation_RenderFlag::kSkipTopLevelIsolation as _;
        const DISABLE_TOP_LEVEL_CLIPPING = sb::skottie_Animation_RenderFlag::kDisableTopLevelClipping as _;
    }
}

impl NativeRefCounted for sb::skottie_Animation {
    fn _ref(&self) {
        unsafe { sb::C_skottie_Animation_ref(self) }
    }

    fn _unref(&self) {
        unsafe { sb::C_skottie_Animation_unref(self) }
    }

    fn unique(&self) -> bool {
        unsafe { sb::C_skottie_Animation_unique(self) }
    }
}

impl Animation {
    /// Parse the supplied .lottie file data and return an animation. Returns `None` if the data is
    /// somehow invalid.
    ///
    /// Since Lottie files may reference external data, this function will also return `None` if
    /// the file requests an external resource. If you want to be able to load external files,
    /// see `Builder`.
    pub fn from_data(data: &[u8]) -> Option<Self> {
        Self::from_ptr(
            unsafe { sb::skottie_Animation::Make(data.as_ptr() as *const i8, data.len()) }.fPtr,
        )
    }

    /// Opens the .lottie file at the given path. This function must allocate in order to create
    /// a C string from the path, use `open_cstr` if you want to avoid this. Returns `None` if the
    /// file cannot be found or is somehow invalid.
    ///
    /// Since Lottie files may reference external data, this function will also return `None` if
    /// the file requests an external resource. If you want to be able to load external files,
    /// see `Builder`.
    pub fn open<P: AsRef<Path>>(path: P) -> Option<Self> {
        let path = CString::new(path.as_ref().to_string_lossy().into_owned().into_bytes())
            .expect("CString::new failed: path contains null bytes");

        Self::open_cstr(&path)
    }

    /// Opens the .lottie file at the given path (expressed as a C string).
    ///
    /// Since Lottie files may reference external data, this function will also return `None` if
    /// the file requests an external resource. If you want to be able to load external files,
    /// see `Builder`.
    pub fn open_cstr<P: AsRef<CStr>>(path: P) -> Option<Self> {
        Self::from_ptr(unsafe { sb::skottie_Animation::MakeFromFile(path.as_ref().as_ptr()) }.fPtr)
    }

    /// Render this animation to a canvas, optionally specifying the location on the canvas that
    /// the animation should be rendered to.
    pub fn render(&self, canvas: &mut Canvas, dst: impl Into<Option<Rect>>) {
        let dst = dst.into();

        unsafe {
            sb::skottie_Animation::render(
                self.native() as &_,
                canvas.native_mut(),
                dst.as_ref()
                    .map(|r| r.native() as *const _)
                    .unwrap_or(std::ptr::null()),
            )
        }
    }

    /// Render this animation to a canvas, optionally specifying the location on the canvas that
    /// the animation should be rendered to, and supplying flags affecting how the animation is
    /// rendered (see documentation for `RenderFlags`).
    pub fn render_with_flags(
        &self,
        canvas: &mut Canvas,
        dst: impl Into<Option<Rect>>,
        flags: RenderFlags,
    ) {
        let dst = dst.into();

        unsafe {
            sb::skottie_Animation::render1(
                self.native() as &_,
                canvas.native_mut(),
                dst.as_ref()
                    .map(|r| r.native() as *const _)
                    .unwrap_or(std::ptr::null()),
                flags.bits(),
            )
        }
    }
}
