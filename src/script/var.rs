use super::Script;
use std::io::Error;
use yaml_rust2::Yaml;

pub fn run<S: Into<String>>(s: &mut Script, name: S, yaml: &Yaml) -> Result<(), Error> {
    // ???: Need validation. Name must be an identifier.
    s.binding.set_var(name, s.binding.eval_to_yaml(yaml));
    Ok(())
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run() {
        let mut script = Script::new(String::new(), None);

        super::run(&mut script, "a", &Yaml::from_str("42")).unwrap();
        assert_eq!(42, script.binding.var("a").as_i64().unwrap());

        super::run(&mut script, "b", &Yaml::from_str("${a + 1}")).unwrap();
        assert_eq!(43, script.binding.var("b").as_i64().unwrap());
    }
}
