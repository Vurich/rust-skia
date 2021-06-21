use crate::artifact;
use crate::drivers::DrawingDriver;
use skia_safe::{gpu, Budgeted, Canvas, ImageInfo, Surface};
use std::path::Path;

pub struct OpenGL {
    context: gpu::Context,
}

impl DrawingDriver for OpenGL {
    const NAME: &'static str = "opengl";

    fn new() -> Self {
        Self {
            context: gpu::Context::new_gl(None, None).unwrap(),
        }
    }

    fn draw_image(
        &mut self,
        (width, height): (i32, i32),
        path: &Path,
        name: &str,
        func: impl Fn(&mut Canvas),
    ) {
        let image_info = ImageInfo::new_n32_premul((width * 2, height * 2), None);
        let mut surface = Surface::new_render_target(
            // `self.context` is a refcounted pointer, and so cloning here is valid and will still
            // mutate the internal context correctly.
            &mut self.context.clone().into(),
            Budgeted::Yes,
            &image_info,
            None,
            gpu::SurfaceOrigin::BottomLeft,
            None,
            false,
        )
        .unwrap();

        artifact::draw_image_on_surface(&mut surface, path, name, func);
    }
}
