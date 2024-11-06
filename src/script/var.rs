use super::{binding::Binding, Script};
use yaml_rust2::Yaml;

pub fn run<S: Into<String>>(script: &mut Script, name: S, yaml: &Yaml) {
    // ???: Need validation. Name must be identifier.
    script.binding.set(name, Binding::yaml_to_value(yaml));
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn var() {
        let mut script = Script::new(String::new(), None);

        super::run(&mut script, "a", &Yaml::from_str("42"));
        assert_eq!(42, script.binding.get("a"));
    }
}
