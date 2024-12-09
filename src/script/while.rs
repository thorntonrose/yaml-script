use super::{Binding, Script};
use std::io::{Error, ErrorKind::Interrupted};
use yaml_rust2::{
    yaml::{Array, Hash},
    Yaml,
};

// - while: <condition>
//   do:
//     <steps>
pub fn run(s: &mut Script, cond: &Yaml, step: &Hash) -> Result<(), Error> {
    match run_steps(s, cond, &Binding::entry_to_list(step, "do")) {
        Err(e) if e.kind() == Interrupted => Ok(()),
        r => r,
    }
}

pub fn run_steps(s: &mut Script, cond: &Yaml, steps: &Array) -> Result<(), Error> {
    while s.binding.is_truthy(cond) {
        s.run_steps(&steps)?;
    }

    Ok(())
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use yaml_rust2::yaml::Yaml;

    #[test]
    fn run() {
        let mut script = Script::new(String::new(), None);
        let hash = Binding::hash_from_str("do: [a: 42]");
        script.binding.set_var("a", Yaml::Integer(1));

        super::run(&mut script, &Yaml::from_str("${a == 1}"), &hash).unwrap();
        assert_eq!(42, script.binding.var("a").as_i64().unwrap());
    }
}
