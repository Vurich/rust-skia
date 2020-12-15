#![deny(missing_docs)]

use super::{FontFamilies, TextBaseline, TextShadow};
use crate::interop::{AsStr, FromStrs, SetStr};
use crate::prelude::*;
use crate::textlayout::{RangeExtensions, EMPTY_INDEX, EMPTY_RANGE};
use crate::{interop, scalar, Color, FontMetrics, FontStyle, Paint, Typeface};
use skia_bindings as sb;
use std::ops::Range;
use std::slice;

pub use sb::{
    skia_textlayout_PlaceholderAlignment as PlaceholderAlignment,
    skia_textlayout_StyleType as StyleType,
    skia_textlayout_TextDecorationMode as TextDecorationMode,
    skia_textlayout_TextDecorationStyle as TextDecorationStyle,
};

bitflags! {
    /// Flags for possible extra additions to the text.
    pub struct TextDecoration: u32 {
        /// Default - no additional extra decorations.
        const NO_DECORATION = sb::skia_textlayout_TextDecoration::kNoDecoration as _;
        /// A horizontal line underneath each line of the text.
        const UNDERLINE = sb::skia_textlayout_TextDecoration::kUnderline as _;
        /// A horizontal line above each line of the text.
        const OVERLINE = sb::skia_textlayout_TextDecoration::kOverline as _;
        /// A horizontal line through the centerline of each line of the text (often
        /// referred to as strikethrough).
        const LINE_THROUGH = sb::skia_textlayout_TextDecoration::kLineThrough as _;
    }
}

/// All possible decorations (i.e. an underline, overline and strikethrough).
pub const ALL_TEXT_DECORATIONS: TextDecoration = TextDecoration::ALL;

impl Default for TextDecoration {
    fn default() -> Self {
        TextDecoration::NO_DECORATION
    }
}

impl TextDecoration {
    /// All possible decorations (i.e. an underline, overline and strikethrough).
    pub const ALL: TextDecoration = TextDecoration::all();
}

/// Decoration configuration for a piece of text.
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Decoration {
    /// The kind of decoration (underline, overline, or strikethrough).
    pub ty: TextDecoration,
    /// The "mode" for the decoration - if it is visible even for whitespace characters.
    pub mode: TextDecorationMode,
    /// The color of the decoration, see documentation for [Color]. This can be independent
    /// of the color of the text itself.
    pub color: Color,
    /// The style of the text decoration. See documentation for [TextDecorationStyle].
    pub style: TextDecorationStyle,
    /// The thickness, expressed as a multiple of the weight of the text.
    pub thickness_multiplier: scalar,
}

impl NativeTransmutable<sb::skia_textlayout_Decoration> for Decoration {}

/// An individual feature of a supplied font - i.e. settings to enable and disable variantions in the
/// font. For further information on what font features are and how to set them, you can consult the
/// [MDN documentation on the subject](https://developer.mozilla.org/en-US/docs/Web/CSS/font-feature-settings).
pub type FontFeature = Handle<sb::skia_textlayout_FontFeature>;

unsafe impl Send for FontFeature {}
unsafe impl Sync for FontFeature {}

impl NativeDrop for sb::skia_textlayout_FontFeature {
    fn drop(&mut self) {
        unsafe { sb::C_FontFeature_destruct(self) }
    }
}

impl NativeClone for sb::skia_textlayout_FontFeature {
    fn clone(&self) -> Self {
        construct(|ts| unsafe { sb::C_FontFeature_CopyConstruct(ts, self) })
    }
}

impl PartialEq for Handle<sb::skia_textlayout_FontFeature> {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name() && self.value() == other.value()
    }
}

impl FontFeature {
    /// The name of the feature.
    pub fn name(&self) -> &str {
        self.native().fName.as_str()
    }

    /// The value of the feature (this isn't relevant for all features, some are simply enabled
    /// or disabled by presence).
    pub fn value(&self) -> i32 {
        self.native().fValue
    }
}

/// The style for a [Placeholder].
#[derive(Clone, Debug, Default)]
pub struct PlaceholderStyle {
    /// The width of the placeholder.
    pub width: scalar,
    /// The height of the placeholder.
    pub height: scalar,
    /// Where to vertically align the placeholder relative to the surrounding text. See [PlaceholderAlignment]
    /// for more information.
    pub alignment: PlaceholderAlignment,
    /// Whether the placeholder is placed relative to the alphabetic baseline (i.e. where the base
    /// of glyphs for characters such as a and o sit) or the ideographic baseline (i.e. the lowest
    /// point in the text, below the lowest point of glyphs for characters such as j or p).
    pub baseline: TextBaseline,
    /// The offset from the text's baseline.
    pub baseline_offset: scalar,
}

