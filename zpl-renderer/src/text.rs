use core::f32;
use std::{error::Error, fmt::Debug};

use fontdue::Font;
use tiny_skia::{IntSize, Pixmap, PixmapPaint, Transform};
use zpl_interpreter::FieldBlock;
use zpl_parser::{Justification, TextBlockJustification};

use crate::{Drawable, Position};

impl Position {
    pub(crate) fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct FontConfig {
    font: Font,
    font_width: f32,
    font_height: f32,
    bold: bool,
    scale: f32,
}

impl FontConfig {
    pub(crate) fn new(
        font: Font,
        font_width: f32,
        font_height: f32,
        scale: f32,
        bold: bool,
    ) -> Self {
        Self {
            font,
            font_width,
            font_height,
            bold,
            scale,
        }
    }
}

#[derive(Clone, PartialEq)]
struct Glyph {
    ch: char,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    advance_width: f32,
    xmin: isize,
    ymin: isize,
    bitmap: Vec<u8>,
}

impl Debug for Glyph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Glyph")
            .field("char", &self.ch)
            .field("x", &self.x)
            .field("y", &self.y)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("advance_width", &self.advance_width)
            .field("xmin", &self.xmin)
            .field("ymin", &self.ymin)
            .field("bitmap", &"...")
            .finish()
    }
}

impl Glyph {
    fn new(font_config: &FontConfig, ch: char) -> Self {
        let (metrics, bitmap) = font_config.font.rasterize(ch, font_config.font_height);
        Self {
            ch,
            x: 0,
            y: 0,
            width: metrics.width,
            height: metrics.height,
            advance_width: metrics.advance_width,
            xmin: metrics.xmin as isize,
            ymin: metrics.ymin as isize,
            bitmap,
        }
    }

    fn right_bound(&self) -> usize {
        self.x + self.advance_width.round() as usize
    }

    fn start_at(&mut self, position: Position) {
        self.x = position.x;
        self.y = position.y;
    }

    fn position_next_to(&mut self, previous: &Self) {
        self.x = (previous.x + previous.advance_width.round() as usize) as usize;
        let height_diff = previous.height as isize - self.height as isize;
        let ymin_diff = previous.ymin - self.ymin;
        self.y = (previous.y as isize + height_diff + ymin_diff) as usize;
    }

    fn to_rbga(&self) -> Vec<u8> {
        let w = self.width as u32;
        let h = self.height as u32;
        let mut buf = Vec::with_capacity((w * h * 4) as usize);
        for &alpha in &self.bitmap {
            buf.push(0);
            buf.push(0);
            buf.push(0);
            buf.push(alpha);
        }
        buf
    }

    fn to_pixmap(&self) -> Result<Pixmap, Box<dyn std::error::Error>> {
        let buf = self.to_rbga();
        let w = self.width as u32;
        let h = self.height as u32;
        let size = IntSize::from_wh(w, h).ok_or("Invalid size")?;
        let pixmap = Pixmap::from_vec(buf, size).ok_or("Data not matching size")?;
        Ok(pixmap)
    }
}

