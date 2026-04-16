use intermediate_representation::{Justification, YReference};

use crate::{
    LoweringContext,
    elements::{PositionedGlyph, Rect},
};

/// Build the full line/word/glyph structure.
///
/// Glyphs are positioned left-to-right; `check_word_wrap` handles
/// reflowing a word onto the next line when it exceeds the field bound.
/// Whether that wrap happened is communicated back via the bool return,
/// so `to_lines` can open a fresh line bucket for the completed word.
pub fn to_glyphs(
    font_id: &str,
    font_size: f32,
    starting_bounds: Rect,
    line_spacing: Option<f32>,
    justification: Justification,
    y_reference: YReference,
    text: &str,
    ctx: &LoweringContext,
) -> Vec<PositionedGlyph> {
    let mut word: Vec<PositionedGlyph> = Vec::new();
    let mut result = Vec::new();

    let loaded_font = ctx.fonts.get(font_id);

    let mut current_x = starting_bounds.x;
    let mut current_y = starting_bounds.y;

    // let line_metrics = loaded_font.font.horizontal_line_metrics(font_size).unwrap();

    // let cap_height = {
    //     //This works because 'H' has no ascenders or descenders
    //     // its top edge is the cap height by definition.
    //     let idx = loaded_font.font.lookup_glyph_index('H');
    //     let m = loaded_font.font.metrics_indexed(idx, font_size);
    //     // ymin is signed distance from baseline to bottom of bitmap
    //     // adding height gives the distance from baseline to top
    //     m.ymin as f32 + m.height as f32
    // };
    // let baseline_y = match y_reference {
    //     YReference::Baseline => starting_bounds.y,
    //     YReference::CapHeight => starting_bounds.y + cap_height,
    //     YReference::Ascent => starting_bounds.y + line_metrics.ascent,
    //     YReference::Bottom => starting_bounds.y + line_metrics.descent,
    // };
    // let mut current_y = baseline_y;

    let line_spacing = line_spacing.unwrap_or(font_size as f32 * 0.65);

    for ch in text.chars() {
        let glyph_index = loaded_font.font.lookup_glyph_index(ch);
        let metrics = loaded_font.font.metrics_indexed(glyph_index, font_size);

        // check if word would wrap
        let right_bound = starting_bounds.width + starting_bounds.x;
        let glyph_end = current_x + metrics.advance_width;
        if glyph_end > right_bound {
            // set the current_x to the beginning
            current_x = starting_bounds.x;
            // set the current_y to the new line
            current_y += line_spacing as f32;
            // move all glyphs in the current word
            for glyph in &mut word {
                glyph.x = current_x;
                glyph.y = current_y;
                current_x += glyph.advance_width
            }
        }

        // add new glyph to word
        let glyph = PositionedGlyph {
            glyph_index,
            x: current_x,
            y: current_y,
            advance_width: metrics.advance_width,
        };

        current_x += glyph.advance_width;
        word.push(glyph);

        // Space terminates the current word.
        if ch == ' ' {
            let completed: Vec<_> = word.drain(..).collect();
            result.extend(completed);
        }
    }

    result.extend(word.iter());

    apply_justification(&mut result, starting_bounds, justification);

    result
}

fn apply_justification(glyphs: &mut [PositionedGlyph], bounds: Rect, justification: Justification) {
    let lines = split_by_lines(glyphs);
    let last_line_idx = lines.len().saturating_sub(1);

    for (i, line) in lines.into_iter().enumerate() {
        let first = line.first().unwrap();
        let last = line.last().unwrap();
        let line_width = last.x - first.x + last.advance_width;

        let shift_x = match justification {
            Justification::Left => 0.0,
            Justification::Center => {
                let target_start = bounds.x + bounds.width / 2.0 - line_width / 2.0;
                target_start - first.x
            }
            Justification::Right => {
                let target_end = bounds.x + bounds.width;
                target_end - (last.x + last.advance_width) // this one was already correct
            }
            Justification::Justified if i == last_line_idx => 0.0,
            Justification::Justified => {
                continue;
            }
        };

        for glyph in line {
            glyph.x += shift_x;
        }
    }
}

fn split_by_lines(glyphs: &mut [PositionedGlyph]) -> Vec<Vec<&mut PositionedGlyph>> {
    let mut lines: Vec<Vec<&mut PositionedGlyph>> = Vec::new();
    if glyphs.is_empty() {
        return lines;
    }
    let mut current_line: Vec<&mut PositionedGlyph> = Vec::new();
    let mut current_y = glyphs.first().unwrap().y;
    for glyph in glyphs {
        if glyph.y != current_y {
            let line: Vec<_> = current_line.drain(..).collect();
            lines.push(line);
            current_y = glyph.y;
        }
        current_line.push(glyph);
    }

    let line: Vec<_> = current_line.drain(..).collect();
    lines.push(line);

    lines
}