impl NativeTransmutable<sb::skia_textlayout_PlaceholderStyle> for PlaceholderStyle {}

impl PartialEq for PlaceholderStyle {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.native().equals(other.native()) }
    }
}

impl PlaceholderStyle {
    /// Create a new style for a placeholder, see documentation for the fields of
    /// [PlaceholderStyle] for more information.
    pub fn new(
        width: scalar,
        height: scalar,
        alignment: PlaceholderAlignment,
        baseline: TextBaseline,
        offset: scalar,
    ) -> Self {
        Self {
            width,
            height,
            alignment,
            baseline,
            baseline_offset: offset,
        }
    }
}

/// Style settings for a piece of text. See individual methods to see what settings are available to
/// be configured.
pub type TextStyle = Handle<sb::skia_textlayout_TextStyle>;

unsafe impl Send for TextStyle {}
unsafe impl Sync for TextStyle {}

impl NativeDrop for sb::skia_textlayout_TextStyle {
    fn drop(&mut self) {
        unsafe { sb::C_TextStyle_destruct(self) }
    }
}

impl NativeClone for sb::skia_textlayout_TextStyle {
    fn clone(&self) -> Self {
        construct(|ts| unsafe { sb::C_TextStyle_CopyConstruct(ts, self) })
    }
}

impl NativePartialEq for sb::skia_textlayout_TextStyle {
    fn eq(&self, rhs: &Self) -> bool {
        unsafe { self.equals(rhs) }
    }
}

impl Default for Handle<sb::skia_textlayout_TextStyle> {
    fn default() -> Self {
        Self::new()
    }
}

impl TextStyle {
    /// Create a new, default text style.
    pub fn new() -> Self {
        TextStyle::construct(|ts| unsafe { sb::C_TextStyle_Construct(ts) })
    }

    /// Create a new style with all the same properties as this one but which is a "placeholder"
    /// style - i.e. the returned style will calculate metrics and reflow as if all the
    /// properties are set, but will not actually draw anything.
    pub fn to_placeholder(&self) -> Self {
        TextStyle::from_native_c(unsafe { sb::skia_textlayout_TextStyle::new(self.native(), true) })
    }

    /// Compare this style to another.
    pub fn equals(&self, other: &TextStyle) -> bool {
        *self == *other
    }

    /// Compare this style to another, only comparing the font set.
    pub fn equals_by_fonts(&self, that: &TextStyle) -> bool {
        unsafe { self.native().equalsByFonts(that.native()) }
    }

    /// Compare a single attribute for equality between this style and another.
    pub fn match_one_attribute(&self, style_type: StyleType, other: &TextStyle) -> bool {
        unsafe { self.native().matchOneAttribute(style_type, other.native()) }
    }

    /// Get the color of the text body.
    pub fn color(&self) -> Color {
        Color::from_native_c(self.native().fColor)
    }

    /// Set the color of the text body.
    pub fn set_color(&mut self, color: impl Into<Color>) -> &mut Self {
        self.native_mut().fColor = color.into().into_native();
        self
    }

    /// Get the foreground paint configuration, which affects how the text body is drawn.
    /// By default this will use flat color, defined by `color`. See documentation for [Paint].
    pub fn foreground(&self) -> Option<&Paint> {
        self.native()
            .fHasForeground
            .if_true_some(Paint::from_native_ref(&self.native().fForeground))
    }

    /// Set the foreground paint configuration, which affects how the text body is drawn.
    /// By default this will use flat color, defined by `color`. See documentation for [Paint].
    pub fn set_foreground_color(&mut self, paint: impl Into<Option<Paint>>) -> &mut Self {
        let n = self.native_mut();
        n.fHasForeground = paint
            .into()
            .map(|paint| n.fForeground.replace_with(paint))
            .is_some();
        self
    }

    /// Get the background paint configuration, which affects how the bounding box of the text is
    /// drawn. By default this is transparent. See documentation for [Paint].
    pub fn background(&self) -> Option<&Paint> {
        self.native()
            .fHasBackground
            .if_true_some(Paint::from_native_ref(&self.native().fBackground))
    }

    /// Set the background paint configuration, which affects how the bounding box of the text is
    /// drawn. By default this is transparent. See documentation for [Paint].
    pub fn set_background_color(&mut self, paint: impl Into<Option<Paint>>) -> &mut Self {
        let n = self.native_mut();
        n.fHasBackground = paint
            .into()
            .map(|paint| n.fBackground.replace_with(paint))
            .is_some();
        self
    }

