use zpl_interpreter::interpret;
use zpl_parser::parse_zpl;
use zpl_renderer::render;

fn main() {
    // simple ZPL
    let zpl = "^FO50,50^FDHello World^FS^FO50,100^FDSecond Line^FS";
    let (_, cmds) = parse_zpl(zpl).unwrap();
    let elements = interpret(&cmds);
    let out = render(&elements, 600, 300);
    std::fs::write("label.png", out.png).expect("write png");
    println!("Wrote label.png");
}
