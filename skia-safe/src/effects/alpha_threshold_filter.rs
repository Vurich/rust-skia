use crate::prelude::*;
use crate::{effects::image_filters, image_filter::CropRect, scalar, IRect, ImageFilter, Region};
use skia_bindings as sb;

impl ImageFilter {
    pub fn alpha_threshold<'a>(
        self,
        crop_rect: impl Into<Option<&'a IRect>>,
        region: &Region,
        inner_min: scalar,
        outer_max: scalar,
    ) -> Option<Self> {
        image_filters::alpha_threshold(region, inner_min, outer_max, self, crop_rect)
    }
}

#[deprecated(since = "0.19.0", note = "use image_filters::alpha_threshold()")]
pub fn new<'a>(
    region: &Region,
    inner_min: scalar,
    outer_max: scalar,
    input: impl Into<ImageFilter>,
    crop_rect: impl Into<Option<&'a CropRect>>,
) -> Option<ImageFilter> {
    ImageFilter::from_ptr(unsafe {
        sb::C_SkAlphaThresholdFilter_Make(
            region.native(),
            inner_min,
            outer_max,
            input.into().into_ptr(),
            crop_rect.into().native_ptr_or_null(),
        )
    })
}
