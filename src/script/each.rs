use super::{var, Binding, Script};
use std::io::{Error, ErrorKind::Interrupted};
use yaml_rust2::{
    yaml::{Array, Hash},
    Yaml,
};

// - each: <var>
//   in: <list>
//   do: <steps>
pub fn run(s: &mut Script, name: &Yaml, step: &Hash) -> Result<(), Error> {
    // ???: Need validation. Name must be an identifier.
    let var_name = name.as_str().expect("expected string");
    let items = Binding::entry_to_list(step, "in");
    let steps = Binding::entry_to_list(step, "do");

    match run_steps(s, var_name, &items, &steps) {
        Err(e) if e.kind() == Interrupted => Ok(()),
        res => res,
    }
}

pub fn run_steps(s: &mut Script, name: &str, items: &Array, steps: &Array) -> Result<(), Error> {
    for item in items {
        var::run(s, name, &item)?;
        s.run_steps(steps)?;
    }

    Ok(())
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let hash = Binding::hash_from_str("{in: [1, 2], do: [echo: '${x}']}");

        super::run(&mut script, &Yaml::from_str("x"), &hash).unwrap();
        assert_eq!(2, script.binding.var("x").as_i64().unwrap());
        assert_eq!("1", script.writer.log[0]);
        assert_eq!("2", script.writer.log[1]);
    }
}