#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use intermediate_representation::{ADWAITA_MONO, Justification, YReference};

    #[cfg(test)]
    use crate::elements::PositionedGlyph;
    use crate::{
        LoweringContext, RenderConfig,
        elements::Rect,
        font::FontStore,
        text::{apply_justification, split_by_lines, to_glyphs},
    };

    #[cfg(test)]
    fn make_ctx() -> LoweringContext {
        LoweringContext {
            fonts: FontStore::load_defaults(),
            config: RenderConfig::default(),
        }
    }

    #[cfg(test)]
    fn make_glyphs(text: &str, font_size: f32) -> Vec<PositionedGlyph> {
        let ctx = make_ctx();
        let bounds = ctx
            .fonts
            .measure_text_dimensions(ADWAITA_MONO, text, font_size);
        to_glyphs(
            ADWAITA_MONO,
            font_size,
            bounds,
            None,
            Justification::Left,
            YReference::Baseline,
            text,
            &ctx,
        )
    }

    #[cfg(test)]
    fn make_glyphs_bounded(text: &str, font_size: f32, max_width: f32) -> Vec<PositionedGlyph> {
        let ctx = make_ctx();
        let mut bounds = ctx
            .fonts
            .measure_text_dimensions(ADWAITA_MONO, text, font_size);
        bounds.width = max_width;
        to_glyphs(
            ADWAITA_MONO,
            font_size,
            bounds,
            None,
            Justification::Left,
            YReference::Baseline,
            text,
            &ctx,
        )
    }

    #[test]
    fn should_create_correct_num_of_glyphs() {
        let text = "some Text";
        let font_size = 12.;

        let glyphs = make_glyphs(text, font_size);

        assert_eq!(9, glyphs.len());
    }

    #[test]
    fn should_pick_the_correct_glyph_indexes() {
        let text = "some Text";
        let font_size = 12.;
        let glyphs = make_glyphs(text, font_size);

        let ctx = make_ctx();
        let loaded_font = ctx.fonts.get(ADWAITA_MONO);

        for (glyph, expected_ch) in glyphs.iter().zip(text.chars()) {
            let resolved_ch = loaded_font
                .font
                .chars()
                .iter()
                .find(|(_, idx)| **idx == NonZero::new(glyph.glyph_index).unwrap())
                .map(|(&ch, _)| ch)
                .expect("glyph_index not found in font");

            assert_eq!(
                resolved_ch, expected_ch,
                "reverse lookup failed for '{expected_ch}'"
            );
            // assert_eq!(glyph.ch, expected_ch);
        }
    }

    #[test]
    fn should_write_text_in_one_line() {
        let text = "This is some Text";
        let font_size = 12.;
        let glyphs = make_glyphs(text, font_size);

        for glyph in glyphs {
            assert_eq!(glyph.y, 0.)
        }
    }

    #[test]
    fn should_write_text_in_multiple_line() {
        let text = "This is some Text";
        let font_size = 12.;

        let ctx = make_ctx();
        let bounds = ctx
            .fonts
            .measure_text_dimensions(ADWAITA_MONO, text, font_size);

        let width = bounds.width / 2.;

        let glyphs = make_glyphs_bounded(text, font_size, width);

        let first_line_y = glyphs.first().unwrap().y;
        let last_line_y = glyphs.last().unwrap().y;
        assert!(last_line_y > first_line_y, "expected at least two lines");

        // first glyph of second line starts at x=0
        let second_line_start = glyphs.iter().find(|g| g.y == last_line_y).unwrap();
        assert_eq!(second_line_start.x, 0.0);
    }

    #[test]
    fn glyphs_increase_in_x_within_a_line() {
        let glyphs = make_glyphs("Hello", 12.0);
        let xs: Vec<f32> = glyphs.iter().map(|g| g.x).collect();
        for w in xs.windows(2) {
            assert!(w[1] > w[0], "x positions not increasing: {:?}", xs);
        }
    }

    #[test]
    fn should_return_one_lines() {
        let first = PositionedGlyph {
            glyph_index: 1,
            x: 0.,
            y: 0.,
            advance_width: 1.,
        };
        let second = PositionedGlyph {
            glyph_index: 2,
            x: 1.,
            y: 0.,
            advance_width: 1.,
        };
        let mut glyphs = vec![first, second];

        let result = split_by_lines(&mut glyphs);
        assert_eq!(result, vec![vec![&first, &second]])
    }

    #[test]
    fn should_return_two_lines() {
        let first = PositionedGlyph {
            glyph_index: 1,
            x: 0.,
            y: 0.,
            advance_width: 1.,
        };
        let second = PositionedGlyph {
            glyph_index: 2,
            x: 1.,
            y: 0.,
            advance_width: 1.,
        };
        let third = PositionedGlyph {
            glyph_index: 3,
            x: 0.,
            y: 1.,
            advance_width: 1.,
        };
        let mut glyphs = vec![first, second, third];

        let result = split_by_lines(&mut glyphs);
        assert_eq!(result, vec![vec![&first, &second], vec![&third]])
    }

    #[test]
    fn should_center_glyphs() {
        let first = PositionedGlyph {
            glyph_index: 1,
            x: 0.,
            y: 0.,
            advance_width: 2.,
        };
        let second = PositionedGlyph {
            glyph_index: 2,
            x: 2.,
            y: 0.,
            advance_width: 1.,
        };
        let third = PositionedGlyph {
            glyph_index: 3,
            x: 0.,
            y: 1.,
            advance_width: 1.,
        };
        let mut glyphs = vec![first, second, third];

        let bounds = Rect {
            x: 0.,
            y: 0.,
            width: 3.5,
            height: 2.,
        };
        apply_justification(&mut glyphs, bounds, Justification::Center);

        assert_eq!(glyphs[0].x, 0.25);
        assert_eq!(glyphs[1].x, 2.25);
        assert_eq!(glyphs[2].x, 1.25)
    }
}
