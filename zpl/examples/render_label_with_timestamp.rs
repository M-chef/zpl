use zpl_interpreter::interpret;
use zpl_parser::parse_zpl;
use zpl_renderer::render;

fn main() {
    // simple ZPL
    let zpl = "^XA^ST01,01,2025,,,,T^SLT,5^FC%^FT20,20^FD%m%Y%d %H:%M:%S^FS^CI27^XZ";
    let cmds = parse_zpl(zpl).unwrap();
    let label = interpret(&cmds);
    let out = render(&label);
    std::fs::write("label_with_timestamp.png", out.png).expect("write png");
    println!("Wrote label.png");
}
