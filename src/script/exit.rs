use super::Script;
use std::{io::Error, process::exit};
use yaml_rust2::Yaml;

pub fn run(script: &Script, yaml: &Yaml) -> Result<(), Error> {
    run_step(script, yaml, exit)
}

#[allow(unreachable_code)]
pub fn run_step(script: &Script, yaml: &Yaml, halt: fn(i32) -> !) -> Result<(), Error> {
    halt(script.binding.eval_to_i32(yaml));
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
