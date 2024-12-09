use super::Script;
use std::io::Error;
use yaml_rust2::{
    yaml::{Array, Hash},
    Yaml,
};

// - call: foo
//   [with:
//      <name>: <expression>
//      ...]
pub fn run(s: &mut Script, name: &Yaml, step: &Hash) -> Result<(), Error> {
    let old_params = s.binding.set_params(with(step, "with"));
    let res = s.run_steps(&steps(s, name));
    s.binding.params = old_params;

    res
}

fn with(step: &Hash, key: &str) -> Hash {
    step.get(&Yaml::from_str(key))
        .unwrap_or(&Yaml::Hash(Hash::new()))
        .clone()
        .into_hash()
        .expect("expected mapping")
}

fn steps(s: &Script, name: &Yaml) -> Array {
    s.binding.proc(name.as_str().expect("expected string")).into_vec().unwrap()
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::super::Binding;
    use super::*;
    use yaml_rust2::{yaml::Hash, Yaml};

    #[test]
    fn run() {
        let mut script = Script::new(String::new(), None);
        script.run_str("[{def: foo, do: [a: 1]}]").unwrap();

        super::run(&mut script, &Yaml::from_str("foo"), &Hash::new()).unwrap();
        assert_eq!(1, script.binding.var("a").as_i64().unwrap());
    }

    #[test]
    fn run_with() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let hash = Binding::hash_from_str("with: {a: 1}");
        script.run_str("[{def: foo, do: [a: '${a + 1}', echo: '${a}']}]").unwrap();

        super::run(&mut script, &Yaml::from_str("foo"), &hash).unwrap();
        assert_eq!("2", script.writer.log[0]);
        assert_eq!(None, script.binding.vars.get("a"));
    }

    #[test]
    fn run_nested() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let hash = Binding::hash_from_str("with: {a: 1}");

        #[rustfmt::skip]
        let lines = vec![
            "- def: bar",
            "  do:",
            "    - a: ${a + 1}",
            "    - echo: 'bar: a=${a}, b=${b}'",
            "",
            "- def: foo",
            "  do:",
            "    - call: bar",
            "      with:",
            "        a: ${a}",
            "        b: 1",
            "    - echo: 'foo: a=${a}'",
        ];

        script.run_str(&lines.join("\n")).unwrap();

        super::run(&mut script, &Yaml::from_str("foo"), &hash).unwrap();
        assert_eq!("bar: a=2, b=1", script.writer.log[0]);
        assert_eq!("foo: a=1", script.writer.log[1]);
    }
}
