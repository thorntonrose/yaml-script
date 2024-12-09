mod binding;
mod r#break;
mod call;
mod def;
mod each;
mod echo;
mod exec;
mod exit;
mod r#if;
mod step;
mod var;
mod r#while;
mod writer;

use binding::Binding;
use std::fs;
use std::io::{Error, ErrorKind::Interrupted};
use writer::Writer;
use yaml_rust2::{yaml::Array, Yaml, YamlLoader};

pub struct Script {
    pub path: String,
    pub binding: Binding,
    pub writer: Writer,
}

impl Script {
    pub fn new(path: String, log: Option<Vec<String>>) -> Self {
        Self {
            path,
            binding: Binding::new(),
            writer: Writer::new(log),
        }
    }

    //-------------------------------------------------------------------------

    pub fn run(&mut self) -> Result<(), Error> {
        match self.run_str(&fs::read_to_string(&self.path).unwrap()) {
            Err(e) if e.kind() == Interrupted => echo::write(self, e.to_string()),
            r => r,
        }
    }

    fn run_str(&mut self, text: &str) -> Result<(), Error> {
        self.run_docs(YamlLoader::load_from_str(text).unwrap())
    }

    fn run_docs(&mut self, docs: Vec<Yaml>) -> Result<(), Error> {
        for doc in docs {
            self.run_steps(doc.as_vec().expect("expected list"))?;
        }

        Ok(())
    }

    fn run_steps(&mut self, steps: &Array) -> Result<(), Error> {
        for step in steps {
            step::run(self, step.as_hash().expect("expected mapping"))?;
        }

        Ok(())
    }
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind::Interrupted;

    #[test]
    fn run_steps() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let docs = YamlLoader::load_from_str("[a: 42, echo: foo]").unwrap();
        let steps = docs[0].as_vec().unwrap();

        _ = script.run_steps(&steps);
        assert_eq!(42, script.binding.var("a").as_i64().unwrap());
        assert_eq!("foo", script.writer.log[0]);
    }

    //-------------------------------------------------------------------------

    #[test]
    fn run_docs_while_break() {
        let mut script = Script::new(String::new(), None);
        let docs = YamlLoader::load_from_str("[{while: true, do: [break: true]}]").unwrap();

        script.run_docs(docs).unwrap();
    }

    #[test]
    fn run_docs_each_break() {
        let mut script = Script::new(String::new(), None);
        let docs = YamlLoader::load_from_str("[{each: x, in: [1, 2], do: [break: true]}]").unwrap();

        script.run_docs(docs).unwrap();
    }

    #[test]
    fn run_docs_break() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let docs = YamlLoader::load_from_str("[break: true]").unwrap();

        let err = script.run_docs(docs).unwrap_err();
        assert_eq!(Interrupted, err.kind());
    }
}
