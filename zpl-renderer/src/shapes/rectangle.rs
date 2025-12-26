use tiny_skia::{Paint, PathBuilder, Rect, Stroke, Transform};
use zpl_parser::Color;

use crate::{Drawable, Position};

pub struct RectDim {
    width: f32,
    height: f32,
    line_thickness: f32,
    rounding: u8,
}

impl RectDim {
    pub fn new(width: f32, height: f32, line_thickness: f32, rounding: u8) -> Self {
        Self {
            width,
            height,
            line_thickness,
            rounding,
        }
    }
}

pub(crate) struct Rectangle {
    position: Position,
    dim: RectDim,
    color: Color,
}

impl Rectangle {
    pub(crate) fn new(position: Position, dim: RectDim, color: Color) -> Self {
        Self {
            position,
            dim,
            color,
        }
    }
}

impl Drawable for Rectangle {
    fn draw(&self, target: &mut tiny_skia::Pixmap) -> Result<(), Box<dyn std::error::Error>> {
        let rect = Rect::from_xywh(
            self.position.x as f32,
            self.position.y as f32,
            self.dim.width,
            self.dim.height,
        )
        .unwrap();
        let inset = self.dim.line_thickness / 2.0;

        // thickness from zpl is not equal to stroke width
        // for thickness value equally to width and height this would lead
        // to a single point rect (i.e. (x: 1, y: 1, widh: 1, height: 1), not be drawn)
        // hence we must correct the inset in such cases
        // TODO: catch this earlier in the interpreter
        let inset = match inset >= self.dim.width / 2. && inset >= self.dim.height / 2. {
            true => inset - 0.1,
            false => inset,
        };
        let rect = rect.inset(inset, inset).unwrap();

        let mut pb = PathBuilder::new();
        pb.push_rect(rect);
        let path = pb.finish().unwrap();

        let mut paint = Paint::default();
        match self.color {
            Color::Black => paint.set_color_rgba8(0, 0, 0, 255),
            Color::White => paint.set_color_rgba8(255, 255, 255, 255),
        }

        let mut stroke = Stroke::default();
        stroke.width = self.dim.line_thickness;

        target.stroke_path(&path, &paint, &stroke, Transform::identity(), None);

        Ok(())
    }
}
