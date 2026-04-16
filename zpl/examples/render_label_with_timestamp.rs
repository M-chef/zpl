use intermediate_representation::ADWAITA_MONO;
use zpl::{ZplError, ZplErrorKind};
use zpl_interpreter::interpret;
use zpl_parser::parse_zpl;
use zpl_renderer::{FontStore, LoweringContext, RenderConfig, render};

fn main() {
    // simple ZPL
    let zpl = "^XA^ST01,01,2025,,,,T^SLT,5^FC%^FT20,20^FD%m%Y%d %H:%M:%S^FS^CI27^XZ";
    let commands = parse_zpl(zpl).unwrap();
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