    /// Get the settings related to the text's decoration (underline, overline, or strikethrough).
    /// See documentation for [Decoration].
    pub fn decoration(&self) -> &Decoration {
        Decoration::from_native_ref(&self.native().fDecoration)
    }

    /// Get a mutable reference to the settings related to the text's decoration (underline,
    /// overline, or strikethrough). See documentation for [Decoration].
    pub fn decoration_mut(&mut self) -> &mut Decoration {
        Decoration::from_native_ref_mut(&mut self.native_mut().fDecoration)
    }

    /// Get the settings related to how the font is configured, such as weight.
    /// See documentation for [FontStyle].
    pub fn font_style(&self) -> FontStyle {
        FontStyle::from_native_c(self.native().fFontStyle)
    }

    /// Set the settings related to how the font is configured, such as weight.
    /// See documentation for [FontStyle].
    pub fn set_font_style(&mut self, font_style: FontStyle) -> &mut Self {
        self.native_mut().fFontStyle = font_style.into_native();
        self
    }

    /// Get any drop shadows that should be drawn under the text. See documentation for [TextShadow].
    pub fn shadows(&self) -> &[TextShadow] {
        unsafe {
            let ts: &sb::TextShadows = transmute_ref(&self.native().fTextShadows);
            let mut cnt = 0;
            let ptr = TextShadow::from_native_ref(&*sb::C_TextShadows_ptr_count(ts, &mut cnt));
            slice::from_raw_parts(ptr, cnt)
        }
    }

    /// Append a new drop shadow to the style. See documenation for [TextShadow] for more information on
    /// configuring this.
    pub fn add_shadow(&mut self, shadow: TextShadow) -> &mut Self {
        unsafe { sb::C_TextStyle_addShadow(self.native_mut(), shadow.native()) }
        self
    }

    /// Remove any drop shadows defined on this style.
    pub fn reset_shadows(&mut self) -> &mut Self {
        unsafe { sb::C_TextStyle_resetShadows(self.native_mut()) }
        self
    }

    /// Give a list of all font feature settings that have been set for this style. See
    /// documentation for [FontFeature].
    pub fn font_features(&self) -> &[FontFeature] {
        unsafe {
            let ff: &sb::FontFeatures = transmute_ref(&self.native().fFontFeatures);
            let mut cnt = 0;
            let ptr = FontFeature::from_native_ref(&*sb::C_FontFeatures_ptr_count(ff, &mut cnt));
            slice::from_raw_parts(ptr, cnt)
        }
    }

    /// Append a new font feature setting to this style. See documentation for [FontFeature].
    pub fn add_font_feature(&mut self, font_feature: impl AsRef<str>, value: i32) {
        let font_feature = interop::String::from_str(font_feature);
        unsafe { sb::C_TextStyle_addFontFeature(self.native_mut(), font_feature.native(), value) }
    }

    /// Remove any font feature settings that have been manually set on this style.
    pub fn reset_font_features(&mut self) {
        unsafe { sb::C_TextStyle_resetFontFeatures(self.native_mut()) }
    }

    /// Get the font size (in px) defined by this style.
    pub fn font_size(&self) -> scalar {
        self.native().fFontSize
    }

    /// Set the font size (in px) for this style.
    pub fn set_font_size(&mut self, size: scalar) -> &mut Self {
        self.native_mut().fFontSize = size;
        self
    }

    /// Get an array of font families, in order of preference, that this style will use.
    pub fn font_families(&self) -> FontFamilies {
        unsafe {
            let mut count = 0;
            let ptr = sb::C_TextStyle_getFontFamilies(self.native(), &mut count);
            FontFamilies(slice::from_raw_parts(ptr, count))
        }
    }

    /// Set the list of font families by name, in order of preference. See [crate::FontMgr] for
    /// more information on how fonts are loaded.
    pub fn set_font_families(&mut self, families: &[impl AsRef<str>]) -> &mut Self {
        let families: Vec<interop::String> = FromStrs::from_strs(families);
        let families = families.native();
        unsafe {
            sb::C_TextStyle_setFontFamilies(self.native_mut(), families.as_ptr(), families.len())
        }
        self
    }

    /// Set the height of the text. This will not take effect unless you have set
    /// `height_override` to `true`.
    pub fn set_height(&mut self, height: scalar) -> &mut Self {
        self.native_mut().fHeight = height;
        self
    }

