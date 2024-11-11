mod script;

use script::Script;
use std::{env, io::Error};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("usage: ys <file>");
        Ok(())
    } else {
        Script::new(args[1].clone(), None).run()
    }
}
