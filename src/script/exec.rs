use super::{var, Script};
use std::io::Error;
use std::process::{Command, Output};
use ternop::ternary;
use yaml_rust2::{yaml::Hash, Yaml};

// - exec: <expression>
//   [as: <name>]
pub fn run(script: &mut Script, expr: &Yaml, step: &Hash) -> Result<(), Error> {
    let text = text(command(script.binding.eval_to_string(expr)).output()?);
    var::run(script, var(step).unwrap_or("_".into()), &Yaml::String(text))
}

fn text(output: Output) -> String {
    String::from_utf8(bytes(output))
        .unwrap()
        .strip_suffix("\n")
        .unwrap()
        .to_string()
}

fn bytes(output: Output) -> Vec<u8> {
    match output.status.code() {
        Some(0) => output.stdout,
        _ => output.stderr,
    }
}

fn command(expr: String) -> Command {
    let tokens: Vec<&str> = expr.split_whitespace().collect();
    let mut command = Command::new(tokens[0]);
    command.args(ternary!(tokens.len() > 1, &tokens[1..], &[]));

    command
}

fn var(step: &Hash) -> Option<String> {
    match step.get(&Yaml::from_str("as")) {
        Some(entry) => Some(entry.as_str().expect("expected string").to_string()),
        None => None,
    }
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use yaml_rust2::yaml::YamlLoader;

    #[test]
    fn exec() {
        let mut script = Script::new(String::new(), None);
        let docs = YamlLoader::load_from_str("a:").unwrap();
        let hash = docs[0].as_hash().unwrap();

        super::run(&mut script, &Yaml::from_str("echo 1"), &hash).unwrap();
        assert_eq!("1", script.binding.get("_"));
    }

    #[test]
    fn exec_as() {
        let mut script = Script::new(String::new(), None);
        let docs = YamlLoader::load_from_str("as: a").unwrap();
        let hash = docs[0].as_hash().unwrap();

        super::run(&mut script, &Yaml::from_str("echo 1"), &hash).unwrap();
        assert_eq!("1", script.binding.get("a"));
    }
}
