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
    pub break_opt: Option<String>,
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
            break_opt: None,
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
        // println!("{step:?}"); // ???: need better option for verbose
        let token = step.iter().next().unwrap();
        let key = token.0.as_str().unwrap();

        match key {
            "echo" => self.do_echo(token.1),
            "if" => self.do_if(token.1, step),
            "while" => self.do_while(token.1, step),
            "break" => self.do_break(token.1, step),
            // ...
            _ => self.do_var(&key.into(), token.1),
        }
    }

    //-------------------------------------------------------------------------

    // - <name>: <value>
    fn do_var(&mut self, name: &String, yaml: &Yaml) {
        let val = self.yaml_to_value(yaml);
        self.vars.insert(name.into(), val);
    }

    //-------------------------------------------------------------------------

    // - echo: <expression>
    fn do_echo(&mut self, yaml: &Yaml) {
        let val = self.eval(yaml);
        let val_str = self.value_to_string(val);
        self.write(val_str);
    }

    fn write(&mut self, val: String) {
        (self.writer)(&mut self.log, val);
    }

    //-------------------------------------------------------------------------

    // - if: <truthy-expression>
    //   [then:
    //      <steps>]
    //   [else:
    //      <steps>]
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

    // - while: <truthy-expression>
    //   do:
    //     <steps>
    fn do_while(&mut self, cond: &Yaml, step: &Hash) {
        let key = "do";
        let steps = step
            .get(&Yaml::from_str(key))
            .expect(format!("expected '{key}'").as_str());

        while self.is_truthy(cond) {
            self.run_steps(steps.as_vec().unwrap());

            if self.break_opt.is_some() {
                self.break_opt = None;
                break;
            }
        }
    }

    //-------------------------------------------------------------------------

    // - break: [<truthy-expression>]
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
        let re = Regex::new(r"\$\{.+\}").unwrap();

        match re.is_match(&expr) {
            true => self.eval_tokens(expr, re),
            false => Value::String(expr.into()),
        }
    }

    fn eval_tokens(&mut self, expr: String, re: Regex) -> Value {
        println!("eval_tokens: expr: {expr}");
        let mut buf = expr.clone();

        for token in re.find_iter(&expr) {
            buf.replace_range(token.start()..token.end(), self.eval_token(token).as_str());
        }

        self.yaml_to_value(&Yaml::from_str(&buf))
    }

    fn eval_token(&mut self, token: Match<'_>) -> String {
        println!("eval_token: token: {token:?}");
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
        script.vars.insert("b".into(), Value::Number(1.into()));

        assert_eq!(0, script.eval(&Yaml::from_str("0")));
        assert_eq!(1, script.eval(&Yaml::from_str("${a}")));
        assert_eq!(2, script.eval(&Yaml::from_str("${a+b}")));
        assert_eq!("a+b = 2", script.eval(&Yaml::from_str("a+b = ${a+b}")));
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
        script.vars.insert("a".into(), Value::Number(42.into()));

        script.do_echo(&Yaml::from_str("a: ${a + 1}"));
        assert_eq!("a: 43", script.log[0]);
    }

    //-------------------------------------------------------------------------

    #[test]
    fn is_truthy_true() {
        let mut script = Script::new(String::new(), None);

        for val in vec!["true", "1", "foo"] {
            assert_eq!(true, script.is_truthy(&Yaml::from_str(val)));
        }
    }

    #[test]
    fn is_truthy_false() {
        let mut script = Script::new(String::new(), None);

        for cond in vec![
            &Yaml::from_str("false"),
            &Yaml::from_str("0"),
            &Yaml::String("".into()),
        ] {
            assert_eq!(false, script.is_truthy(cond));
        }
    }

    #[test]
    fn do_if_true() {
        let mut script = Script::new(String::new(), None);
        let then_yaml = YamlLoader::load_from_str("then: [a: 42]").unwrap();

        script.do_if(&Yaml::from_str("true"), &then_yaml[0].as_hash().unwrap());
        assert_eq!(42, *script.vars.get("a").unwrap());
    }

    #[test]
    fn do_if_false() {
        let mut script = Script::new(String::new(), None);
        let else_yaml = YamlLoader::load_from_str("else: [a: 42]").unwrap();

        script.do_if(&Yaml::from_str("false"), &else_yaml[0].as_hash().unwrap());
        assert_eq!(42, *script.vars.get("a").unwrap());
    }

    //-------------------------------------------------------------------------

    #[test]
    fn do_while() {
        let mut script = Script::new(String::new(), None);
        let do_yaml = YamlLoader::load_from_str("do: [a: 42]").unwrap();
        script.vars.insert("a".into(), Value::Number(1.into()));

        script.do_while(&Yaml::from_str("$a == 1"), &do_yaml[0].as_hash().unwrap());
        assert_eq!(42, *script.vars.get("a").unwrap());
    }

    #[test]
    fn do_while_break() {
        let mut script = Script::new(String::new(), None);
        let do_yaml = YamlLoader::load_from_str("do: [break: true]").unwrap();

        script.do_while(&Yaml::from_str("true"), &do_yaml[0].as_hash().unwrap());
        assert!(script.break_opt.is_none());
    }

    //-------------------------------------------------------------------------

    #[test]
    fn do_break() {
        let mut script = Script::new(String::new(), None);
        let hash_yaml = YamlLoader::load_from_str("foo:").unwrap();

        script.do_break(&Yaml::from_str("true"), &hash_yaml[0].as_hash().unwrap());
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
        assert_eq!("foo", script.log[0]);
    }

    #[test]
    fn run_steps() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let docs = YamlLoader::load_from_str("[a: 42, echo: foo]").unwrap();
        let steps = docs[0].as_vec().unwrap();

        script.run_steps(&steps);
        assert_eq!(42, script.vars.get("a").unwrap().as_i64().unwrap());
        assert_eq!("foo", script.log[0]);
    }

    #[test]
    #[should_panic]
    fn run_docs_break() {
        let mut script = Script::new(String::new(), Some(Vec::new()));
        let docs = YamlLoader::load_from_str("[break: true]").unwrap();

        script.run_docs(docs);
    }
}
