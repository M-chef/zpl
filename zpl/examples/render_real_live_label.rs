use std::fs::read_to_string;

use zpl::*;

fn main() {
    let zpl = read_to_string("zpl/examples/zpl_real_live.txt").unwrap();
    let cmds = parse_zpl(&zpl).unwrap();
    let label = interpret(&cmds);
    let out = render(&label);
    std::fs::write("label.png", out.png).expect("write png");
    println!("Wrote label.png");
}
