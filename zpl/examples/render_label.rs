use zpl_interpreter::interpret;
use zpl_parser::parse_zpl;
use zpl_renderer::render;

fn main() {
    // simple ZPL
    let zpl = "^FO50,50^FDHello World^FS^FO50,100^FDSecond Line^FS";
    let cmds = parse_zpl(zpl).unwrap();
    let label = interpret(&cmds);
    let out = render(&label);
    std::fs::write("label.png", out.png).expect("write png");
    println!("Wrote label.png");
}
