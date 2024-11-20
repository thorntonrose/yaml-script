use super::{binding::Binding, var, Script};
use std::io::{Error, ErrorKind::Interrupted};
use yaml_rust2::{yaml::Hash, Yaml};

// - each: <var>
//   in: <list>
//   do: <steps>
pub fn run(script: &mut Script, name: &Yaml, step: &Hash) -> Result<(), Error> {
    match run_steps(
        script,
        // ???: Need validation. Name must be identifier.
        name.as_str().expect("expected string"),
        &Binding::entry_to_list(step, "in"),
        &Binding::entry_to_list(step, "do"),
    ) {
        Err(e) if e.kind() == Interrupted => Ok(()),
        r => r,
    }
}

pub fn run_steps(
    script: &mut Script,
    name: &str,
    items: &Vec<Yaml>,
    steps: &Vec<Yaml>,
) -> Result<(), Error> {
    for item in items {
        var::run(script, name, &item)?;
        script.run_steps(steps)?;
    }

    Ok(())
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use yaml_rust2::YamlLoader;

    #[test]
    fn run() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let docs = YamlLoader::load_from_str("{in: [1, 2], do: [echo: '${x}']}").unwrap();
        let hash = docs[0].as_hash().unwrap();

        super::run(&mut script, &Yaml::from_str("x"), &hash).unwrap();
        assert_eq!(2, script.binding.get("x"));
        assert_eq!("1", script.writer.log[0]);
        assert_eq!("2", script.writer.log[1]);
    }
}
