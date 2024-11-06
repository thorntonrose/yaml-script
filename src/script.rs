mod writer;

use eval::{Expr, Value};
use regex::{Match, Regex};
use serde_json::Number;
use std::{collections::HashMap, fs};
use writer::Writer;
use yaml_rust2::{yaml::Hash, Yaml, YamlLoader};

pub struct Script {
    pub path: String,
    pub vars: HashMap<String, Value>,
    pub writer: Writer,
    pub break_opt: Option<String>,
}

impl Script {
    pub fn new(path: String, log_opt: Option<Vec<String>>) -> Self {
        Self {
            path,
            vars: HashMap::new(),
            writer: Writer::new(log_opt),
            break_opt: None,
        }
    }

    //-------------------------------------------------------------------------

    pub fn run(&mut self) {
        let text = fs::read_to_string(&self.path).unwrap();
        self.run_docs(YamlLoader::load_from_str(&text).unwrap());
    }

    fn run_docs(&mut self, docs: Vec<Yaml>) {
        for doc in docs {
            self.run_steps(doc.as_vec().unwrap());

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
            "echo" => self.do_echo(token.1),
            "if" => self.do_if(token.1, step),
            "while" => self.do_while(token.1, step),
            "break" => self.do_break(token.1, step),
            "each" => self.do_each(token.1, step),
            // ...
            _ => self.do_var(&key.into(), token.1),
        }
    }

    //-------------------------------------------------------------------------

    // - <name>: <value>
    fn do_var(&mut self, name: &String, yaml: &Yaml) {
        // ???: Need validation. Name must be indentifier.
        let val = self.yaml_to_value(yaml);
        self.vars.insert(name.into(), val);
    }

    //-------------------------------------------------------------------------

    // - echo: <expression>
    fn do_echo(&mut self, yaml: &Yaml) {
        let val = self.eval(yaml);
        let val_str = self.value_to_string(val);
        // self.write(val_str);
        self.writer.write(val_str);
    }

    // fn write(&mut self, val: String) {
    //     (self.writer)(&mut self.log, val);
    // }

    //-------------------------------------------------------------------------

    // - if: <condition>
    //   [then: <steps>]
    //   [else: <steps>]
    //
    // condition = <bool> | !0 | !""
    fn do_if(&mut self, cond: &Yaml, step: &Hash) {
        let key = if self.is_truthy(cond) { "then" } else { "else" };
        let steps = self.get_list(key, step);
        self.run_steps(&steps);
    }

