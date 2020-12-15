#![deny(missing_docs)]

use super::{PositionWithAffinity, RectHeightStyle, RectWidthStyle, TextBox};
use crate::prelude::*;
use crate::textlayout::LineMetrics;
use crate::{scalar, Canvas, Point};
use skia_bindings as sb;
use std::ops::{Index, Range};

/// A simple multiline text block with homogenous text style. This must be created from a
/// [ParagraphBuilder].
pub type Paragraph = RefHandle<sb::skia_textlayout_Paragraph>;

unsafe impl Send for Paragraph {}
unsafe impl Sync for Paragraph {}

impl NativeDrop for sb::skia_textlayout_Paragraph {
    fn drop(&mut self) {
        unsafe { sb::C_Paragraph_delete(self) }
    }
}

impl Paragraph {
    /// Get the maximum width of the paragraph.
    pub fn max_width(&self) -> scalar {
        self.native().fWidth
    }

    /// Get the height of the paragraph (calculated from the text, style and width).
    pub fn height(&self) -> scalar {
        self.native().fHeight
    }

    /// > **TODO**: For Flutter
    pub fn min_intrinsic_width(&self) -> scalar {
        self.native().fMinIntrinsicWidth
    }

    /// > **TODO**: For Flutter
    pub fn max_intrinsic_width(&self) -> scalar {
        self.native().fMaxIntrinsicWidth
    }

    /// > **TODO**: For Flutter
    pub fn alphabetic_baseline(&self) -> scalar {
        self.native().fAlphabeticBaseline
    }

    /// > **TODO**: For Flutter
    pub fn ideographic_baseline(&self) -> scalar {
        self.native().fIdeographicBaseline
    }

    /// > **TODO**: For Flutter
    pub fn longest_line(&self) -> scalar {
        self.native().fLongestLine
    }

    /// True if the number of lines needed to layout the text exceeded the supplied maximum lines
    /// at build-time.
    pub fn did_exceed_max_lines(&self) -> bool {
        self.native().fExceededMaxLines
    }

    /// Reflow the text to the new supplied width.
    pub fn layout(&mut self, width: scalar) {
        unsafe { sb::C_Paragraph_layout(self.native_mut(), width) }
    }

    /// Draw this paragraph to the canvas at the supplied offset.
    pub fn paint(&self, canvas: &mut Canvas, p: impl Into<Point>) {
        let p = p.into();
        unsafe { sb::C_Paragraph_paint(self.native_mut_force(), canvas.native_mut(), p.x, p.y) }
    }

    /// Get the list of bounding boxes representing the area that would be drawn to
    /// when this paragraph is drawn to the canvas.
    pub fn get_rects_for_range(
        &self,
        range: Range<usize>,
        rect_height_style: RectHeightStyle,
        rect_width_style: RectWidthStyle,
    ) -> TextBoxes {
        TextBoxes::construct(|tb| unsafe {
            sb::C_Paragraph_getRectsForRange(
                self.native_mut_force(),
                range.start.try_into().unwrap(),
                range.end.try_into().unwrap(),
                rect_height_style,
                rect_width_style,
                tb,
            )
        })
    }

    pub fn get_rects_for_placeholders(&self) -> TextBoxes {
        TextBoxes::construct(|tb| unsafe {
            sb::C_Paragraph_getRectsForPlaceholders(self.native_mut_force(), tb)
        })
    }

    /// Returns the glyph at the position. The supplied point is relative to the top-left corner of
    /// the paragraph, with +y being down.
    ///
    /// See [PositionWithAffinity] for more information on the meaning of the returned value.
    pub fn get_glyph_position_at_coordinate(&self, p: impl Into<Point>) -> PositionWithAffinity {
        let p = p.into();
        let mut r = Default::default();
        unsafe {
            sb::C_Paragraph_getGlyphPositionAtCoordinate(self.native_mut_force(), p.x, p.y, &mut r)
        }
        r
    }

    /// Returns the glyph range that defines the word boundaries before and after the supplied offset
    /// in the paragraph.
    pub fn get_word_boundary(&self, offset: u32) -> Range<usize> {
        let mut range: [usize; 2] = Default::default();
        unsafe {
            sb::C_Paragraph_getWordBoundary(self.native_mut_force(), offset, range.as_mut_ptr())
        }
        range[0]..range[1]
    }

