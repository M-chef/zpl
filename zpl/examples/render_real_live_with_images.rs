use std::fs::read_to_string;

use intermediate_representation::ADWAITA_MONO;
use zpl::*;
use zpl_renderer::{FontStore, LoweringContext, RenderConfig};

fn main() {
    let zpl = read_to_string("zpl/examples/render_with_images.txt").unwrap();
    let commands = parse_zpl(&zpl).unwrap();
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
    let out = render(&doc, &ctx)
        .map_err(|err| ZplError {
            kind: ZplErrorKind::RenderError,
            message: err.to_string(),
        })
        .unwrap();

    std::fs::write("label.png", out.png).expect("write png");
    println!("Wrote label.png");
}
