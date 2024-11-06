use eval::{Expr, Value};
use regex::{Match, Regex};
use serde_json::Number;
use std::collections::HashMap;
use yaml_rust2::Yaml;

pub struct Binding {
    vars: HashMap<String, Value>,
}

impl Binding {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn get<S: Into<String>>(&self, name: S) -> Value {
        // ???: If not found, return empty string? Panic?
        self.vars.get(&name.into()).unwrap_or(&Value::Null).clone()
    }

    pub fn set<S: Into<String>>(&mut self, name: S, val: Value) {
        self.vars.insert(name.into(), val);
    }

    //-------------------------------------------------------------------------

    // pub fn var(&mut self, name: String, yaml: &Yaml) {
    pub fn var<S: Into<String>>(&mut self, name: S, yaml: &Yaml) {
        // ???: Need validation. Name must be identifier.
        self.set(name, Self::yaml_to_value(yaml));
    }

    //-------------------------------------------------------------------------

    pub fn eval(&mut self, yaml: &Yaml) -> Value {
        let val = Self::yaml_to_value(yaml);

        match val {
            Value::String(s) => self.eval_expr(s),
            _ => val,
        }
    }

    pub fn eval_expr(&mut self, expr: String) -> Value {
        let re = Regex::new(r"\$\{[a-zA-Z0-9_\.+\-\*/%=<>!&| ]*\}").unwrap();

        match re.is_match(&expr) {
            true => self.eval_tokens(expr, re),
            false => Value::String(expr.into()),
        }
    }

    pub fn eval_tokens(&mut self, expr: String, re: Regex) -> Value {
        let mut buf = expr.clone();

        while let Some(m) = re.find(&buf) {
            buf.replace_range(m.start()..m.end(), self.eval_token(m).as_str());
        }

        Self::yaml_to_value(&Yaml::from_str(&buf))
    }

    pub fn eval_token(&mut self, token: Match<'_>) -> String {
        let expr_str = token.as_str().replace("${", "").replace("}", "");
        let mut expr = Expr::new(expr_str);

        for (name, val) in &self.vars {
            expr = expr.value(name, val);
        }

        Self::value_to_string(expr.exec().unwrap())
    }

    //-------------------------------------------------------------------------

    pub fn yaml_to_value(yaml: &Yaml) -> Value {
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

    pub fn value_to_string(val: Value) -> String {
        match val {
            Value::String(s) => s.as_str().into(),
            _ => format!("{val}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn var() {
        let mut binding = Binding::new();

        binding.var("a", &Yaml::from_str("42"));
        assert_eq!(42, binding.get("a"));
    }

    #[test]
    fn eval() {
        let mut binding = Binding::new();
        binding.set("a", Value::Number(1.into()));
        binding.set("b", Value::Number(2.into()));

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
            assert_eq!(e.1, binding.eval(&Yaml::from_str(e.0)), "{e:?}");
        }
    }
}
