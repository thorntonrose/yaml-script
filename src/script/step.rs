use super::{def, each, echo, exec, exit, r#break, r#if, r#while, var, Script};
use std::io::Error;
use yaml_rust2::yaml::Hash;

pub fn run(script: &mut Script, step: &Hash) -> Result<(), Error> {
    // example: ("echo", 1)
    let entry = step.iter().next().unwrap();
    let name = entry.0.as_str().unwrap();

    match name {
        "break" => r#break::run(script, entry.1, step),
        "def" => def::run(script, entry.1, step),
        "each" => each::run(script, entry.1, step),
        "echo" => echo::run(script, entry.1),
        "exec" => exec::run(script, entry.1, step),
        "exit" => exit::run(script, entry.1),
        "if" => r#if::run(script, entry.1, step),
        "while" => r#while::run(script, entry.1, step),
        _ => var::run(script, name, entry.1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yaml_rust2::YamlLoader;

    #[test]
    fn run() {
        // ???: Run each step type?
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let docs = YamlLoader::load_from_str("echo: foo").unwrap();
        let step = docs[0].as_hash().unwrap();

        _ = super::run(&mut script, &step);
        assert_eq!("foo", script.writer.log[0]);
    }
}
