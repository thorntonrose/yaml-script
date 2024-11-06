use eval::Value;
use ternop::ternary;
use yaml_rust2::{yaml::Hash, Yaml};

use super::{binding::Binding, Script};

// - if: <condition>
//   [then: <steps>]
//   [else: <steps>]
//
// condition = <bool> | !0 | !""
pub fn run(script: &mut Script, cond: &Yaml, step: &Hash) {
    let key = ternary!(is_truthy(cond, &script.binding), "then", "else");
    let steps = Binding::hash_to_list(key, step);
    script.run_steps(&steps);
}

pub fn is_truthy(cond: &Yaml, binding: &Binding) -> bool {
    match binding.eval(cond) {
        Value::Bool(b) => b,
        Value::Number(n) => !Binding::is_zero(n),
        Value::String(s) => !s.is_empty(),
        // ???: more?
        _ => false,
    }
}

//=============================================================================

#[cfg(test)]
mod tests {
    use yaml_rust2::YamlLoader;

    use super::*;

    #[test]
    fn is_truthy() {
        let binding = Binding::new();

        for e in vec![
            (&Yaml::from_str("true"), true),
            (&Yaml::from_str("false"), false),
            (&Yaml::from_str("1"), true),
            (&Yaml::from_str("0"), false),
            (&Yaml::from_str("foo"), true),
            (&Yaml::String("".into()), false),
        ] {
            assert_eq!(e.1, super::is_truthy(e.0, &binding), "{e:?}");
        }
    }

    #[test]
    fn if_then() {
        let mut script = Script::new(String::new(), None);
        let docs = YamlLoader::load_from_str("then: [a: 42]").unwrap();
        let hash = docs[0].as_hash().unwrap();

        super::run(&mut script, &Yaml::from_str("true"), &hash);
        assert_eq!(42, script.binding.get("a"));
    }

    #[test]
    fn if_else() {
        let mut script = Script::new(String::new(), None);
        let docs = YamlLoader::load_from_str("else: [a: 42]").unwrap();
        let hash = docs[0].as_hash().unwrap();

        super::run(&mut script, &Yaml::from_str("false"), &hash);
        assert_eq!(42, script.binding.get("a"));
    }
}
