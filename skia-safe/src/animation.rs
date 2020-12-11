use std::{ffi::CString, path::Path};

use crate::{Canvas, RCHandle, Rect};
use skia_bindings as sb;

pub type Animation = RCHandle<sb::skottie_Animation>;

bitflags::bitflags! {
    #[derive(Default)]
    pub struct RenderFlags: u32 {
        const SKIP_TOP_LEVEL_ISOLATION = sb::skottie_Animation_RenderFlag::kSkipTopLevelIsolation as _;
        const DISABLE_TOP_LEVEL_CLIPPING = sb::skottie_Animation_RenderFlag::kDisableTopLevelClipping as _;
    }
}

impl Animation {
    pub fn from_data(data: &[u8]) -> Option<Self> {
        Self::from_ptr(unsafe { sb::skottie_Animation::Make(data.as_ptr(), data.len()) })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Option<Self> {
        let path = CString::from(path.as_ref());

        Self::from_ptr(unsafe { sb::skottie_Animation::MakeFromFile(path.as_ptr()) })
    }

    pub fn render_with_flags(&self, canvas: &mut Canvas, dst: impl Into<Option<Rect>>) {
        let dst = dst.into();

        unsafe { sb::skottie_Animation::render(&*self.0, canvas as *mut _, dst.into_ptr_or_null()) }
    }

    pub fn render_with_flags(
        &self,
        canvas: &mut Canvas,
        dst: impl Into<Option<Rect>>,
        flags: RenderFlags,
    ) {
        let dst = dst.into();

        unsafe {
            sb::skottie_Animation::render1(
                &*self.0,
                canvas as *mut _,
                dst.into_ptr_or_null(),
                flags.into(),
            )
        }
    }
}
