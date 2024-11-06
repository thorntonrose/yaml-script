use super::Script;
use yaml_rust2::Yaml;

// - echo: <expression>
pub fn run(script: &mut Script, yaml: &Yaml) {
    script.writer.write(script.binding.eval_to_string(yaml));
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use eval::Value;

    #[test]
    fn run() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        script.binding.set("a", Value::Number(41.into()));

        super::run(&mut script, &Yaml::from_str("answer: ${a + 1}"));
        assert_eq!("answer: 42", script.writer.log[0]);
    }
}
