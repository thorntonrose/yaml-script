use super::{binding::Binding, Script};
use yaml_rust2::{yaml::Hash, Yaml};

// - while: <condition>
//   do:
//     <steps>
pub fn run(script: &mut Script, cond: &Yaml, step: &Hash) {
    let steps = Binding::hash_to_list("do", step);

    while script.binding.is_truthy(cond) {
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
    use eval::Value;
    use yaml_rust2::YamlLoader;

    #[test]
    fn run() {
        let mut script = Script::new(String::new(), None);
        script.binding.set("a", Value::Number(1.into()));

        let docs = YamlLoader::load_from_str("do: [a: 42]").unwrap();
        let hash = docs[0].as_hash().unwrap();

        super::run(&mut script, &Yaml::from_str("${a == 1}"), &hash);
        assert_eq!(42, script.binding.get("a"));
    }
}
