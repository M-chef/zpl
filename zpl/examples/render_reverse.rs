use std::fs::read_to_string;

use zpl_interpreter::interpret;
use zpl_parser::parse_zpl;
use zpl_renderer::render;

fn main() {
    let zpl = read_to_string("zpl/examples/reverse.txt").unwrap();
    let (_, cmds) = parse_zpl(&zpl).unwrap();
    let label = interpret(&cmds);
    let out = render(&label);
    std::fs::write("label.png", out.png).expect("write png");
    println!("Wrote label.png");
}
