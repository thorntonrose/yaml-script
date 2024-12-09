use super::{Binding, Script};
use std::io::Error;
use yaml_rust2::{yaml::Hash, Yaml};

// - def: <name>
//   do: <steps>
pub fn run(s: &mut Script, name: &Yaml, step: &Hash) -> Result<(), Error> {
    // ???: Need validation. Name must be an identifier.
    let steps = Yaml::Array(Binding::entry_to_list(step, "do"));
    s.binding.set_proc(name.as_str().expect("expected string"), steps);

    Ok(())
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run() {
        let mut script = Script::new(String::new(), None);
        let hash = Binding::hash_from_str("do: [a: 42]");
        let steps = Yaml::Array(Binding::entry_to_list(&hash, "do"));

        super::run(&mut script, &Yaml::from_str("foo"), &hash).unwrap();
        assert_eq!(steps, script.binding.proc("foo"));
    }
}
