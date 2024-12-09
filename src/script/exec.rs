use super::{var, Script};
use std::io::Error;
use std::process::{Command, Output};
use ternop::ternary;
use yaml_rust2::{yaml::Hash, Yaml};

// - exec: <expression>
//   [as: <name>]
pub fn run(s: &mut Script, expr: &Yaml, step: &Hash) -> Result<(), Error> {
    let text = text(command(s.binding.eval_to_string(expr)).output()?);
    var::run(s, var(step).unwrap_or("_".into()), &Yaml::String(text))
}

fn text(output: Output) -> String {
    String::from_utf8(bytes(output)).unwrap().strip_suffix("\n").unwrap().to_string()
}

fn bytes(output: Output) -> Vec<u8> {
    ternary!(output.status.code() == Some(0), output.stdout, output.stderr)
}

fn command(expr: String) -> Command {
    let tokens: Vec<&str> = expr.split_whitespace().collect();
    let mut command = Command::new(tokens[0]);
    command.args(ternary!(tokens.len() > 1, &tokens[1..], &[]));

    command
}

fn var(step: &Hash) -> Option<String> {
    step.get(&Yaml::from_str("as")).map(|e| e.as_str().expect("expected string").to_string())
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::super::binding::Binding;
    use super::*;

    #[test]
    fn run() {
        let mut script = Script::new(String::new(), None);

        super::run(&mut script, &Yaml::from_str("echo 1"), &Hash::new()).unwrap();
        assert_eq!("1", script.binding.var("_").as_str().unwrap());
    }

    #[test]
    fn run_as() {
        let mut script = Script::new(String::new(), None);
        let hash = Binding::hash_from_str("as: a");

        super::run(&mut script, &Yaml::from_str("echo 1"), &hash).unwrap();
        assert_eq!("1", script.binding.var("a").as_str().unwrap());
    }
}
