use super::{binding::Binding, Script};
use std::io::Error;
use yaml_rust2::{yaml::Hash, Yaml};

// - def: <name>
//   do: <steps>
pub fn run(script: &mut Script, name: &Yaml, step: &Hash) -> Result<(), Error> {
    // ???: Need validation. Name must be an identifier.
    script.binding.set_proc(
        name.as_str().expect("expected string"),
        Binding::entry_to_list(step, "do"),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use yaml_rust2::YamlLoader;

    #[test]
    fn run() {
        let mut script = Script::new(String::new(), None);
        let docs = YamlLoader::load_from_str("do: [a: 42]").unwrap();
        let hash = docs[0].as_hash().unwrap();

        super::run(&mut script, &Yaml::from_str("foo"), &hash).unwrap();
        assert_eq!(
            Binding::entry_to_list(hash, "do"),
            script.binding.get_proc("foo")
        );
    }
}
