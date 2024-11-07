use super::{binding::Binding, var, Script};
use yaml_rust2::{yaml::Hash, Yaml};

// - each: <var>
//   in: <list>
//   do: <steps>
pub fn run(script: &mut Script, name: &Yaml, step: &Hash) {
    // ???: Need validation. Name must be identifier.
    let var_name = name.as_str().expect("expected string");
    let items = Binding::hash_to_list("in", step);
    let steps = Binding::hash_to_list("do", step);

    for item in items {
        var::run(script, var_name, &item);
        script.run_steps(&steps);

        if script.break_opt.is_some() {
            script.break_opt = None;
            break;
        }
    }
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

        super::run(&mut script, &Yaml::from_str("x"), &hash);
        assert_eq!(2, script.binding.get("x"));
        assert_eq!("1", script.writer.log[0]);
        assert_eq!("2", script.writer.log[1]);
    }
}
