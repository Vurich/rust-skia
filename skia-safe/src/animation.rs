#![deny(missing_docs)]

//! 2D vector animations using the [Lottie](https://lottiefiles.com/) file format. These can be rendered like
//! any image into the scene.

use std::{
    error::Error,
    ffi::{CStr, CString},
    fmt, io,
    ops::{Deref, DerefMut},
    path::Path,
};

use crate::{interop::RustStream, prelude::*, Canvas, FontMgr, RCHandle, Rect, Size};
use skia_bindings as sb;

bitflags::bitflags! {
    /// Flags that affect behavior for when a [Builder] loads an [Animation].
    #[derive(Default)]
    pub struct BuilderFlags: u32 {
        /// Normally, any static images are resolved at load time. This defers loading of images to
        /// when you call `Animation::seek_frame`/`Animation::seek_time`.
        const DEFER_IMAGE_LOADING = sb::skottie_Animation_Builder_Flags_kDeferImageLoading as _;
        /// By default Skia will use native typefaces when possible, but supplying this flag will cause
        /// Skia to use the fallback glyph paths by default.
        const PREFER_EMBEDDED_FONTS = sb::skottie_Animation_Builder_Flags_kPreferEmbeddedFonts as _;
    }
}

/// Loader for [Animation], which allows you to supply the types necessary to load fonts
/// and external assets, as well as allowing access to more advanced settings and hooks
/// for affecting loading.
///
/// For simple files you can simply use `Animation::open` or `Animation::from_data`.
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

impl Builder {
    /// Initialize a new animation builder with default flags.
    pub fn new() -> Self {
        Self::new_with_flags(Default::default())
    }

    /// Initialize a new animation builder, setting loading flags (see [BuilderFlags]).
    pub fn new_with_flags(flags: BuilderFlags) -> Self {
        Self(unsafe { sb::skottie_Animation_Builder::new(flags.bits()) })
    }

    /// Set the font manager that will be used to load fonts for any text used in the animation.
    pub fn with_font_manager(&mut self, font_manager: FontMgr) -> &mut Self {
        unsafe {
            self.setFontManager(font_manager.into());
        }

        self
    }

    /// Parse the supplied .lottie file data and return an animation. Returns [None] if the data is
    /// somehow invalid.
    ///
    /// Since Lottie files may reference external data, this function will also return [None] if
    /// the file requests an external resource. If you want to be able to load external files,
    /// see [Builder].
    pub fn from_data(&mut self, data: &[u8]) -> Option<Animation> {
        Animation::from_ptr(unsafe { self.make1(data.as_ptr() as *const _, data.len()) }.fPtr)
    }

    /// Opens the .lottie file at the given path (expressed as a C string).
    ///
    /// Since Lottie files may reference external data, this function will also return [None] if
    /// the file requests an external resource. If you want to be able to load external files,
    /// see [Builder].
    pub fn open_cstr<P: AsRef<CStr>>(&mut self, path: P) -> Option<Animation> {
        Animation::from_ptr(unsafe { self.makeFromFile(path.as_ref().as_ptr()) }.fPtr)
    }

    /// Opens the .lottie file at the given path. This function must allocate in order to create
    /// a C string from the path, use `open_cstr` if you want to avoid this. Returns [None] if the
    /// file cannot be found or is somehow invalid.
    ///
    /// Since Lottie files may reference external data, this function will also return [None] if
    /// the file requests an external resource. If you want to be able to load external files,
    /// see [Builder].
    pub fn open<P: AsRef<Path>>(&mut self, path: P) -> Option<Animation> {
        let path = CString::new(path.as_ref().to_string_lossy().into_owned().into_bytes())
            .expect("CString::new failed: path contains null bytes");

        self.open_cstr(&path)
    }
}

bitflags::bitflags! {
    /// Flags related to rendering an animation (distinct from flags related to loading an animation, see
    /// [BuilderFlags]).
    #[derive(Default)]
    pub struct RenderFlags: u32 {
        /// For animations with non-trivial blending that cannot be directly composited into the scene,
        /// they are usually automatically composited into a separate layer, which is then composited
        /// into the scene separately. Enabling this flag causes the animation to always be directly
        /// composited into the scene. This is faster, but can lead to incorrect results.
        const SKIP_TOP_LEVEL_ISOLATION = sb::skottie_Animation_RenderFlag::kSkipTopLevelIsolation as _;
        /// By default, the animation is automatically clipped to its bounds as specified in the file.
        /// Enabling this flag will prevent the animation from being clipped at all, which allows it to
        /// draw outside its specified bounds.
        const DISABLE_TOP_LEVEL_CLIPPING = sb::skottie_Animation_RenderFlag::kDisableTopLevelClipping as _;
    }
}

