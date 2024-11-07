mod binding;
mod each;
mod echo;
mod r#if;
mod var;
mod r#while;
mod writer;

use binding::Binding;
use std::fs;
use writer::Writer;
use yaml_rust2::{yaml::Hash, Yaml, YamlLoader};

pub struct Script {
    pub path: String,
    pub writer: Writer,
    pub break_opt: Option<String>,
    pub binding: Binding,
}

impl Script {
    pub fn new(path: String, log: Option<Vec<String>>) -> Self {
        Self {
            path,
            writer: Writer::new(log),
            break_opt: None,
            binding: Binding::new(),
        }
    }

    //-------------------------------------------------------------------------

    pub fn run(&mut self) {
        let text = fs::read_to_string(&self.path).unwrap();
        self.run_docs(YamlLoader::load_from_str(&text).unwrap());
    }

    fn run_docs(&mut self, docs: Vec<Yaml>) {
        for doc in docs {
            self.run_steps(doc.as_vec().expect("expected list"));

            if let Some(s) = &self.break_opt {
                panic!("{s}");
            }
        }
    }

    fn run_steps(&mut self, steps: &Vec<Yaml>) {
        for step in steps {
            self.run_step(step.as_hash().expect("expected mapping"));

            if self.break_opt.is_some() {
                break;
            }
        }
    }

    fn run_step(&mut self, step: &Hash) {
        // ???: Need better option for verbose.
        // println!("{step:?}");
        let token = step.iter().next().unwrap();
        let key = token.0.as_str().unwrap();

        match key {
            "echo" => echo::run(self, token.1),
            "if" => r#if::run(self, token.1, step),
            "while" => r#while::run(self, token.1, step),
            "each" => each::run(self, token.1, step),
            "break" => self.do_break(token.1, step),
            _ => var::run(self, key, token.1),
        }
    }

    //-------------------------------------------------------------------------

    // - break: [<condition>]
    //   [message: <string>]
    fn do_break(&mut self, cond: &Yaml, step: &Hash) {
        if self.binding.is_truthy(cond) {
            let message = self.message_string(step, "(break)");
            self.break_opt = Some(message);
        }
    }

    fn message_string(&mut self, step: &Hash, def: &str) -> String {
        let message = Yaml::from_str("message".into());
        let def_yaml = Yaml::from_str(&def);
        let message_yaml = step.get(&message).unwrap_or(&def_yaml);

        Binding::value_to_string(Binding::yaml_to_value(message_yaml))
    }
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_break() {
        let mut script = Script::new(String::new(), None);
        let docs = YamlLoader::load_from_str("foo:").unwrap();

        script.do_break(&Yaml::from_str("true"), &docs[0].as_hash().unwrap());
        assert_eq!(Some("(break)".into()), script.break_opt);
    }

    #[test]
    fn do_break_message() {
        let mut script = Script::new(String::new(), None);
        let message_yaml = YamlLoader::load_from_str("message: foo").unwrap();

        script.do_break(&Yaml::from_str("true"), &message_yaml[0].as_hash().unwrap());
        assert_eq!(Some("foo".into()), script.break_opt);
    }

    //-------------------------------------------------------------------------

    #[test]
    fn run_step() {
        // ???: Need to run each step type?
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let docs = YamlLoader::load_from_str("echo: foo").unwrap();
        let step = docs[0].as_hash().unwrap();

        script.run_step(&step);
        assert_eq!("foo", script.writer.log[0]);
    }

    #[test]
    fn run_steps() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let docs = YamlLoader::load_from_str("[a: 42, echo: foo]").unwrap();
        let steps = docs[0].as_vec().unwrap();

        script.run_steps(&steps);
        assert_eq!(42, script.binding.get("a").as_i64().unwrap());
        assert_eq!("foo", script.writer.log[0]);
    }

    //-------------------------------------------------------------------------

    #[test]
    fn run_docs_while_break() {
        let mut script = Script::new(String::new(), None);
        let docs = YamlLoader::load_from_str("[{while: true, do: [break: true]}]").unwrap();

        script.run_docs(docs);
        assert!(script.break_opt.is_none());
    }

    #[test]
    fn run_docs_each_break() {
        let mut script = Script::new(String::new(), None);
        let docs = YamlLoader::load_from_str("[{each: x, in: [1, 2], do: [break: true]}]").unwrap();

        script.run_docs(docs);
        assert!(script.break_opt.is_none());
    }

    #[test]
    #[should_panic]
    fn run_docs_break() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let docs = YamlLoader::load_from_str("[break: true]").unwrap();

        script.run_docs(docs);
    }
}
