use eval::{Expr, Value};
use regex::{Match, Regex};
use serde_json::Number;
use std::collections::HashMap;
use yaml_rust2::{
    yaml::{Array, Hash},
    Yaml, YamlLoader,
};

pub struct Binding {
    pub vars: HashMap<String, Yaml>,
    pub procs: HashMap<String, Yaml>,
    pub params: HashMap<String, Yaml>,
}

impl Binding {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            procs: HashMap::new(),
            params: HashMap::new(),
        }
    }

    pub fn entry_to_list(hash: &Hash, key: &str) -> Array {
        hash.get(&Yaml::from_str(key))
            .expect(&format!("expected '${key}'"))
            .clone()
            .into_vec()
            .expect("expected list")
    }

    #[allow(dead_code)]
    pub fn hash_from_str(text: &str) -> Hash {
        YamlLoader::load_from_str(text).unwrap()[0].as_hash().unwrap().clone()
    }

    //-------------------------------------------------------------------------

    #[allow(dead_code)]
    pub fn var<S: Into<String>>(&self, name: S) -> Yaml {
        let key = &name.into();

        self.params
            .get(key)
            .or_else(|| -> Option<&Yaml> { self.vars.get(key) })
            .unwrap_or(&Yaml::Null)
            .clone()
    }

    pub fn set_var<S: Into<String>>(&mut self, name: S, val: Yaml) {
        let key = name.into();

        match self.params.get(&key) {
            Some(_) => self.set_param(key, val),
            None => _ = self.vars.insert(key, val),
        }
    }

    #[allow(dead_code)]
    pub fn param<S: Into<String>>(&mut self, name: S) -> Yaml {
        self.params.get(&name.into()).unwrap_or(&Yaml::Null).clone()
    }

    pub fn set_param<S: Into<String>>(&mut self, name: S, val: Yaml) {
        self.params.insert(name.into(), val);
    }

    pub fn set_params(&mut self, hash: Hash) -> HashMap<String, Yaml> {
        let old = self.params.clone();

        for (param, yaml) in hash {
            self.set_param(param.as_str().unwrap(), self.eval_to_yaml(&yaml));
        }

        old
    }

    pub fn proc<S: Into<String>>(&self, name: S) -> Yaml {
        self.procs.get(&name.into()).unwrap().clone()
    }

    pub fn set_proc<S: Into<String>>(&mut self, name: S, val: Yaml) {
        self.procs.insert(name.into(), val);
    }

    //-------------------------------------------------------------------------

    pub fn is_truthy(&self, cond: &Yaml) -> bool {
        match self.eval_to_yaml(cond) {
            Yaml::Boolean(b) => b,
            Yaml::Real(s) => s.parse::<f64>().unwrap() != 0.0f64,
            Yaml::Integer(n) => n != 0i64,
            Yaml::String(s) => !s.is_empty(),
            // ???: more?
            _ => false,
        }
    }

    //-------------------------------------------------------------------------

    pub fn eval_to_string(&self, yaml: &Yaml) -> String {
        self.value_to_string(self.eval(yaml))
    }

    pub fn eval_to_i32(&self, yaml: &Yaml) -> i32 {
        self.value_to_i32(self.eval(yaml))
    }

    pub fn eval_to_yaml(&self, yaml: &Yaml) -> Yaml {
        self.value_to_yaml(self.eval(yaml))
    }

    pub fn eval(&self, yaml: &Yaml) -> Value {
        match self.yaml_to_value(yaml) {
            Value::String(s) => self.eval_expr(s),
            v => v,
        }
    }

    fn eval_expr(&self, expr: String) -> Value {
        let re = Regex::new(r"\$\{[a-zA-Z0-9_\.+\-\*/%=<>!&| ]*\}").unwrap();

        match re.is_match(&expr) {
            true => self.eval_tokens(expr, re),
            false => Value::String(expr.into()),
        }
    }

    fn eval_tokens(&self, expr: String, re: Regex) -> Value {
        let mut buf = expr.clone();

        while let Some(m) = re.find(&buf) {
            buf.replace_range(m.start()..m.end(), &self.eval_token(m));
        }

        self.yaml_to_value(&Yaml::from_str(&buf))
    }

    fn eval_token(&self, token: Match<'_>) -> String {
        let mut expr = Expr::new(token.as_str().replace("${", "").replace("}", ""));
        expr = self.add_values(expr, &self.vars);
        expr = self.add_values(expr, &self.params);

        self.value_to_string(expr.exec().unwrap())
    }

    fn add_values(&self, mut expr: Expr, vars: &HashMap<String, Yaml>) -> Expr {
        for (name, yaml) in vars {
            expr = expr.value(name, self.yaml_to_value(yaml));
        }

        expr
    }

    //-------------------------------------------------------------------------

    pub fn value_to_string(&self, val: Value) -> String {
        match val {
            Value::String(s) => s,
            _ => format!("{val}"),
        }
    }

    pub fn value_to_i32(&self, val: Value) -> i32 {
        val.as_i64().expect("expected number").try_into().unwrap_or(0)
    }

    pub fn yaml_to_value(&self, yaml: &Yaml) -> Value {
        match yaml {
            Yaml::Boolean(b) => Value::Bool(*b),
            Yaml::Integer(i) => Value::Number((*i).into()),
            Yaml::Null => Value::Null,
            Yaml::Real(_) => Value::Number(Number::from_f64(yaml.as_f64().unwrap()).unwrap()),
            Yaml::String(s) => Value::String(s.into()),
            // ...
            _ => Value::String(format!("{yaml:?}")),
        }
    }

    pub fn value_to_yaml(&self, val: Value) -> Yaml {
        match val {
            Value::Array(_) => todo!(),
            Value::Bool(b) => Yaml::Boolean(b),
            Value::Null => Yaml::Null,
            Value::Number(n) if n.is_f64() => Yaml::Real(n.to_string()),
            Value::Number(n) => Yaml::Integer(n.as_i64().unwrap()),
            Value::Object(_) => todo!(),
            Value::String(s) => Yaml::String(s),
        }
    }
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use yaml_rust2::yaml::Yaml;

    #[test]
    fn set_var() {
        let mut binding = Binding::new();

        binding.set_var("a", Yaml::Integer(1));
        assert_eq!(1, binding.vars.get("a").unwrap().as_i64().unwrap());
    }

    #[test]
    fn set_var_param() {
        let mut binding = Binding::new();
        binding.set_param("a", Yaml::Integer(1));

        binding.set_var("a", Yaml::Integer(1));
        assert_eq!(1, binding.param("a").as_i64().unwrap());
        assert_eq!(None, binding.vars.get("a"));
    }

    #[test]
    fn eval() {
        let mut binding = Binding::new();
        binding.set_var("a", Yaml::Integer(1));
        binding.set_var("b", Yaml::Integer(2));

        for e in vec![
            ("0", Value::from(0)),
            ("1.0", Value::from(1.0f64)),
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

    #[test]
    fn is_truthy() {
        let binding = Binding::new();

        for e in vec![
            (Yaml::from_str("true"), true),
            (Yaml::from_str("false"), false),
            (Yaml::from_str("1"), true),
            (Yaml::from_str("0"), false),
            (Yaml::from_str("foo"), true),
            (Yaml::String("".into()), false),
        ] {
            assert_eq!(e.1, binding.is_truthy(&e.0), "{e:?}");
        }
    }
}
