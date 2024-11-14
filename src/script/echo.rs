use super::Script;
use std::io::Error;
use yaml_rust2::Yaml;

// - echo: <expression>
pub fn run(script: &mut Script, yaml: &Yaml) -> Result<(), Error> {
    write(script, script.binding.eval_to_string(yaml))
}

pub fn write(script: &mut Script, s: String) -> Result<(), Error> {
    script.writer.write(s);
    Ok(())
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

        super::run(&mut script, &Yaml::from_str("answer: ${a + 1}")).unwrap();
        assert_eq!("answer: 42", script.writer.log[0]);
    }
}
