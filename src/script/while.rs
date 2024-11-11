use std::io::{Error, ErrorKind::Interrupted};

use super::{binding::Binding, Script};
use yaml_rust2::{yaml::Hash, Yaml};

// - while: <condition>
//   do:
//     <steps>
pub fn run(script: &mut Script, cond: &Yaml, step: &Hash) -> Result<(), Error> {
    match run_steps(script, cond, &Binding::hash_to_list("do", step)) {
        Err(e) if e.kind() == Interrupted => Ok(()),
        r => r,
    }
}

pub fn run_steps(script: &mut Script, cond: &Yaml, steps: &Vec<Yaml>) -> Result<(), Error> {
    while script.binding.is_truthy(cond) {
        script.run_steps(&steps)?;
    }

    Ok(())
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use eval::Value;
    use yaml_rust2::YamlLoader;

    #[test]
    fn run() {
        let mut script = Script::new(String::new(), None);
        script.binding.set("a", Value::Number(1.into()));

        let docs = YamlLoader::load_from_str("do: [a: 42]").unwrap();
        let hash = docs[0].as_hash().unwrap();

        super::run(&mut script, &Yaml::from_str("${a == 1}"), &hash).unwrap();
        assert_eq!(42, script.binding.get("a"));
    }
}
