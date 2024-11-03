use eval::{Expr, Value};
use regex::{Match, Regex};
use serde_json::Number;
use std::collections::HashMap;
use std::fs;
use yaml_rust2::yaml::Hash;
use yaml_rust2::{Yaml, YamlLoader};

pub struct Script {
    pub path: String,
    pub vars: HashMap<String, Value>,
    pub log: Vec<String>,
    pub writer: fn(&mut Vec<String>, val: String),
}

impl Script {
    pub fn new(path: String, log: Option<Vec<String>>) -> Self {
        let writer = match log {
            Some(_) => Self::write_log,
            None => Self::write_stdout,
        };

        Self {
            path,
            vars: HashMap::new(),
            log: log.unwrap_or(Vec::new()),
            writer,
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

        for doc in docs {
            self.run_steps(doc.as_vec().unwrap());
        }
    }

    fn run_steps(&mut self, steps: &Vec<Yaml>) {
        for step in steps {
            self.run_step(step.as_hash().expect("expected mapping"))
        }
    }

    fn run_step(&mut self, step: &Hash) {
        // println!("{step:?}"); // ???: need better option for verbose
        let token = step.iter().next().unwrap();
        let key = token.0.as_str().unwrap();

        match key {
            "echo" => self.do_echo(token.1),
            "if" => self.do_if(token.1, step),
            // ...
            _ => self.do_var(&key.into(), token.1),
        }
    }

    //-------------------------------------------------------------------------

    fn do_var(&mut self, name: &String, yaml: &Yaml) {
        let val = self.yaml_to_value(yaml);
        self.vars.insert(name.into(), val);
    }

    //-------------------------------------------------------------------------

    fn do_echo(&mut self, yaml: &Yaml) {
        let val = self.eval(yaml);
        let val_str = self.value_to_string(val);
        self.write(val_str);
    }

    fn write(&mut self, val: String) {
        (self.writer)(&mut self.log, val);
    }

    //-------------------------------------------------------------------------

    fn do_if(&mut self, cond: &Yaml, step: &Hash) {
        let truthy = self.is_truthy(cond);
        let key = if truthy { "then" } else { "else" };
        let steps = step
            .get(&Yaml::from_str(key))
            .expect(format!("expected '{key}'").as_str());
        self.run_steps(steps.as_vec().unwrap());
    }

    fn is_truthy(&mut self, cond: &Yaml) -> bool {
        let val = self.eval(cond);
        println!("is_truthy: val: {val:?}");

        match val {
            Value::Bool(b) => b,
            Value::Number(n) => !self.is_zero(n),
            Value::String(s) => s.len() > 0,
            // ???: more?
            _ => false,
        }
    }

    fn is_zero(&mut self, num: Number) -> bool {
        (num.is_i64() && num.as_i64() == Some(0i64))
            || (num.is_f64() && num.as_f64() == Some(0.0f64))
    }

    //-------------------------------------------------------------------------

    fn eval(&mut self, yaml: &Yaml) -> Value {
        println!("eval: yaml: {yaml:?}");
        let val = self.yaml_to_value(yaml);

        match val {
            Value::String(s) => self.eval_expr(s),
            _ => val,
        }
    }

    fn eval_expr(&mut self, expr: String) -> Value {
        println!("eval: expr: {expr}");
        let re = Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();

        if re.is_match(&expr) {
            self.eval_interpolated(expr, re)
        } else {
            Value::String(expr.into())
        }
    }

    fn eval_interpolated(&mut self, expr: String, re: Regex) -> Value {
        let mut buf = expr.clone();

        for token in re.find_iter(&expr) {
            buf = self.interpolate(buf, token);
            println!("eval: buf: {buf}");
        }

        let val = Expr::new(buf).exec().unwrap();
        println!("eval: val: {val}");
        val
    }

    fn interpolate(&mut self, buf: String, token: Match<'_>) -> String {
        let placeholder = token.as_str().to_string();
        let var_name = placeholder.strip_prefix("$").unwrap().to_string();
        let var_val = self.vars.get(&var_name).unwrap_or(&Value::Null);

        let var_val_str = match var_val {
            Value::String(s) => format!("\"{s}\""),
            _ => self.value_to_string(var_val.clone()),
        };

        buf.replace(&placeholder, &var_val_str)
    }

    fn yaml_to_value(&mut self, yaml: &Yaml) -> Value {
        match yaml {
            Yaml::Boolean(b) => Value::Bool(*b),
            Yaml::String(s) => Value::String(s.into()),
            Yaml::Integer(i) => Value::Number((*i).into()),
            Yaml::Real(_) => Value::Number(Number::from_f64(yaml.as_f64().unwrap()).unwrap()),
            // ...
            _ => Value::String(format!("{yaml:?}")),
        }
    }

    fn value_to_string(&mut self, val: Value) -> String {
        match val {
            Value::String(s) => s.as_str().into(),
            _ => format!("{val}"),
        }
    }
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_expr() {
        let mut s = Script::new(String::new(), None);
        s.vars.insert("_".into(), Value::Number(42.into()));
        s.vars.insert("a".into(), Value::Number(42.into()));
        s.vars.insert("A".into(), Value::Number(42.into()));
        s.vars.insert("_aA0_".into(), Value::Number(42.into()));

        assert_eq!(42, s.eval_expr("$_".into()));
        assert_eq!(42, s.eval_expr("$a".into()));
        assert_eq!(42, s.eval_expr("$A".into()));
        assert_eq!(42, s.eval_expr("$_aA0_".into()));
    }

    //-------------------------------------------------------------------------

    #[test]
    fn do_var() {
        let mut s = Script::new(String::new(), None);
        let key = "a".to_string();

        s.do_var(&key, &Yaml::from_str("42"));
        assert_eq!(42, *s.vars.get(&key).unwrap());
    }

    //-------------------------------------------------------------------------

    #[test]
    fn do_echo() {
        let mut s = Script::new(String::new(), Some(Vec::new()));
        s.vars.insert("a".into(), Value::Number(42.into()));

        s.do_echo(&Yaml::from_str("$a + 1"));
        assert_eq!("43", s.log[0]);
    }

    //-------------------------------------------------------------------------

    #[test]
    fn is_truthy_true() {
        let mut s = Script::new(String::new(), None);

        for val in vec!["true", "1", "foo"] {
            assert_eq!(true, s.is_truthy(&Yaml::from_str(val)));
        }
    }

    #[test]
    fn is_truthy_false() {
        let mut s = Script::new(String::new(), None);

        for cond in vec![
            &Yaml::from_str("false"),
            &Yaml::from_str("0"),
            &Yaml::String("".into()),
        ] {
            assert_eq!(false, s.is_truthy(cond));
        }
    }

    #[test]
    fn do_if_true() {
        let mut s = Script::new(String::new(), None);
        let then_yaml = YamlLoader::load_from_str("then: [a: 42]").unwrap();

        s.do_if(&Yaml::from_str("true"), &then_yaml[0].as_hash().unwrap());
        assert_eq!(42, *s.vars.get("a").unwrap());
    }

    #[test]
    fn do_if_false() {
        let mut s = Script::new(String::new(), None);
        let else_yaml = YamlLoader::load_from_str("else: [a: 42]").unwrap();

        s.do_if(&Yaml::from_str("false"), &else_yaml[0].as_hash().unwrap());
        assert_eq!(42, *s.vars.get("a").unwrap());
    }

    //-------------------------------------------------------------------------

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
        assert_eq!(42, s.vars.get("a").unwrap().as_i64().unwrap());
        assert_eq!("foo", s.log[0]);
    }
}
