#[cfg(feature = "gpu")]
#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "gpu")))]
use crate::gpu::{self, BackendRenderTarget};
use crate::prelude::*;
use crate::{
    Bitmap, Budgeted, Canvas, ColorSpace, ColorType, DeferredDisplayList, IPoint, IRect, ISize,
    Image, ImageInfo, Paint, Pixmap, Size, SurfaceCharacterization, SurfaceProps,
};
use skia_bindings as sb;
use skia_bindings::{SkRefCntBase, SkSurface};
use std::ptr;

pub use skia_bindings::SkSurface_BackendHandleAccess as BackendHandleAccess;
pub use skia_bindings::SkSurface_BackendSurfaceAccess as BackendSurfaceAccess;
pub use skia_bindings::SkSurface_ContentChangeMode as ContentChangeMode;

pub type Surface = RCHandle<SkSurface>;

impl NativeRefCountedBase for SkSurface {
    type Base = SkRefCntBase;
}

impl Surface {
    pub fn new_raster_direct<'pixels>(
        image_info: &ImageInfo,
        pixels: &'pixels mut [u8],
        row_bytes: impl Into<Option<usize>>,
        surface_props: Option<&SurfaceProps>,
    ) -> Option<Borrows<'pixels, Surface>> {
        let row_bytes = row_bytes
            .into()
            .unwrap_or_else(|| image_info.min_row_bytes());

        if pixels.len() < image_info.compute_byte_size(row_bytes) {
            return None;
        };

        Self::from_ptr(unsafe {
            sb::C_SkSurface_MakeRasterDirect(
                image_info.native(),
                pixels.as_mut_ptr() as _,
                row_bytes,
                surface_props.native_ptr_or_null(),
            )
        })
        .map(move |surface| surface.borrows(pixels))
    }

    // TODO: MakeRasterDirect(&Pixmap)
    // TODO: MakeRasterDirectReleaseProc()?

    pub fn new_raster(
        image_info: &ImageInfo,
        row_bytes: impl Into<Option<usize>>,
        surface_props: Option<&SurfaceProps>,
    ) -> Option<Self> {
        Self::from_ptr(unsafe {
            sb::C_SkSurface_MakeRaster(
                image_info.native(),
                row_bytes.into().unwrap_or_default(),
                surface_props.native_ptr_or_null(),
            )
        })
    }

    pub fn new_raster_n32_premul(size: impl Into<ISize>) -> Option<Self> {
        let size = size.into();
        Self::from_ptr(unsafe {
            sb::C_SkSurface_MakeRasterN32Premul(size.width, size.height, ptr::null())
        })
    }
}

#[cfg(feature = "gpu")]
#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "gpu")))]
impl Surface {
    pub fn from_backend_texture(
        context: &mut gpu::Context,
        backend_texture: &gpu::BackendTexture,
        origin: gpu::SurfaceOrigin,
        sample_count: impl Into<Option<usize>>,
        color_type: crate::ColorType,
        color_space: impl Into<Option<crate::ColorSpace>>,
        surface_props: Option<&SurfaceProps>,
    ) -> Option<Self> {
        Self::from_ptr(unsafe {
            sb::C_SkSurface_MakeFromBackendTexture(
                context.native_mut(),
                backend_texture.native(),
                origin,
                sample_count.into().unwrap_or(0).try_into().unwrap(),
                color_type.into_native(),
                color_space.into().into_ptr_or_null(),
                surface_props.native_ptr_or_null(),
            )
        })
    }

