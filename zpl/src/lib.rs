use intermediate_representation::ADWAITA_MONO;
use zpl_interpreter::*;
use zpl_parser::*;
use zpl_renderer::*;

mod error;

pub use error::*;

pub use zpl_interpreter::interpret;
pub use zpl_parser::parse_zpl;
pub use zpl_renderer::render;

pub struct ZplViewer;

impl ZplViewer {
    pub fn parse_and_render(input: &str) -> Result<RenderOutput, ZplError> {
        let commands = parse_zpl(input)?;
        let doc = interpret(&commands);
        let ctx = LoweringContext {
            fonts: FontStore::load_defaults(),
            config: RenderConfig {
                dpi: 1.,
                scale_factor: 1.,
                default_font_id: ADWAITA_MONO,
                background: intermediate_representation::Color::White,
            },
        };
        let result = render(&doc, &ctx).map_err(|err| ZplError {
            kind: ZplErrorKind::RenderError,
            message: err.to_string(),
        })?;
        Ok(result)
    }
}