impl Drawable for Glyph {
    fn draw(&self, target: &mut Pixmap) -> Result<(), Box<dyn Error>> {
        if self.height > 0 && self.width > 0 {
            let glyph_pixmap = self.to_pixmap()?;
            target.draw_pixmap(
                self.x as i32,
                self.y as i32,
                glyph_pixmap.as_ref(),
                &PixmapPaint::default(),
                Transform::identity(),
                None,
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldBox {
    width: usize,
    lines: usize,
    line_spacing: isize,
    justification: TextBlockJustification,
    hanging_indent: usize,
}

impl FieldBox {
    fn new(
        width: usize,
        lines: usize,
        line_spacing: isize,
        justification: TextBlockJustification,
        hanging_indent: usize,
    ) -> Self {
        Self {
            width,
            lines,
            line_spacing,
            justification,
            hanging_indent,
        }
    }
}

impl From<&FieldBlock> for FieldBox {
    fn from(value: &FieldBlock) -> Self {
        FieldBox {
            width: value.width,
            lines: value.lines,
            line_spacing: value.line_spacing,
            justification: value.justification,
            hanging_indent: value.hanging_indent,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    content: String,
    font_config: FontConfig,
    position: Position,
    justification: Justification,
    field_box: Option<FieldBox>,
}

/// Line → word → glyph. Each inner Vec<Glyph> is one word (including its
/// trailing space character, if any). Each outer Vec is one visual line.
type Lines = Vec<Vec<Vec<Glyph>>>;
type Word = Vec<Glyph>;

impl Text {
    pub(crate) fn new(
        content: String,
        font_config: FontConfig,
        position: Position,
        justification: Justification,
        field_box: Option<FieldBox>,
    ) -> Self {
        Self {
            content,
            font_config,
            position,
            justification,
            field_box,
        }
    }

    fn right_field_bound(&self) -> Option<usize> {
        self.field_box.as_ref().map(|b| self.position.x + b.width)
    }

    // -----------------------------------------------------------------------
    // Core layout: build lines of words
    // -----------------------------------------------------------------------

    /// Build the full line/word/glyph structure.
    ///
    /// Glyphs are positioned left-to-right; `check_word_wrap` handles
    /// reflowing a word onto the next line when it exceeds the field bound.
    /// Whether that wrap happened is communicated back via the bool return,
    /// so `to_lines` can open a fresh line bucket for the completed word.
    fn to_lines(&self) -> Lines {
        let mut lines: Lines = vec![vec![]];
        let mut word: Word = Vec::new();
        let mut word_was_wrapped = false;

        for ch in self.content.chars() {
            let mut glyph = Glyph::new(&self.font_config, ch);

            // Position relative to whatever came last.
            if let Some(prev) = word.last() {
                glyph.position_next_to(prev);
            } else if let Some(prev) = lines.last().and_then(|l| l.last()).and_then(|w| w.last()) {
                glyph.position_next_to(prev);
            } else {
                glyph.start_at(self.position);
            }

            word.push(glyph);

            if self.check_word_wrap(&mut word) {
                word_was_wrapped = true;
            }

            // Space terminates the current word.
            if ch == ' ' {
                let completed: Vec<Glyph> = word.drain(..).collect();
                if word_was_wrapped {
                    lines.push(vec![completed]);
                } else {
                    lines.last_mut().unwrap().push(completed);
                }
                word_was_wrapped = false;
            }
        }

        // Flush whatever remains (the last word, if no trailing space).
        if !word.is_empty() {
            self.check_word_wrap(&mut word);
            if word_was_wrapped {
                lines.push(vec![word]);
            } else {
                lines.last_mut().unwrap().push(word);
            }
        }

        lines
    }

    /// Reflow `word` onto the next line when its last glyph exceeds the right
    /// field bound.  Returns `true` when a wrap was actually performed.
    fn check_word_wrap(&self, word: &mut Word) -> bool {
        let last = word.last().expect("check_word_wrap called on empty word");

        if !self
            .right_field_bound()
            .is_some_and(|rb| last.right_bound() > rb)
        {
            return false;
        }

        // Move the whole word down by one line height, starting at the
        // field's left edge.
        let mut position = self.position;
        position.y += self.font_config.font_height.round() as usize;

        let mut iter = word.iter_mut();
        let mut prev = iter.next();
        if let Some(first) = prev.as_deref_mut() {
            first.start_at(position);
        }
        for next in iter {
            next.position_next_to(prev.unwrap());
            prev = Some(next);
        }

        true
    }

    // -----------------------------------------------------------------------
    // Justification
    // -----------------------------------------------------------------------

    /// Shift glyph x-coordinates on each line to satisfy `field_box.justification`.
    ///
    /// Called after `to_lines()`, before drawing. Mutates positions in-place.
    fn apply_justification(&self, lines: &mut Lines) {
        let field_box = match &self.field_box {
            Some(fb) => fb,
            None => return,
        };

        if field_box.justification == TextBlockJustification::Left {
            return; // already left-aligned, nothing to do
        }

        let last_line_idx = lines.len().saturating_sub(1);

        for (line_idx, line_words) in lines.iter_mut().enumerate() {
            if line_words.is_empty() {
                continue;
            }

            // Line content width: right bound of the furthest non-space glyph,
            // relative to the field's left edge.
            let content_right = line_words
                .iter()
                .flat_map(|w| w.iter())
                .filter(|g| g.ch != ' ')
                .map(|g| g.right_bound())
                .max()
                .unwrap_or(self.position.x);
            let line_width = content_right.saturating_sub(self.position.x);

            // Justified text treats its last line as Left (standard typography).
            let effective = if line_idx == last_line_idx
                && field_box.justification == TextBlockJustification::Justified
            {
                TextBlockJustification::Left
            } else {
                field_box.justification
            };

            match effective {
                TextBlockJustification::Left => {
                    // nothing
                }

                TextBlockJustification::Right => {
                    let offset = field_box.width.saturating_sub(line_width);
                    for glyph in line_words.iter_mut().flatten() {
                        glyph.x += offset;
                    }
                }

                TextBlockJustification::Center => {
                    let offset = field_box.width.saturating_sub(line_width) / 2;
                    for glyph in line_words.iter_mut().flatten() {
                        glyph.x += offset;
                    }
                }

                TextBlockJustification::Justified => {
                    // Distribute the leftover space evenly *between* words by
                    // adding a cumulative offset to each word after the first.
                    // The trailing-space glyph that's already part of each word
                    // provides the natural gap; this just stretches it.
                    let n_words = line_words.len();
                    if n_words <= 1 {
                        continue; // can't distribute with a single word
                    }
                    let extra = field_box.width.saturating_sub(line_width) as f32;
                    let gap = extra / (n_words - 1) as f32;
                    let mut cumulative = 0.0_f32;

                    for word in line_words.iter_mut() {
                        let shift = cumulative.round() as usize;
                        for glyph in word.iter_mut() {
                            glyph.x += shift;
                        }
                        cumulative += gap;
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Public helpers
    // -----------------------------------------------------------------------

    pub fn width(&self) -> usize {
        if let Some(fb) = &self.field_box {
            return fb.width;
        }

        // Without a field box, report the max right extent across all lines.
        let max_right = self
            .to_lines()
            .into_iter()
            .flatten()
            .flatten()
            .map(|g| g.right_bound())
            .max()
            .unwrap_or(self.position.x);

        max_right.saturating_sub(self.position.x)
    }

    fn draw_words(&self, target: &mut Pixmap) -> Result<(), Box<dyn Error>> {
        let mut lines = self.to_lines();
        self.apply_justification(&mut lines);
        for word in lines.into_iter().flatten() {
            for glyph in word {
                glyph.draw(target)?;
            }
        }
        Ok(())
    }
}

impl Drawable for Text {
    fn draw(&self, target: &mut Pixmap) -> Result<(), Box<dyn Error>> {
        self.draw_words(target)
    }
}

fn dilate_bitmap(bitmap: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut result = vec![0u8; bitmap.len()];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let mut max_alpha = bitmap[idx];

            // Check 8 neighbors + self
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let neighbor_idx = (ny as usize) * width + (nx as usize);
                        max_alpha = max_alpha.max(bitmap[neighbor_idx]);
                    }
                }
            }

            result[idx] = max_alpha;
        }
    }

    result
}

fn dilate_bitmap_hybrid(bitmap: &[u8], width: usize, height: usize) -> Vec<u8> {
    // First pass: Max filter for expansion
    let mut expanded = vec![0u8; bitmap.len()];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let mut max_alpha = bitmap[idx];

            // 4-connected for clean edges
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    let neighbor_idx = (ny as usize) * width + (nx as usize);
                    max_alpha = max_alpha.max(bitmap[neighbor_idx]);
                }
            }

            expanded[idx] = max_alpha;
        }
    }

    // Second pass: Gentle blur for smooth anti-aliasing
    let mut result = vec![0u8; bitmap.len()];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let mut sum = expanded[idx] as u32 * 4; // Center weight
            let mut count = 4u32;

            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    let neighbor_idx = (ny as usize) * width + (nx as usize);
                    sum += expanded[neighbor_idx] as u32;
                    count += 1;
                }
            }

