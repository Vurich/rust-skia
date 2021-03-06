use crate::prelude::*;
use skia_bindings as sb;
use skia_bindings::SkColorMatrix;

pub type ColorMatrix = Handle<SkColorMatrix>;
unsafe impl Send for ColorMatrix {}
unsafe impl Sync for ColorMatrix {}

impl NativeDrop for SkColorMatrix {
    fn drop(&mut self) {}
}

impl PartialEq for Handle<SkColorMatrix> {
    fn eq(&self, other: &Self) -> bool {
        let mut array_self = [0.0f32; 20];
        let mut array_other = [0.0f32; 20];
        self.get_row_major(&mut array_self);
        other.get_row_major(&mut array_other);
        array_self.eq(&array_other)
    }
}

impl Default for Handle<SkColorMatrix> {
    fn default() -> Self {
        ColorMatrix::construct(|cm| unsafe { sb::C_SkColorMatrix_Construct(cm) })
    }
}

impl ColorMatrix {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        m00: f32,
        m01: f32,
        m02: f32,
        m03: f32,
        m04: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m13: f32,
        m14: f32,
        m20: f32,
        m21: f32,
        m22: f32,
        m23: f32,
        m24: f32,
        m30: f32,
        m31: f32,
        m32: f32,
        m33: f32,
        m34: f32,
    ) -> Self {
        ColorMatrix::construct(|cm| unsafe {
            sb::C_SkColorMatrix_Construct2(
                cm, m00, m01, m02, m03, m04, m10, m11, m12, m13, m14, m20, m21, m22, m23, m24, m30,
                m31, m32, m33, m34,
            )
        })
    }

    pub fn set_identity(&mut self) {
        unsafe { self.native_mut().setIdentity() }
    }

    pub fn set_scale(
        &mut self,
        r_scale: f32,
        g_scale: f32,
        b_scale: f32,
        a_scale: impl Into<Option<f32>>,
    ) {
        unsafe {
            self.native_mut()
                .setScale(r_scale, g_scale, b_scale, a_scale.into().unwrap_or(1.0))
        }
    }

    pub fn post_translate(&mut self, dr: f32, dg: f32, db: f32, da: f32) {
        unsafe { self.native_mut().postTranslate(dr, dg, db, da) }
    }

    pub fn set_concat(&mut self, a: &ColorMatrix, b: &ColorMatrix) {
        unsafe { self.native_mut().setConcat(a.native(), b.native()) }
    }

    pub fn pre_concat(&mut self, mat: &ColorMatrix) {
        let self_ptr = self.native() as *const _;
        unsafe { self.native_mut().setConcat(self_ptr, mat.native()) }
    }

    pub fn post_concat(&mut self, mat: &ColorMatrix) {
        let self_ptr = self.native() as *const _;
        unsafe { self.native_mut().setConcat(mat.native(), self_ptr) }
    }

    pub fn set_saturation(&mut self, sat: f32) {
        unsafe { self.native_mut().setSaturation(sat) }
    }

    pub fn set_row_major(&mut self, src: &[f32; 20]) {
        unsafe {
            sb::C_SkColorMatrix_setRowMajor(self.native_mut(), src.as_ptr());
        }
    }

    pub fn get_row_major(&self, dst: &mut [f32; 20]) {
        unsafe {
            sb::C_SkColorMatrix_getRowMajor(self.native(), dst.as_mut_ptr());
        }
    }
}