    /// Create a new surface from a render target (see the documentation for [BackendRenderTarget]
    /// for more details). Usually, this is the framebuffer. You can set the destination color
    /// space, which affects how images are rendered, how gradients are calculated, how alpha
    /// blending and anti-aliasing work, etc. If in doubt, use `ColorSpace::new_srgb()`. Specifying
    /// [None] defaults to legacy behaviour, which is not color-correct.
    ///
    /// The [ColorType] _must_ match the `Format` specifed in the `FramebufferInfo` that was used to
    /// create the [BackendRenderTarget]. [ColorType] is backend-agnostic, but the `Format` is
    /// specific to each backend, and right now there is no automatic conversion. Therefore, this
    /// needs to be handled manually. If these values do not match, [None] is returned.
    pub fn from_backend_render_target<'a>(
        context: &'a mut gpu::Context,
        backend_render_target: &'a BackendRenderTarget,
        origin: gpu::SurfaceOrigin,
        color_type: ColorType,
        color_space: impl Into<Option<ColorSpace>>,
        surface_props: impl Into<Option<&'a SurfaceProps>>,
    ) -> Option<Self> {
        Self::from_ptr(unsafe {
            sb::C_SkSurface_MakeFromBackendRenderTarget(
                context.native_mut(),
                backend_render_target.native(),
                origin,
                color_type.into_native(),
                color_space.into().into_ptr_or_null(),
                surface_props.into().native_ptr_or_null(),
            )
        })
    }

    #[cfg(feature = "metal")]
    #[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "metal")))]
    pub fn from_ca_metal_layer(
        context: &mut gpu::RecordingContext,
        layer: gpu::mtl::Handle,
        origin: gpu::SurfaceOrigin,
        sample_count: impl Into<Option<usize>>,
        color_type: ColorType,
        color_space: impl Into<Option<ColorSpace>>,
        surface_props: Option<&SurfaceProps>,
    ) -> Option<(Self, gpu::mtl::Handle)> {
        let mut drawable = ptr::null();
        Self::from_ptr(unsafe {
            sb::C_SkSurface_MakeFromCAMetalLayer(
                context.native_mut(),
                layer,
                origin,
                sample_count.into().unwrap_or(0).try_into().unwrap(),
                color_type.into_native(),
                color_space.into().into_ptr_or_null(),
                surface_props.native_ptr_or_null(),
                &mut drawable,
            )
        })
        .map(|surface| (surface, drawable))
    }

    #[cfg(feature = "metal")]
    #[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "metal")))]
    pub fn from_mtk_view(
        context: &mut gpu::RecordingContext,
        mtk_view: gpu::mtl::Handle,
        origin: gpu::SurfaceOrigin,
        sample_count: impl Into<Option<usize>>,
        color_type: ColorType,
        color_space: impl Into<Option<ColorSpace>>,
        surface_props: Option<&SurfaceProps>,
    ) -> Option<Self> {
        Self::from_ptr(unsafe {
            sb::C_SkSurface_MakeFromMTKView(
                context.native_mut(),
                mtk_view,
                origin,
                sample_count.into().unwrap_or(0).try_into().unwrap(),
                color_type.into_native(),
                color_space.into().into_ptr_or_null(),
                surface_props.native_ptr_or_null(),
            )
        })
    }

    pub fn new_render_target(
        context: &mut gpu::RecordingContext,
        budgeted: Budgeted,
        image_info: &ImageInfo,
        sample_count: impl Into<Option<usize>>,
        surface_origin: gpu::SurfaceOrigin,
        surface_props: Option<&SurfaceProps>,
        should_create_with_mips: impl Into<Option<bool>>,
    ) -> Option<Self> {
        Self::from_ptr(unsafe {
            sb::C_SkSurface_MakeRenderTarget(
                context.native_mut(),
                budgeted.into_native(),
                image_info.native(),
                sample_count.into().unwrap_or(0).try_into().unwrap(),
                surface_origin,
                surface_props.native_ptr_or_null(),
                should_create_with_mips.into().unwrap_or_default(),
            )
        })
    }

    pub fn new_render_target_with_characterization(
        context: &mut gpu::RecordingContext,
        characterization: &SurfaceCharacterization,
        budgeted: Budgeted,
    ) -> Option<Self> {
        Self::from_ptr(unsafe {
            sb::C_SkSurface_MakeRenderTarget2(
                context.native_mut(),
                characterization.native(),
                budgeted.into_native(),
            )
        })
    }
}

