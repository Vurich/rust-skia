use crate::prelude::*;
use crate::{image_filter::CropRect, effects::image_filters, IRect, ImageFilter};
use skia_bindings as sb;
use skia_bindings::SkImageFilter;
use std::convert::TryInto;

impl ImageFilter {
    pub fn merge<'a>(
        filters: impl IntoIterator<Item = Option<Self>>,
        crop_rect: impl Into<Option<&'a IRect>>,
    ) -> Option<Self> {
        image_filters::merge(filters, crop_rect)
    }
}

#[deprecated(since = "0.19.0", note = "use image_filters::merge")]
#[allow(clippy::new_ret_no_self)]
pub fn new<'a>(
    filters: impl IntoIterator<Item = ImageFilter>,
    crop_rect: impl Into<Option<&'a CropRect>>,
) -> Option<ImageFilter> {
    let filter_ptrs: Vec<*mut SkImageFilter> = filters.into_iter().map(|f| f.into_ptr()).collect();
    ImageFilter::from_ptr(unsafe {
        sb::C_SkMergeImageFilter_Make(
            filter_ptrs.as_ptr(),
            filter_ptrs.len().try_into().unwrap(),
            crop_rect.into().native_ptr_or_null(),
        )
    })
}
