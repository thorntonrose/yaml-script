use super::{Binding, Script};
use std::io::Error;
use ternop::ternary;
use yaml_rust2::{yaml::Hash, Yaml};

// - if: <condition>
//   [then: <steps>]
//   [else: <steps>]
pub fn run(s: &mut Script, cond: &Yaml, step: &Hash) -> Result<(), Error> {
    let key = ternary!(s.binding.is_truthy(cond), "then", "else");
    s.run_steps(&Binding::entry_to_list(step, key))
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_then() {
        let mut script = Script::new(String::new(), None);
        let hash = Binding::hash_from_str("then: [a: 42]");

        super::run(&mut script, &Yaml::from_str("true"), &hash).unwrap();
        assert_eq!(42, script.binding.var("a").as_i64().unwrap());
    }

    #[test]
    fn run_else() {
        let mut script = Script::new(String::new(), None);
        let hash = Binding::hash_from_str("else: [a: 42]");

        super::run(&mut script, &Yaml::from_str("false"), &hash).unwrap();
        assert_eq!(42, script.binding.var("a").as_i64().unwrap());
    }
}
