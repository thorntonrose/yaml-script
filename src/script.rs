use std::fs;
use std::collections::HashMap;
use eval::{Expr, Value};
use serde_json::Number;
use regex::Regex;
use yaml_rust2::{Yaml, YamlLoader};
use yaml_rust2::yaml::Hash;

pub struct Script {
    pub path: String,
    pub vars: HashMap<String, Value>,
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
        // println!("{step:?}"); // ???: need better option for verbose
        let token = step.iter().next().unwrap();
        let key = token.0.as_str().unwrap();

        match key {
            "echo" => self.do_echo(token.1),
            _ => self.do_var(&key.to_string(), token.1)
        }
    }

    //-------------------------------------------------------------------------

    fn do_echo(&mut self, yaml: &Yaml) {
        let val = self.to_value(yaml);
        println!("echo: val: {val:?}");
        let val_string = self.value_string(val);
        self.write(val_string);
    }

    fn write(&mut self, val: String) {
        (self.writer)(&mut self.log, val);
    }

    //-------------------------------------------------------------------------

    fn do_var(&mut self, name: &String, yaml: &Yaml) {
        let val = self.to_value(yaml);
        self.vars.insert(name.to_string(), val);
    }

    fn to_value(&mut self, yaml: &Yaml) -> Value {
        match yaml {
            Yaml::Boolean(b) => Value::Bool(*b),
            Yaml::String(s) => self.eval(s.to_string()),
            Yaml::Integer(i) => Value::Number((*i).into()),
            Yaml::Real(_) => Value::Number(Number::from_f64(yaml.as_f64().unwrap()).unwrap()),
            _ => Value::String(format!("{yaml:?}"))
        }
    }

    fn eval(&mut self, expr: String) -> Value {
        println!("eval: expr: {expr}");
        let re = Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();

        if re.is_match(&expr) {
            self.expr_value(expr, re)
        } else {
            Value::String(expr.to_string())
        }
    }

    fn expr_value(&mut self, expr: String, re: Regex) -> Value {
        let mut buf = expr.clone();

        for token in re.find_iter(&expr) {
            let placeholder = token.as_str().to_string();
            let var_name = placeholder.strip_prefix("$").unwrap().to_string();
            let var_val = self.vars.get(&var_name).unwrap_or(&Value::Null);
            buf = buf.replace(&placeholder, self.value_string(var_val.clone()).as_str());
            println!("eval: buf: {buf}");
        }

        let val = Expr::new(buf).exec().unwrap();
        println!("eval: val: {val}");
        val
    }

    fn value_string(&mut self, val: Value) -> String {
        match val {
            Value::String(s) => s.as_str().to_string(),
            _ => format!("{val}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval() {
        let mut s = Script::new(String::new(), None);
        s.vars.insert("_".to_string(), Value::Number(42.into()));
        s.vars.insert("a".to_string(), Value::Number(42.into()));
        s.vars.insert("A".to_string(), Value::Number(42.into()));
        s.vars.insert("_aA0_".to_string(), Value::Number(42.into()));

        assert_eq!(42, s.eval("$_".to_string()));
        assert_eq!(42, s.eval("$a".to_string()));
        assert_eq!(42, s.eval("$A".to_string()));
        assert_eq!(42, s.eval("$_aA0_".to_string()));
    }

    #[test]
    fn do_var() {
        let mut s = Script::new(String::new(), None);
        let key = "a".to_string();

        s.do_var(&key, &Yaml::from_str("42"));
        assert_eq!(42, *s.vars.get(&key).unwrap());
    }

    #[test]
    fn do_var_expr() {
        let mut s = Script::new(String::new(), None);
        s.vars.insert("a".to_string(), Value::Number(42.into()));

        let key = "b".to_string();

        s.do_var(&key, &Yaml::from_str("$a"));
        assert_eq!(42, *s.vars.get(&key).unwrap());
    }

    #[test]
    fn do_echo() {
        let mut s = Script::new(String::new(), Some(Vec::new()));

        s.do_echo(&Yaml::from_str("foo"));
        assert_eq!("foo", s.log[0]);
    }

    #[test]
    fn do_echo_expr() {
        let mut s = Script::new(String::new(), Some(Vec::new()));
        s.vars.insert("a".to_string(), Value::Number(42.into()));

        s.do_echo(&Yaml::from_str("$a + 1"));
        assert_eq!("43", s.log[0]);
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
}
