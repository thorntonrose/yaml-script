use super::Script;
use std::{io::Error, process::exit};
use yaml_rust2::Yaml;

// - exit: <number>
pub fn run(s: &Script, code: &Yaml) -> Result<(), Error> {
    run_step(s, code, exit)
}

#[allow(unreachable_code)]
fn run_step(s: &Script, code: &Yaml, halt: fn(i32) -> !) -> Result<(), Error> {
    halt(s.binding.eval_to_i32(code));
    Ok(())
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn run_step() {
        let script = Script::new(String::new(), None);
        let halt = |code: i32| -> ! { panic!("{code}") };

        super::run_step(&script, &Yaml::from_str("1"), halt).unwrap();
    }
}