impl Surface {
    pub fn is_compatible(&self, characterization: &SurfaceCharacterization) -> bool {
        unsafe { self.native().isCompatible(characterization.native()) }
    }

    pub fn new_null(size: impl Into<ISize>) -> Option<Self> {
        let size = size.into();
        Self::from_ptr(unsafe { sb::C_SkSurface_MakeNull(size.width, size.height) })
    }

    pub fn width(&self) -> i32 {
        unsafe { sb::C_SkSurface_width(self.native()) }
    }

    pub fn height(&self) -> i32 {
        unsafe { sb::C_SkSurface_height(self.native()) }
    }

    pub fn image_info(&mut self) -> ImageInfo {
        let mut info = ImageInfo::default();
        unsafe { sb::C_SkSurface_imageInfo(self.native_mut(), info.native_mut()) };
        info
    }

    pub fn generation_id(&mut self) -> u32 {
        unsafe { self.native_mut().generationID() }
    }

    pub fn notify_content_will_change(&mut self, mode: ContentChangeMode) -> &mut Self {
        unsafe { self.native_mut().notifyContentWillChange(mode) }
        self
    }
}

#[cfg(feature = "gpu")]
#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "gpu")))]
impl Surface {
    #[deprecated(
        since = "0.35.0",
        note = "Use recordingContext() and RecordingContext::as_direct_context()"
    )]
    pub fn context(&mut self) -> Option<gpu::Context> {
        self.recording_context()
            .and_then(|mut rc| rc.as_direct_context())
            .map(|dc| dc.into())
    }

    pub fn recording_context(&mut self) -> Option<gpu::RecordingContext> {
        gpu::RecordingContext::from_unshared_ptr(unsafe { self.native_mut().recordingContext() })
    }

    pub fn get_backend_texture(
        &mut self,
        handle_access: BackendHandleAccess,
    ) -> Option<gpu::BackendTexture> {
        unsafe {
            let mut backend_texture = construct(|bt| sb::C_GrBackendTexture_Construct(bt));
            sb::C_SkSurface_getBackendTexture(
                self.native_mut(),
                handle_access,
                &mut backend_texture as _,
            );

            gpu::BackendTexture::from_native_if_valid(backend_texture)
        }
    }

    pub fn get_backend_render_target(
        &mut self,
        handle_access: BackendHandleAccess,
    ) -> Option<BackendRenderTarget> {
        unsafe {
            let mut backend_render_target =
                construct(|rt| sb::C_GrBackendRenderTarget_Construct(rt));
            sb::C_SkSurface_getBackendRenderTarget(
                self.native_mut(),
                handle_access,
                &mut backend_render_target,
            );

            BackendRenderTarget::from_native_c_if_valid(backend_render_target)
        }
    }

    // TODO: support variant with TextureReleaseProc and ReleaseContext
    pub fn replace_backend_texture(
        &mut self,
        backend_texture: &gpu::BackendTexture,
        origin: gpu::SurfaceOrigin,
    ) -> bool {
        self.replace_backend_texture_with_mode(backend_texture, origin, ContentChangeMode::Retain)
    }

    pub fn replace_backend_texture_with_mode(
        &mut self,
        backend_texture: &gpu::BackendTexture,
        origin: gpu::SurfaceOrigin,
        mode: impl Into<Option<ContentChangeMode>>,
    ) -> bool {
        unsafe {
            self.native_mut().replaceBackendTexture(
                backend_texture.native(),
                origin,
                mode.into().unwrap_or(ContentChangeMode::Retain),
                None,
                ptr::null_mut(),
            )
        }
    }
}

impl Surface {
    pub fn canvas(&mut self) -> &mut Canvas {
        let canvas_ref = unsafe { &mut *self.native_mut().getCanvas() };
        Canvas::borrow_from_native(canvas_ref)
    }

    // TODO: why is self mutable here?
    pub fn new_surface(&mut self, info: &ImageInfo) -> Option<Surface> {
        Surface::from_ptr(unsafe { sb::C_SkSurface_makeSurface(self.native_mut(), info.native()) })
    }