    /// Get the maximum height of the text. If `height_override` is false (the default) this
    /// will return 0, as the height has no effect.
    pub fn height(&self) -> scalar {
        let n = self.native();
        if n.fHeightOverride {
            n.fHeight
        } else {
            0.0
        }
    }

    /// Set whether the height should be explicitly overridden.
    pub fn set_height_override(&mut self, height_override: bool) -> &mut Self {
        self.native_mut().fHeightOverride = height_override;
        self
    }

    /// Returns true if this style should explicitly override the height of the
    /// text using the value set by `set_height`.
    pub fn height_override(&self) -> bool {
        self.native().fHeightOverride
    }

    /// Set the letter spacing, in px. 0 is the "natural" spacing defined by the font, negative
    /// numbers cause letters to be closer together than usual, and positive numbers cause the
    /// letters to be further apart than usual.
    pub fn set_letter_spacing(&mut self, letter_spacing: scalar) -> &mut Self {
        self.native_mut().fLetterSpacing = letter_spacing;
        self
    }

    /// Get the letter spacing, in px.
    pub fn letter_spacing(&self) -> scalar {
        self.native().fLetterSpacing
    }

    /// Set the word spacing, in px. 0 is the "natural" spacing defined by the font, negative
    /// numbers cause words to be closer together than usual, and positive numbers cause the
    /// words to be further apart than usual.
    pub fn set_word_spacing(&mut self, word_spacing: scalar) -> &mut Self {
        self.native_mut().fWordSpacing = word_spacing;
        self
    }

    /// Get the word spacing, in px.
    pub fn word_spacing(&self) -> scalar {
        self.native().fWordSpacing
    }

    /// Get the specific typeface used by this style, if one is set. This specifies both a font
    /// family, but also its variant such as bold, italic, etc.
    pub fn typeface(&self) -> Option<Typeface> {
        Typeface::from_unshared_ptr(self.native().fTypeface.fPtr)
    }

    /// Set the specific typeface to use. This specifies both a font family, but also its variant such
    /// as bold, italic, etc.
    pub fn set_typeface(&mut self, typeface: impl Into<Option<Typeface>>) -> &mut Self {
        unsafe {
            sb::C_TextStyle_setTypeface(self.native_mut(), typeface.into().into_ptr_or_null())
        }
        self
    }

    /// Get the locale of this text style. This can affect capitalization rules and other transformations.
    pub fn locale(&self) -> &str {
        self.native().fLocale.as_str()
    }

    /// Set the locale of this text style. This can affect capitalization rules and other transformations.
    pub fn set_locale(&mut self, locale: impl AsRef<str>) -> &mut Self {
        self.native_mut().fLocale.set_str(locale);
        self
    }

    /// Get the baseline of the text. The default is [TextBaseline::Alphabetic], which specifies that
    /// the glyphs for characters such as y, j, q (depending on font), as well as many characters in
    /// languages such as Chinese, may go below the baseline of the text. [TextBaseline::Ideographic]
    /// specifies that each line of text will be raised such that these glyphs will not be drawn below
    /// the baseline of the text. This does not affect the vertical positions of the glyphs in relation
    /// to one another, just their relation to the baseline.
    pub fn text_baseline(&self) -> TextBaseline {
        self.native().fTextBaseline
    }

    /// Set the baseline of the text. The default is [TextBaseline::Alphabetic], which specifies that
    /// the glyphs for characters such as y, j, q (depending on font), as well as many characters in
    /// languages such as Chinese, may go below the baseline of the text. [TextBaseline::Ideographic]
    /// specifies that each line of text will be raised such that these glyphs will not be drawn below
    /// the baseline of the text. This does not affect the vertical positions of the glyphs in relation
    /// to one another, just their relation to the baseline.
    pub fn set_text_baseline(&mut self, baseline: TextBaseline) -> &mut Self {
        self.native_mut().fTextBaseline = baseline;
        self
    }

    /// A list of metrics for all the fonts in this style.
    pub fn font_metrics(&self) -> FontMetrics {
        FontMetrics::construct(|fm| unsafe { self.native().getFontMetrics(fm) })
    }

    /// Whether this is a "placeholder" style - i.e. it will calculate metrics and reflow as if all the
    /// properties are set, but will not actually draw anything.
    pub fn is_placeholder(&self) -> bool {
        self.native().fIsPlaceholder
    }

    /// Make this a "placeholder" style - i.e. it will calculate metrics and reflow as if all the
    /// properties are set, but will not actually draw anything.
    pub fn set_placeholder(&mut self) -> &mut Self {
        self.native_mut().fIsPlaceholder = true;
        self
    }
}

