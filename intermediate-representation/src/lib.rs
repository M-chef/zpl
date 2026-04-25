mod barcode;
mod measure;

pub use barcode::*;
pub use measure::*;

pub const OSWALD: &'static str = "Oswald";
pub const ADWAITA_MONO: &str = "AdwaitaMono";
pub const OCR_B: &str = "OCR-B";

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Alignment {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Justification {
    #[default]
    Left,
    Center,
    Right,
    Justified,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Color {
    #[default]
    Black,
    White,
}

/// Describes how the Y coordinate of a text field should be interpreted
/// when converting from a source format into the internal baseline used
/// for glyph layout.
///
/// Different label and document formats define the Y origin of a text field
/// differently. This enum makes that semantic explicit so that [`to_glyphs`]
/// can normalize any source coordinate to a typographic baseline without
/// knowing which format it came from.
///
/// # Typographic reference points (top to bottom)
/// ```text
///  ___________  ← Ascent      (top of tallest possible glyph incl. diacritics)
///  |         |  ← Cap height  (top of flat capitals like H, I)
///  |  Hello  |
///  |_________|  ← Baseline    (the line glyphs sit on)
///       |       ← Bottom      (bottom of descenders like g, p, q)
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum YReference {
    /// The Y coordinate is the typographic baseline — the line that glyphs
    /// sit on. Descenders (g, p, q) extend below it.
    ///
    /// Used by: PostScript, PDF.
    Baseline,

    /// The Y coordinate is the top of flat capital letters (e.g. H, I).
    /// Roughly 66–70% of ascent for most fonts, but measured precisely
    /// from the font's own glyph metrics rather than approximated.
    ///
    /// Used by: ZPL (`^FO` Y coordinate).
    CapHeight,

    /// The Y coordinate is the full typographic ascent — the maximum height
    /// reserved above the baseline, including space for diacritics above
    /// capitals (Ä, Ö, etc.).
    ///
    /// Used by: REA-JET XML (verify against format spec).
    Ascent,

    Bottom,
}

#[derive(Clone, Default)]
pub struct DecodedBitmap {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>, // 0 = white, 1 = black
}

impl std::fmt::Debug for DecodedBitmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecodedBitmap")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("pixels", &"pixels...")
            .finish()
    }
}

impl DecodedBitmap {
    fn enlarge_modules_for_ean13(&mut self, module_width: usize) {
        for _ in 0..=self.height / 15 {
            self.add_module(1, module_width, 1u8);
            self.add_module(1, module_width, 0u8);
            self.add_module(1, module_width, 1u8);

            self.add_module(43, module_width, 0u8);

            self.add_module(1, module_width, 1u8);
            self.add_module(1, module_width, 0u8);
            self.add_module(1, module_width, 1u8);

            self.add_module(43, module_width, 0u8);

            self.add_module(1, module_width, 1u8);
            self.add_module(1, module_width, 0u8);
            self.add_module(1, module_width, 1u8);

            self.height += 1;
        }
    }

    fn add_module(&mut self, modules: usize, module_width: usize, kind: u8) {
        if kind != 0u8 && kind != 1u8 {
            panic!("no valid bitmap value")
        }
        let first = (0..module_width * modules).map(|_| kind);
        self.pixels.extend(first);
    }
}

#[derive(Debug, Clone)]
pub enum Element {
    Text {
        x: f32,
        y: f32,
        max_width: Option<f32>,
        lines: usize,
        font: String,
        font_size: f32,
        content: String,
        alignment: Alignment,
        justification: Justification,
        y_reference: YReference,
        inverted: bool,
    },
    Rectangle {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        thickness: f32,
        color: Color,
        rounding: u8,
        inverted: bool,
    },
    Image {
        x: f32,
        y: f32,
        bmp: DecodedBitmap,
    },
}

impl Element {
    pub fn is_inverted(&self) -> bool {
        match self {
            Element::Text { inverted, .. } => *inverted,
            Element::Rectangle { inverted, .. } => *inverted,
            Element::Image { .. } => false,
        }
    }
}

#[derive(Debug)]
pub struct Document {
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub elements: Vec<Element>,
}

pub struct BarcodeBuilder {
    pub x: f32,
    pub y: f32,
    pub symbology: Symbology,
    pub data: String,
    pub show_text: bool,
    pub width: usize,
    pub heigth: usize,
}

impl BarcodeBuilder {
    pub fn build(self) -> Vec<Element> {
        let mut elements = Vec::new();

        let bmp = match self.symbology {
            Symbology::Code39 => todo!(),
            Symbology::Code128 => generate_code_128(self.width, self.heigth, &self.data),
            Symbology::Ean13 => generate_ean13(self.width, self.heigth, &self.data),
            Symbology::Qr => todo!(),
            Symbology::DataMatrix => todo!(),
        }
        .unwrap_or_default();

        let text_elements = if self.show_text {
            match self.symbology {
                Symbology::Code39 => todo!(),
                Symbology::Code128 => generate_code_128_text(self.x, self.y, &self.data, &bmp),
                Symbology::Ean13 => {
                    generate_ean13_text(self.x, self.y, &self.data, &bmp).unwrap_or_default()
                }
                Symbology::Qr => todo!(),
                Symbology::DataMatrix => todo!(),
            }
        } else {
            vec![]
        };

        let img = Element::Image {
            x: self.x,
            y: self.y,
            bmp,
        };

        elements.push(dbg!(img));
        elements.extend(dbg!(text_elements));

        elements
    }
}