    /// Calculate a vector containing metrics about each line in the paragraph. See [LineMetricsVector] and
    /// [LineMetrics] for more information.
    pub fn get_line_metrics(&self) -> LineMetricsVector {
        Handle::<sb::LineMetricsVector>::construct(|lmv| unsafe {
            sb::C_Paragraph_getLineMetrics(self.native_mut_force(), lmv)
        })
        .borrows(self)
    }

    /// Returns the number of lines in the paragraph.
    pub fn line_number(&self) -> usize {
        unsafe { sb::C_Paragraph_lineNumber(self.native_mut_force()) }
    }

    /// Manually mark this paragraph as needing to have internal values recalculated. This should usually
    /// never need to be called by a consumer of this library.
    pub fn mark_dirty(&self) {
        unsafe { sb::C_Paragraph_markDirty(self.native_mut_force()) }
    }
}

/// An array of bounding boxes returned by [Paragraph].
pub type TextBoxes = Handle<sb::TextBoxes>;

impl NativeDrop for sb::TextBoxes {
    fn drop(&mut self) {
        unsafe { sb::C_TextBoxes_destruct(self) }
    }
}

impl Index<usize> for Handle<sb::TextBoxes> {
    type Output = TextBox;
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl AsRef<[TextBox]> for TextBoxes {
    fn as_ref(&self) -> &[TextBox] {
        self.as_slice()
    }
}

impl TextBoxes {
    pub fn iter(&self) -> impl Iterator<Item = &TextBox> {
        self.as_slice().iter()
    }

    pub fn as_slice(&self) -> &[TextBox] {
        unsafe {
            let mut count = 0;
            let ptr = sb::C_TextBoxes_ptr_count(self.native(), &mut count);
            std::slice::from_raw_parts(ptr as *const TextBox, count)
        }
    }
}

pub type LineMetricsVector<'a> = Borrows<'a, Handle<sb::LineMetricsVector>>;

impl NativeDrop for sb::LineMetricsVector {
    fn drop(&mut self) {
        unsafe { sb::C_LineMetricsVector_destruct(self) }
    }
}

impl<'a> Index<usize> for Borrows<'a, Handle<sb::LineMetricsVector>> {
    type Output = LineMetrics<'a>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<'a> AsRef<[LineMetrics<'a>]> for Borrows<'a, Handle<sb::LineMetricsVector>> {
    fn as_ref(&self) -> &[LineMetrics<'a>] {
        self.as_slice()
    }
}

impl<'a> Borrows<'a, Handle<sb::LineMetricsVector>> {
    pub fn iter(&self) -> impl Iterator<Item = &'a LineMetrics<'a>> {
        self.as_slice().iter()
    }

    pub fn as_slice(&self) -> &'a [LineMetrics<'a>] {
        unsafe {
            let mut count = 0;
            let ptr = sb::C_LineMetricsVector_ptr_count(self.native(), &mut count);
            std::slice::from_raw_parts(ptr as *const LineMetrics, count)
        }
    }
}

#[test]
#[serial_test::serial]
fn test_line_metrics() {
    // note: some of the following code is copied from the skparagraph skia-org example.
    use crate::icu;
    use crate::textlayout::{FontCollection, ParagraphBuilder, ParagraphStyle, TextStyle};
    use crate::FontMgr;

    icu::init();

    let mut font_collection = FontCollection::new();
    font_collection.set_default_font_manager(FontMgr::new(), None);
    let paragraph_style = ParagraphStyle::new();
    let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, font_collection);
    let ts = TextStyle::new();
    paragraph_builder.push_style(&ts);
    paragraph_builder.add_text(LOREM_IPSUM);
    let mut paragraph = paragraph_builder.build();
    paragraph.layout(256.0);

    let line_metrics = paragraph.get_line_metrics();
    for (line, lm) in line_metrics.iter().enumerate() {
        println!("line {}: width: {}", line + 1, lm.width)
    }

    static LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Curabitur at leo at nulla tincidunt placerat. Proin eget purus augue. Quisque et est ullamcorper, pellentesque felis nec, pulvinar massa. Aliquam imperdiet, nulla ut dictum euismod, purus dui pulvinar risus, eu suscipit elit neque ac est. Nullam eleifend justo quis placerat ultricies. Vestibulum ut elementum velit. Praesent et dolor sit amet purus bibendum mattis. Aliquam erat volutpat.";
}