/// Index into a piece of text, specified in UTF-16 codepoints.
pub type TextIndex = usize;
/// A range of characters in a piece of text, specified in UTF-16 codepoints.
pub type TextRange = Range<usize>;
/// An empty string, specified as a range.
pub const EMPTY_TEXT: TextRange = EMPTY_RANGE;

/// A run of text which shares the same style.
#[derive(Clone, PartialEq)]
pub struct Block {
    /// The range of glyphs.
    pub range: TextRange,
    /// The style associated with it.
    pub style: TextStyle,
}

impl NativeTransmutable<sb::skia_textlayout_Block> for Block {}

impl Default for Block {
    fn default() -> Self {
        Self {
            range: EMPTY_RANGE,
            style: Default::default(),
        }
    }
}

impl Block {
    /// Create a new "block", specifying the style, as well as the range of characters (specified in
    /// UTF-16 codepoints).
    pub fn new(text_range: TextRange, style: TextStyle) -> Self {
        Self {
            range: text_range,
            style,
        }
    }

    /// Extend this block to cover a wider range. This additional range _must_ start at the end of the current range.
    pub fn add(&mut self, tail: TextRange) -> &mut Self {
        debug_assert!(self.range.end == tail.start);
        self.range = self.range.start..self.range.start + self.range.width() + tail.width();
        self
    }
}

/// An index to a single [Block].
pub type BlockIndex = usize;
/// A range of [Block]s.
pub type BlockRange = Range<usize>;

/// The "empty" block, i.e. an always-invalid index.
pub const EMPTY_BLOCK: usize = EMPTY_INDEX;
/// An empty range of blocks.
pub const EMPTY_BLOCKS: Range<usize> = EMPTY_RANGE;

/// A placeholder, which takes the place of a glyph without actually being rendered or being
/// associated with any text. This can be used, for example, to insert spans of arbitrary
/// content into a larger paragraph.
#[derive(Clone, PartialEq)]
pub struct Placeholder {
    /// The range of characters covered by this placeholder. Specified in UTF-16 codepoints.
    pub range: TextRange,
    /// The style of the placeholder, see [PlaceholderStyle] for more information.
    pub style: PlaceholderStyle,
    /// The style associated with the text. See documentation for [TextStyle] (and specifically the
    /// methods related to placeholders) for more information.
    pub text_style: TextStyle,
    /// Which blocks come before this in the text blob.
    pub blocks_before: BlockRange,
    /// Which text comes before this in the text blob.
    pub text_before: TextRange,
}

impl NativeTransmutable<sb::skia_textlayout_Placeholder> for Placeholder {}

impl Default for Placeholder {
    fn default() -> Self {
        #[allow(clippy::unknown_clippy_lints)]
        #[allow(clippy::reversed_empty_ranges)] // 1.45 lint
        Self {
            range: EMPTY_RANGE,
            style: Default::default(),
            text_style: Default::default(),
            blocks_before: 0..0,
            text_before: 0..0,
        }
    }
}

impl Placeholder {
    /// Create a new placeholder, see the fields of [Placeholder] for more information.
    pub fn new(
        range: TextRange,
        style: PlaceholderStyle,
        text_style: TextStyle,
        blocks_before: BlockRange,
        text_before: TextRange,
    ) -> Self {
        Self {
            range,
            style,
            text_style,
            blocks_before,
            text_before,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Block, Decoration, NativeTransmutable, Placeholder, PlaceholderAlignment, PlaceholderStyle,
        StyleType, TextDecorationMode, TextDecorationStyle,
    };

    #[test]
    fn text_decoration_style_naming() {
        let _ = TextDecorationStyle::Solid;
    }

    #[test]
    fn text_decoration_mode_naming() {
        let _ = TextDecorationMode::Gaps;
    }

    #[test]
    fn style_type_member_naming() {
        let _ = StyleType::Foreground;
        let _ = StyleType::LetterSpacing;
    }

    #[test]
    fn decoration_layout() {
        Decoration::test_layout();
    }

    #[test]
    fn placeholder_alignment_member_naming() {
        let _ = PlaceholderAlignment::Baseline;
        let _ = PlaceholderAlignment::AboveBaseline;
    }

    #[test]
    fn placeholder_style_layout() {
        PlaceholderStyle::test_layout()
    }

    #[test]
    fn block_layout() {
        Block::test_layout()
    }

    #[test]
    fn placeholder_layout() {
        Placeholder::test_layout()
    }
}
