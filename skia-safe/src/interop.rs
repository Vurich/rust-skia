/// Simple Skia types that are not exported and used to
/// to marshal between Rust and Skia types only.
mod stream;
pub(crate) use self::stream::*;

mod string;
pub(crate) use self::string::*;

#[cfg(feature = "textlayout")]
#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "textlayout")))]
mod strings;
#[cfg(feature = "textlayout")]
#[cfg_attr(any(docsrs, feature = "nightly"), doc(cfg(feature = "textlayout")))]
pub(crate) use self::strings::*;