    fn is_truthy(&mut self, cond: &Yaml) -> bool {
        match self.eval(cond) {
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

    fn get_list(&mut self, key: &str, step: &Hash) -> Vec<Yaml> {
        step.get(&Yaml::from_str(key))
            .expect(format!("expected '${key}'").as_str())
            .clone()
            .into_vec()
            .expect("expected list")
    }

    //-------------------------------------------------------------------------

    // - while: <truthy-expression>
    //   do:
    //     <steps>
    fn do_while(&mut self, cond: &Yaml, step: &Hash) {
        let steps = self.get_list("do", step);

        while self.is_truthy(cond) {
            self.run_steps(&steps);

            if self.break_opt.is_some() {
                self.break_opt = None;
                break;
            }
        }
    }

    //-------------------------------------------------------------------------

    // - each: <var>
    //   in: <list>
    //   do: <steps>
    fn do_each(&mut self, var: &Yaml, step: &Hash) {
        // ???: Need validation. Name must be indentifier.
        let var_name = var.as_str().expect("expected string").to_string();
        let items = self.get_list("in", step);
        let steps = self.get_list("do", step);

        for item in items {
            self.do_var(&var_name, &item);
            self.run_steps(&steps);

            if self.break_opt.is_some() {
                self.break_opt = None;
                break;
            }
        }
    }

    //-------------------------------------------------------------------------

    // - break: [<condition>]
    //   [message: <string>]
    fn do_break(&mut self, cond: &Yaml, step: &Hash) {
        let truthy = self.is_truthy(cond);

        if truthy {
            let message = self.message_string(step, "(break)");
            self.break_opt = Some(message);
        }
    }

    fn message_string(&mut self, step: &Hash, def: &str) -> String {
        let def_yaml = Yaml::from_str(&def);
        let message = step
            .get(&Yaml::from_str("message".into()))
            .unwrap_or(&def_yaml);

        let val = self.yaml_to_value(message);
        self.value_to_string(val)
    }

    //-------------------------------------------------------------------------

    fn eval(&mut self, yaml: &Yaml) -> Value {
        let val = self.yaml_to_value(yaml);

        match val {
            Value::String(s) => self.eval_expr(s),
            _ => val,
        }
    }

    fn eval_expr(&mut self, expr: String) -> Value {
        let re = Regex::new(r"\$\{[a-zA-Z0-9_\.+\-\*/%=<>!&| ]*\}").unwrap();

        match re.is_match(&expr) {
            true => self.eval_tokens(expr, re),
            false => Value::String(expr.into()),
        }
    }

    fn eval_tokens(&mut self, expr: String, re: Regex) -> Value {
        let mut buf = expr.clone();

        while let Some(m) = re.find(&buf) {
            buf.replace_range(m.start()..m.end(), self.eval_token(m).as_str());
        }

        self.yaml_to_value(&Yaml::from_str(&buf))
    }

    fn eval_token(&mut self, token: Match<'_>) -> String {
        let expr_str = token.as_str().replace("${", "").replace("}", "");
        let mut expr = Expr::new(expr_str);

        for (name, val) in &self.vars {
            expr = expr.value(name, val);
        }

        self.value_to_string(expr.exec().unwrap())
    }

    fn yaml_to_value(&mut self, yaml: &Yaml) -> Value {
        match yaml {
            Yaml::Boolean(b) => Value::Bool(*b),
            Yaml::String(s) => Value::String(s.into()),
            Yaml::Integer(i) => Value::Number((*i).into()),
            Yaml::Real(_) => Value::Number(Number::from_f64(yaml.as_f64().unwrap()).unwrap()),
            Yaml::Null => Value::Null,
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
    fn eval() {
        let mut script = Script::new(String::new(), None);
        script.vars.insert("a".into(), Value::Number(1.into()));
        script.vars.insert("b".into(), Value::Number(2.into()));

        for e in vec![
            ("0", Value::from(0)),
            ("1.0", Value::from(1.0)),
            ("true", Value::from(true)),
            ("${a}", Value::from(1)),
            ("${a + b}", Value::from(3)),
            ("${a}, ${b}", Value::from("1, 2")),
            ("a+b = ${a + b}", Value::from("a+b = 3")),
            ("${a == 1}", Value::from(true)),
            // ...
        ] {
            assert_eq!(e.1, script.eval(&Yaml::from_str(e.0)), "{e:?}");
        }
    }

    //-------------------------------------------------------------------------

    #[test]
    fn do_var() {
        let mut script = Script::new(String::new(), None);
        let key = "a".to_string();

        script.do_var(&key, &Yaml::from_str("42"));
        assert_eq!(42, *script.vars.get(&key).unwrap());
    }

    //-------------------------------------------------------------------------

    #[test]
    fn do_echo() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        script.vars.insert("a".into(), Value::Number(41.into()));

        script.do_echo(&Yaml::from_str("answer: ${a + 1}"));
        assert_eq!("answer: 42", script.writer.log[0]);
    }

    //-------------------------------------------------------------------------

    #[test]
    fn is_truthy() {
        let mut script = Script::new(String::new(), None);

        for e in vec![
            (&Yaml::from_str("true"), true),
            (&Yaml::from_str("false"), false),
            (&Yaml::from_str("1"), true),
            (&Yaml::from_str("0"), false),
            (&Yaml::from_str("foo"), true),
            (&Yaml::String("".into()), false),
        ] {
            assert_eq!(e.1, script.is_truthy(e.0), "{e:?}");
        }
    }

    #[test]
    fn do_if_then() {
        let mut script = Script::new(String::new(), None);
        let docs = YamlLoader::load_from_str("then: [a: 42]").unwrap();

        script.do_if(&Yaml::from_str("true"), &docs[0].as_hash().unwrap());
        assert_eq!(42, *script.vars.get("a").unwrap());
    }

    #[test]
    fn do_if_else() {
        let mut script = Script::new(String::new(), None);
        let docs = YamlLoader::load_from_str("else: [a: 42]").unwrap();

        script.do_if(&Yaml::from_str("false"), &docs[0].as_hash().unwrap());
        assert_eq!(42, *script.vars.get("a").unwrap());
    }

    //-------------------------------------------------------------------------

    #[test]
    fn do_while() {
        let mut script = Script::new(String::new(), None);
        let docs = YamlLoader::load_from_str("do: [a: 42]").unwrap();
        script.vars.insert("a".into(), Value::Number(1.into()));

        script.do_while(&Yaml::from_str("${a == 1}"), &docs[0].as_hash().unwrap());
        assert_eq!(42, *script.vars.get("a").unwrap());
    }

    #[test]
    fn do_while_break() {
        let mut script = Script::new(String::new(), None);
        let docs = YamlLoader::load_from_str("do: [break: true]").unwrap();

        script.do_while(&Yaml::from_str("true"), &docs[0].as_hash().unwrap());
        assert!(script.break_opt.is_none());
    }

    //-------------------------------------------------------------------------

    #[test]
    fn do_each() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let docs = YamlLoader::load_from_str("{in: [1, 2], do: [echo: '${x}']}").unwrap();

        script.do_each(&Yaml::from_str("x"), &docs[0].as_hash().unwrap());
        assert_eq!(2, *script.vars.get("x").unwrap());
        assert_eq!("1", script.writer.log[0]);
        assert_eq!("2", script.writer.log[1]);
    }

    #[test]
    fn do_each_break() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let docs = YamlLoader::load_from_str("{in: [1, 2], do: [break: true]}").unwrap();

        script.do_each(&Yaml::from_str("x"), &docs[0].as_hash().unwrap());
        assert_eq!(1, *script.vars.get("x").unwrap());
        assert_eq!(0, script.writer.log.len());
    }

    //-------------------------------------------------------------------------

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
        assert_eq!(42, script.vars.get("a").unwrap().as_i64().unwrap());
        assert_eq!("foo", script.writer.log[0]);
    }

    #[test]
    #[should_panic]
    fn run_docs_break() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let docs = YamlLoader::load_from_str("[break: true]").unwrap();

        script.run_docs(docs);
    }
}
