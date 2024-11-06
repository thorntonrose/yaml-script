mod script;

use script::Script;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("usage: ys <file>");
    } else {
        Script::new(args[1].clone(), None).run()
    }
}
