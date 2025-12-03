use std::fs::read_to_string;

use zpl::*;

fn main() {
    let zpl = read_to_string("zpl/examples/render_real_live_label.rs").unwrap();
    let (_, cmds) = parse_zpl(&zpl).unwrap();
    let elements = interpret(&cmds);
    let out = render(&elements, 600, 600);
    std::fs::write("label.png", out.png).expect("write png");
    println!("Wrote label.png");
}
