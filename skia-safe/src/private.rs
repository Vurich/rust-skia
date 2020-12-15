#[cfg(feature = "gpu")]
#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "gpu")))]
pub mod gpu;
pub(crate) mod safe32;
