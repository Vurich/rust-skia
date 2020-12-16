#[cfg(feature = "textlayout")]
#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "textlayout")))]
pub(crate) mod paragraph;
#[cfg(feature = "textlayout")]
#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "textlayout")))]
pub mod shaper;
#[cfg(feature = "textlayout")]
#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "textlayout")))]
pub use shaper::{icu, Shaper};

// Export everything below paragraph under textlayout
#[cfg(feature = "textlayout")]
#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "textlayout")))]
pub mod textlayout {
    use crate::paragraph;
    pub use paragraph::*;
}