    pub fn new_surface_with_dimensions(&mut self, dim: impl Into<ISize>) -> Option<Surface> {
        let dim = dim.into();
        Surface::from_ptr(unsafe {
            sb::C_SkSurface_makeSurface2(self.native_mut(), dim.width, dim.height)
        })
    }

    pub fn image_snapshot(&mut self) -> Image {
        Image::from_ptr(unsafe {
            sb::C_SkSurface_makeImageSnapshot(self.native_mut(), ptr::null())
        })
        .unwrap()
    }

    // TODO: combine this function with image_snapshot and make bounds optional()?
    pub fn image_snapshot_with_bounds(&mut self, bounds: impl AsRef<IRect>) -> Option<Image> {
        Image::from_ptr(unsafe {
            sb::C_SkSurface_makeImageSnapshot(self.native_mut(), bounds.as_ref().native())
        })
    }

    pub fn draw(&mut self, canvas: &mut Canvas, size: impl Into<Size>, paint: Option<&Paint>) {
        let size = size.into();
        unsafe {
            self.native_mut().draw(
                canvas.native_mut(),
                size.width,
                size.height,
                paint.native_ptr_or_null(),
            )
        }
    }

    pub fn peek_pixels(&mut self) -> Option<Borrows<Pixmap>> {
        let mut pm = Pixmap::default();
        unsafe { self.native_mut().peekPixels(pm.native_mut()) }
            .if_true_then_some(move || pm.borrows(self))
    }

    // TODO: why is self mut?
    pub fn read_pixels_to_pixmap(&mut self, dst: &Pixmap, src: impl Into<IPoint>) -> bool {
        let src = src.into();
        unsafe { self.native_mut().readPixels(dst.native(), src.x, src.y) }
    }

    pub fn read_pixels(
        &mut self,
        dst_info: &ImageInfo,
        dst_pixels: &mut [u8],
        dst_row_bytes: usize,
        src: impl Into<IPoint>,
    ) -> bool {
        if dst_row_bytes < dst_info.min_row_bytes() {
            return false;
        };
        let height: usize = dst_info.height().try_into().unwrap();
        if dst_pixels.len() < dst_row_bytes * height {
            return false;
        };
        let src = src.into();
        unsafe {
            self.native_mut().readPixels1(
                dst_info.native(),
                dst_pixels.as_mut_ptr() as _,
                dst_row_bytes,
                src.x,
                src.y,
            )
        }
    }

    // TODO: why is self mut?
    // TODO: why is Bitmap non-mutable.
    pub fn read_pixels_to_bitmap(&mut self, bitmap: &Bitmap, src: impl Into<IPoint>) -> bool {
        let src = src.into();
        unsafe { self.native_mut().readPixels2(bitmap.native(), src.x, src.y) }
    }

    // TODO: AsyncReadResult, RescaleGamma (m79, m86)
    // TODO: wrap asyncRescaleAndReadPixels (m76, m79)
    // TODO: wrap asyncRescaleAndReadPixelsYUV420 (m77, m79)

    pub fn write_pixels_from_pixmap(&mut self, src: &Pixmap, dst: impl Into<IPoint>) {
        let dst = dst.into();
        unsafe { self.native_mut().writePixels(src.native(), dst.x, dst.y) }
    }

    pub fn write_pixels_from_bitmap(&mut self, bitmap: &Bitmap, dst: impl Into<IPoint>) {
        let dst = dst.into();
        unsafe {
            self.native_mut()
                .writePixels1(bitmap.native(), dst.x, dst.y)
        }
    }

    pub fn props(&self) -> &SurfaceProps {
        SurfaceProps::from_native_ref(unsafe { &*sb::C_SkSurface_props(self.native()) })
    }

    pub fn flush_and_submit(&mut self, sync_cpu: bool) {
        unsafe {
            self.native_mut().flushAndSubmit(sync_cpu);
        }
    }

