use std::fs;
use std::collections::HashMap;
use eval::Expr;
use yaml_rust2::{Yaml, YamlLoader};
use yaml_rust2::yaml::Hash;

pub struct Script {
    pub path: String,
    pub vars: HashMap<String, Yaml>,
    pub log: Vec<String>,
    pub writer: fn(&mut Vec<String>, val: String)
}

impl Script {
    pub fn new(path: String, log: Option<Vec<String>>) -> Self {
        Self {
            path,
            vars: HashMap::new(),
            writer: match log { Some(_) => Self::write_log, None => Self::write_stdout },
            log: log.unwrap_or(Vec::new())
        }
    }

    fn write_log(log: &mut Vec<String>, val: String) {
        log.push(val);
    }

    fn write_stdout(_: &mut Vec<String>, val: String) {
        println!("{val}");
    }

    //-------------------------------------------------------------------------

    pub fn run(&mut self) {
        let text = fs::read_to_string(&self.path).unwrap();
        let docs = YamlLoader::load_from_str(&text).unwrap();

        self.run_steps(docs[0].as_vec().unwrap());
    }

    fn run_steps(&mut self, steps: &Vec<Yaml>) {
        for step in steps { self.run_step(step.as_hash().unwrap()) }
    }

    fn run_step(&mut self, step: &Hash) {
        // println!("{step:?}"); // ???: need better for verbose
        let token = step.iter().next().unwrap();
        let key = token.0.as_str().unwrap();

        match key {
            "echo" => self.do_echo(token.1),
            _ => self.do_var(&key.to_string(), token.1)
        }
    }

    fn do_echo(&mut self, val: &Yaml) {
        match val {
            Yaml::Boolean(b) => self.write(b.to_string()),
            Yaml::String(s) => self.write(s.to_string()),
            Yaml::Integer(i) => self.write(i.to_string()),
            Yaml::Real(r) => self.write(r.to_string()),
            _ => self.write(format!("{:?}", val))
        }
    }

    fn write(&mut self, val: String) {
        (self.writer)(&mut self.log, val);
    }

    fn do_var(&mut self, name: &String, val: &Yaml) {
        self.vars.insert(name.to_string(), val.clone());
    }

    fn eval_string(self, val: String) -> String {
        Expr::new(val).value("a", 42).exec().unwrap().as_i64().unwrap().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_var() {
        let mut s = Script::new(String::new(), None);
        let key = "a".to_string();

        s.do_var(&key, &Yaml::from_str("42"));
        assert_eq!(42, s.vars.get(&key).unwrap().as_i64().unwrap());
    }

    #[test]
    fn do_echo() {
        let mut s = Script::new(String::new(), Some(Vec::new()));

        s.do_echo(&Yaml::from_str("foo"));
        assert_eq!("foo", s.log[0]);
    }

    #[test]
    fn run_step() {
        let mut s = Script::new(String::new(), Some(Vec::new()));
        let docs = YamlLoader::load_from_str("echo: foo").unwrap();
        let step = docs[0].as_hash().unwrap();

        s.run_step(&step);
        assert_eq!("foo", s.log[0]);
    }

    #[test]
    fn run_steps() {
        let mut s = Script::new(String::new(), Some(Vec::new()));
        let docs = YamlLoader::load_from_str("[a: 42, echo: foo]").unwrap();
        let steps = docs[0].as_vec().unwrap();

        s.run_steps(&steps);
        assert_eq!(42, s.vars.get(&"a".to_string()).unwrap().as_i64().unwrap());
        assert_eq!("foo", s.log[0]);
    }

    #[test]
    fn eval_string() {
        let s = Script::new(String::new(), None);
        assert_eq!("43", s.eval_string("a + 1".to_string()));
    }
}