/// A [Lottie](https://lottiefiles.com/) animation. If you need more advanced loading (such as automatically
/// loading external assets) see [Builder].
///
/// ```rust,ignore
/// let mut anim = Animation::open("data/my-animation.json");
///
/// let time = anim.duration() / 2.;
///
/// anim.seek_time::<()>(time);
///
/// {
///     let canvas = surface.canvas();
///     anim.draw(canvas, None);
/// }
/// ```
pub type Animation = RCHandle<sb::skottie_Animation>;

impl NativeDrop for sb::skottie_Animation {
    fn drop(&mut self) {
        unsafe {
            self.destruct();
        }
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

/// Regions that would be drawn to by `Animation::render` after the most-recent `Animation::seek_frame`
/// or `Animation::seek_time`.
pub struct DirtyRegion(sb::sksg_InvalidationController);

impl Default for DirtyRegion {
    fn default() -> Self {
        Self::new()
    }
}

impl DirtyRegion {
    fn new() -> Self {
        Self(unsafe { sb::sksg_InvalidationController::new() })
    }

    /// The bounding box of the region that would be dirtied by the change from the previous frame
    /// to the one that was just seeked to. This is relative to the animation, and if the animation
    /// would be transformed then you should transform these bounds to get the final bounding box.
    pub fn bounds(&self) -> Rect {
        self.0.fBounds.into()
    }
}

/// > **Note**: This is internal, and is only documented for the sake of completeness.
///
/// A possible result for `Animation::seek_frame` and `Animation::seek_time`. These functions
/// can optionally mark regions that would be made dirty, but instead of an optional, mutable
/// argument we instead use generic return types to capture this.
///
/// This trait is `unsafe` because the implementor must ensure that `as_invalidation_controller_ptr_mut`
/// returns a pointer that is valid for use in `Animation::seek_frame` or `Animation::seek_time`. The
/// definition of exactly what that means is left undefined for now, and this trait can be considered
/// internal.
pub unsafe trait SeekResult: Default {
    /// > **Note**: This is internal, and is only documented for the sake of completeness.
    ///
    /// Turn this value into a pointer that can be used in `seek_frame` or `seek_time`. See
    /// trait documentation.
    fn as_invalidation_controller_ptr_mut(&mut self) -> *mut sb::sksg_InvalidationController;
}

unsafe impl SeekResult for () {
    fn as_invalidation_controller_ptr_mut(&mut self) -> *mut sb::sksg_InvalidationController {
        std::ptr::null_mut()
    }
}

unsafe impl SeekResult for DirtyRegion {
    fn as_invalidation_controller_ptr_mut(&mut self) -> *mut sb::sksg_InvalidationController {
        &mut self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct AnimationLoadError;

impl fmt::Display for AnimationLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to load animation (reason unknown)")
    }
}

impl Error for AnimationLoadError {
    fn description(&self) -> &str {
        "Failed to load animation (reason unknown)"
    }
}

impl From<AnimationLoadError> for io::Error {
    fn from(other: AnimationLoadError) -> Self {
        io::Error::new(io::ErrorKind::Other, other)
    }
}

impl Animation {
    /// Parse the supplied .lottie file data and return an animation. Returns [None] if the data is
    /// somehow invalid.
    ///
    /// Since Lottie files may reference external data, this function will also return [None] if
    /// the file requests an external resource. If you want to be able to load external files,
    /// see [Builder].
    pub fn from_data(data: &[u8]) -> Result<Self, AnimationLoadError> {
        Self::from_ptr(unsafe {
            sb::C_skottie_Animation_MakeFromData(data.as_ptr() as *const _, data.len())
        })
        .ok_or(AnimationLoadError)
    }

    /// Load the animation from an arbitrary stream.
    pub fn read<R: io::Read>(mut reader: R) -> Result<Self, AnimationLoadError> {
        let mut reader = RustStream::new(&mut reader);

        let stream = reader.stream_mut();

        let out = unsafe { sb::C_skottie_Animation_MakeFromStream(stream) };

        Self::from_ptr(out).ok_or(AnimationLoadError)
    }

    /// Opens the .lottie file at the given path (expressed as a C string).
    ///
    /// Since Lottie files may reference external data, this function will also return [None] if
    /// the file requests an external resource. If you want to be able to load external files,
    /// see [Builder].
    pub fn open_cstr<P: AsRef<CStr>>(path: P) -> Result<Self, AnimationLoadError> {
        Self::from_ptr(unsafe { sb::C_skottie_Animation_MakeFromFile(path.as_ref().as_ptr()) })
            .ok_or(AnimationLoadError)
    }

    /// Opens the .lottie file at the given path. This function must allocate in order to create
    /// a C string from the path, use `open_cstr` if you want to avoid this. Returns [None] if the
    /// file cannot be found or is somehow invalid.
    ///
    /// Since Lottie files may reference external data, this function will also return [None] if
    /// the file requests an external resource. If you want to be able to load external files,
    /// see [Builder].
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, AnimationLoadError> {
        let path = CString::new(path.as_ref().to_string_lossy().into_owned().into_bytes())
            .expect("CString::new failed: path contains null bytes");

        Self::open_cstr(&path).ok_or(AnimationLoadError)
    }

    /// Get the animation's duration, in seconds.
    pub fn duration(&self) -> f64 {
        self.native().fDuration
    }

    /// Get the animation's duration, in frames.
    pub fn num_frames(&self) -> f64 {
        self.duration() * self.fps()
    }

    /// Get the framerate of this animation, in frames per second.
    pub fn fps(&self) -> f64 {
        self.native().fFPS
    }

    /// Get the size of this animation, in pixels. This is the size of the whole animation, and
    /// not the bounding box of a single frame. To get the bounding box of a single frame, seek
    /// to it with either `seek_frame` or `seek_time` and extract the `DirtyRegion`.
    pub fn size(&self) -> Size {
        Size::new(self.native().fSize.fWidth, self.native().fSize.fHeight)
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
    /// rendered (see documentation for [RenderFlags]).
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

    /// Seek to the specified frame. Inputs with fractional components (such as 0.5, 1.2) will show the
    /// interpolated frame between the closest whole keyframes before and after. A frame greater than
    /// the number of frames in the animation will seek to the final frame.
    ///
    /// This function can optionally return a [DirtyRegion], see that type's documentation for what this
    /// means. If in doubt, keep with the default return type of `()`.
    ///
    /// ### Discard the dirty region
    ///
    /// ```rust,no_run
    /// # use skia_safe::animation::Animation;
    ///
    /// let mut anim = Animation::open("data/my-animation.json").unwrap();
    ///
    /// anim.seek_frame::<()>(0.);
    /// ```
    ///
    /// ### Calculate the dirty region
    ///
    /// ```rust,no_run
    /// # use skia_safe::animation::{Animation, DirtyRegion};
    ///
    /// let mut anim = Animation::open("data/my-animation.json").unwrap();
    ///
    /// let region = anim.seek_frame::<DirtyRegion>(0.);
    ///
    /// println!("{:?}", region.bounds());
    /// ```
    pub fn seek_frame<O: SeekResult>(&mut self, frame: f64) -> O {
        let mut out = O::default();

        unsafe {
            self.native_mut()
                .seekFrame(frame, out.as_invalidation_controller_ptr_mut());
        }

        out
    }

    /// Seek to the specified time, in seconds, in the animation. If the time is greater than
    /// `self.duration()`, then it will seek to the closest frame (i.e. the final frame). To
    /// loop the animation, seek to `time % duration`.
    ///
    /// This function can optionally return a [DirtyRegion], see that type's documentation for what this
    /// means. If in doubt, keep with the default return type of `()`.
    ///
    /// ### Discard the dirty region
    ///
    /// ```rust,no_run
    /// # use skia_safe::animation::Animation;
    ///
    /// let mut anim = Animation::open("data/my-animation.json").unwrap();
    ///
    /// anim.seek_time::<()>(0.);
    /// ```
    ///
    /// ### Calculate the dirty region
    ///
    /// ```rust,no_run
    /// # use skia_safe::animation::{Animation, DirtyRegion};
    ///
    /// let mut anim = Animation::open("data/my-animation.json").unwrap();
    ///
    /// let region = anim.seek_time::<DirtyRegion>(0.);
    ///
    /// println!("{:?}", region.bounds());
    /// ```
    pub fn seek_time<O: SeekResult>(&mut self, time: f64) -> O {
        let mut out = O::default();

        unsafe {
            self.native_mut()
                .seekFrameTime(time, out.as_invalidation_controller_ptr_mut());
        }

        out
    }
}