    // TODO: wait()

    pub fn characterize(&self) -> Option<SurfaceCharacterization> {
        let mut sc = SurfaceCharacterization::default();
        unsafe { self.native().characterize(sc.native_mut()) }.if_true_some(sc)
    }

    pub fn draw_display_list(
        &mut self,
        deferred_display_list: impl Into<DeferredDisplayList>,
    ) -> bool {
        unsafe {
            sb::C_SkSurface_draw(
                self.native_mut(),
                deferred_display_list.into().into_ptr() as *const _,
            )
        }
    }
}

#[cfg(feature = "gpu")]
#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "gpu")))]
#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "gpu")))]
impl Surface {
    // After deprecated since 0.30.0 (m85), the default flush() behavior changed in m86.
    // For more information, take a look at the documentation in Skia's SkSurface.h
    pub fn flush(&mut self) {
        let info = gpu::FlushInfo::default();
        self.flush_with_mutable_state(&info, None);
    }

    pub fn flush_with_access_info(
        &mut self,
        access: BackendSurfaceAccess,
        info: &gpu::FlushInfo,
    ) -> gpu::SemaphoresSubmitted {
        unsafe { self.native_mut().flush(access, info.native()) }
    }

    pub fn flush_with_mutable_state<'a>(
        &mut self,
        info: &gpu::FlushInfo,
        new_state: impl Into<Option<&'a gpu::BackendSurfaceMutableState>>,
    ) -> gpu::SemaphoresSubmitted {
        unsafe {
            self.native_mut()
                .flush1(info.native(), new_state.into().native_ptr_or_null())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BackendHandleAccess, BackendSurfaceAccess, Canvas, ContentChangeMode, ISize, ImageInfo,
        NativeAccess, NativeRefCounted, NativeRefCountedBase, Paint, Surface,
    };

    #[test]
    fn test_surface_content_change_mode_naming() {
        let _ = ContentChangeMode::Retain;
    }

    #[test]
    fn test_surface_backend_handle_access_naming() {
        let _ = BackendHandleAccess::FlushWrite;
    }

    #[test]
    fn test_surface_backend_surface_access_naming() {
        let _ = BackendSurfaceAccess::Present;
    }

    #[test]
    fn create() {
        assert!(Surface::new_raster_n32_premul((0, 0)).is_none());
        let surface = Surface::new_raster_n32_premul((1, 1)).unwrap();
        assert_eq!(1, surface.native().ref_counted_base()._ref_cnt())
    }

    #[test]
    fn test_raster_direct() {
        let image_info = ImageInfo::new(
            (20, 20),
            crate::ColorType::RGBA8888,
            crate::AlphaType::Unpremul,
            None,
        );
        let min_row_bytes = image_info.min_row_bytes();
        let mut pixels = vec![0u8; image_info.compute_byte_size(min_row_bytes)];
        let mut surface = Surface::new_raster_direct(
            &image_info,
            pixels.as_mut_slice(),
            Some(min_row_bytes),
            None,
        )
        .unwrap();
        let paint = Paint::default();
        surface.canvas().draw_circle((10, 10), 10.0, &paint);
    }

    #[test]
    fn test_drawing_owned_as_exclusive_ref_ergonomics() {
        let mut surface = Surface::new_raster_n32_premul((16, 16)).unwrap();

        // option1:
        // - An &mut canvas can be drawn to.
        {
            let mut canvas = Canvas::new(ISize::new(16, 16), None).unwrap();
            surface.draw(&mut canvas, (5.0, 5.0), None);
            surface.draw(&mut canvas, (10.0, 10.0), None);
        }

        // option2:
        // - A canvas from another surface can be drawn to.
        {
            let mut surface2 = Surface::new_raster_n32_premul((16, 16)).unwrap();
            let canvas = surface2.canvas();
            surface.draw(canvas, (5.0, 5.0), None);
            surface.draw(canvas, (10.0, 10.0), None);
        }
    }
}