            result[idx] = (sum / count) as u8;
        }
    }

    result
}

fn dilate_bitmap_hybrid_passes(
    bitmap: &[u8],
    width: usize,
    height: usize,
    passes: usize,
) -> Vec<u8> {
    let mut current = bitmap.to_vec();

    // Multiple expansion passes for consistent boldness
    for _ in 0..passes {
        let mut expanded = vec![0u8; bitmap.len()];

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let mut max_alpha = current[idx];

                // 4-connected for clean edges
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let neighbor_idx = (ny as usize) * width + (nx as usize);
                        max_alpha = max_alpha.max(current[neighbor_idx]);
                    }
                }

                expanded[idx] = max_alpha;
            }
        }

        current = expanded;
    }

    // Final pass: Gentle blur for smooth anti-aliasing
    let mut result = vec![0u8; bitmap.len()];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let mut sum = current[idx] as u32 * 4; // Center weight
            let mut count = 4u32;

            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    let neighbor_idx = (ny as usize) * width + (nx as usize);
                    sum += current[neighbor_idx] as u32;
                    count += 1;
                }
            }

            result[idx] = (sum / count) as u8;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use std::fs;

    use fontdue::FontSettings;
    use zpl_parser::TextBlockJustification;

    use crate::{
        Position,
        text::{FieldBox, FontConfig, Glyph, Text},
    };

    fn gen_font_config() -> FontConfig {
        let data = fs::read("../fonts/Oswald/Oswald-Medium.ttf").unwrap();
        let font = fontdue::Font::from_bytes(data, FontSettings::default()).unwrap();
        FontConfig::new(font, 20., 20., 1., false)
    }

    #[test]
    fn should_calcualte_correct_width_at_x_0() {
        let input = "This is a test with some content".into();
        let mut font_config = gen_font_config();
        let scale = 1.;
        font_config.scale = scale;
        let position = Position::new(0, 0);
        let justification = zpl_parser::Justification::Left;
        let text = Text::new(input, font_config, position, justification, None);

        let text_width = text.width();
        assert_eq!(text_width, 248);
    }

    #[test]
    fn should_calcualte_correct_width() {
        let input = "This is a test with some content".into();
        let mut font_config = gen_font_config();
        let scale = 0.85;
        font_config.scale = scale;
        let position = Position::new(200, 0);
        let justification = zpl_parser::Justification::Left;
        let text = Text::new(input, font_config, position, justification, None);

        let text_width = text.width();
        assert_eq!(text_width, 248);
    }

    #[test]
    fn should_respect_fielbox_settings() {
        let input = "This is a test with some content".into();
        let mut font_config = gen_font_config();
        let scale = 0.85;
        font_config.scale = scale;
        let position = Position::new(0, 0);
        let justification = zpl_parser::Justification::Left;
        let field_box = FieldBox::new(200, 2, 1, TextBlockJustification::Left, 0);
        let text = Text::new(input, font_config, position, justification, Some(field_box));

        let text_width = text.width();
        assert_eq!(text_width, 200);
    }

    #[test]
    fn test_glyph_position() {
        let font_config = gen_font_config();
        let position = Position::new(10, 20);
        let mut glyph_1 = Glyph::new(&font_config, 'A');
        glyph_1.start_at(position);
        let mut glyph_2 = Glyph::new(&font_config, 'c');
        glyph_2.position_next_to(&glyph_1);
        assert_eq!(glyph_1.y, 20);

        let height_diff = glyph_1.height as isize - glyph_2.height as isize;
        let ymin_diff = glyph_1.ymin - glyph_2.ymin;
        assert_eq!(20 + height_diff + ymin_diff, glyph_2.y as isize)
    }

    #[test]
    fn to_words_test() {
        let input = "S wo".into();
        let mut font_config = gen_font_config();
        let scale = 1.;
        font_config.scale = scale;
        let position = Position::new(0, 0);
        let justification = zpl_parser::Justification::Left;
        let text = Text::new(input, font_config.clone(), position, justification, None);

        let lines = text.to_lines();
        let glyph_1 = Glyph::new(&font_config, 'S');
        let mut glyph_2 = Glyph::new(&font_config, ' ');
        glyph_2.position_next_to(&glyph_1);
        let mut glyph_3 = Glyph::new(&font_config, 'w');
        glyph_3.position_next_to(&glyph_2);
        let mut glyph_4 = Glyph::new(&font_config, 'o');
        glyph_4.position_next_to(&glyph_3);
        assert_eq!(
            lines,
            vec![vec![vec![glyph_1, glyph_2], vec![glyph_3, glyph_4]]]
        );
    }

    #[test]
    fn to_words_with_ending_space_test() {
        let input = "S wo ".into();
        let mut font_config = gen_font_config();
        let scale = 1.;
        font_config.scale = scale;
        let position = Position::new(0, 0);
        let justification = zpl_parser::Justification::Left;
        let text = Text::new(input, font_config.clone(), position, justification, None);

        let words = text.to_lines();
        assert!(!words.last().unwrap().is_empty())
    }

    #[test]
    fn check_word_wrap_test() {
        let input = "".into();
        let mut font_config = gen_font_config();
        let scale = 1.;
        font_config.scale = scale;
        let position = Position::new(0, 0);
        let justification = zpl_parser::Justification::Left;
        let fielbox = FieldBox::new(200, 2, 1, TextBlockJustification::Left, 0);
        let text = Text::new(
            input,
            font_config.clone(),
            position,
            justification,
            Some(fielbox),
        );

        let glyph = Glyph {
            ch: 'S',
            x: 200,
            y: position.y,
            width: 10,
            height: 10,
            advance_width: 0.5,
            xmin: 0,
            ymin: 0,
            bitmap: vec![],
        };
        let mut word = vec![glyph.clone()];
        assert_eq!(word[0].x, glyph.x);
        assert_eq!(word[0].y, glyph.y);

        text.check_word_wrap(&mut word);

        assert_eq!(word[0].x, position.x);
        assert!(word[0].y > glyph.y);
    }

    #[test]
    fn word_wrap_test() {
        let input = "This is a test with some content".into();
        let mut font_config = gen_font_config();
        let scale = 1.;
        font_config.scale = scale;
        let position = Position::new(0, 0);
        let justification = zpl_parser::Justification::Left;
        let fielbox = FieldBox::new(200, 2, 1, TextBlockJustification::Left, 0);
        let text = Text::new(
            input,
            font_config.clone(),
            position,
            justification,
            Some(fielbox),
        );
        let lines = text.to_lines();

        let (first_line, second_line) = (lines.first().unwrap(), lines.iter().nth(1).unwrap());
        for glyph in first_line.iter().flatten() {
            assert!(glyph.y + glyph.height <= 20)
        }
        for glyph in second_line.iter().flatten() {
            assert!(glyph.y + glyph.height >= 20)
        }
    }
}
