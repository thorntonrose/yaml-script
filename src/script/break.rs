use super::Script;
use std::io::{Error, ErrorKind::Interrupted};
use yaml_rust2::{yaml::Hash, Yaml};

// - break: [<condition>]
//   [message: <string>]
pub fn run(s: &mut Script, cond: &Yaml, step: &Hash) -> Result<(), Error> {
    match s.binding.is_truthy(cond) {
        true => Err(Error::new(Interrupted, message(step))),
        false => Ok(()),
    }
}

fn message(step: &Hash) -> String {
    step.get(&Yaml::from_str("message"))
        .unwrap_or(&Yaml::from_str("(break)"))
        .as_str()
        .unwrap()
        .to_string()
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::super::binding::Binding;
    use super::*;

    #[test]
    fn run() {
        let mut script = Script::new(String::new(), None);

        let err = super::run(&mut script, &Yaml::from_str("true"), &Hash::new()).unwrap_err();
        assert_eq!(Interrupted, err.kind());
        assert_eq!("(break)", err.to_string());
    }

    #[test]
    fn run_message() {
        let mut script = Script::new(String::new(), None);
        let hash = Binding::hash_from_str("message: foo");

        let err = super::run(&mut script, &Yaml::from_str("true"), &hash).unwrap_err();
        assert_eq!("foo", err.to_string());
    }
}
