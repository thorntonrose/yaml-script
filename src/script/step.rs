use super::{call, def, each, echo, exec, exit, r#break, r#if, r#while, var, Script};
use std::io::Error;
use yaml_rust2::yaml::Hash;

pub fn run(s: &mut Script, step: &Hash) -> Result<(), Error> {
    // example: ("echo", 1)
    let entry = step.iter().next().unwrap();
    let name = entry.0.as_str().unwrap();

    match name {
        "break" => r#break::run(s, entry.1, step),
        "call" => call::run(s, entry.1, step),
        "def" => def::run(s, entry.1, step),
        "each" => each::run(s, entry.1, step),
        "echo" => echo::run(s, entry.1),
        "exec" => exec::run(s, entry.1, step),
        "exit" => exit::run(s, entry.1),
        "if" => r#if::run(s, entry.1, step),
        "while" => r#while::run(s, entry.1, step),
        _ => var::run(s, name, entry.1),
    }
}

#[cfg(test)]
mod tests {
    use super::super::binding::Binding;
    use super::*;

    #[test]
    fn run() {
        // ???: Run each step type?
        let mut script = Script::new(String::new(), Some(Vec::new()));

        _ = super::run(&mut script, &Binding::hash_from_str("echo: foo"));
        assert_eq!("foo", script.writer.log[0]);
    }
}
