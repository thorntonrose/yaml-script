use super::Script;
use std::io::Error;
use yaml_rust2::Yaml;

// - echo: <expression>
pub fn run(s: &mut Script, expr: &Yaml) -> Result<(), Error> {
    write(s, s.binding.eval_to_string(expr))
}

pub fn write(s: &mut Script, val: String) -> Result<(), Error> {
    s.writer.write(val);
    Ok(())
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use yaml_rust2::yaml::Yaml;

    #[test]
    fn run() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        script.binding.set_var("a", Yaml::Integer(41));

        super::run(&mut script, &Yaml::from_str("answer: ${a + 1}")).unwrap();
        assert_eq!("answer: 42", script.writer.log[0]);
    }
}
