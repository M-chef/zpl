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
        let label = interpret(&commands);
        let result = render(&label);
        Ok(result)
    }
}
