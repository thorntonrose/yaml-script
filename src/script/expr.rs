use eval::{Expr, Value};
use regex::{Match, Regex};
use serde_json::Number;
use std::collections::HashMap;
use yaml_rust2::Yaml;

pub fn eval(yaml: &Yaml, vars: &HashMap<String, Value>) -> Value {
    let val = yaml_to_value(yaml);

    match val {
        Value::String(s) => eval_expr(s, vars),
        _ => val,
    }
}

pub fn eval_expr(expr: String, vars: &HashMap<String, Value>) -> Value {
    let re = Regex::new(r"\$\{[a-zA-Z0-9_\.+\-\*/%=<>!&| ]*\}").unwrap();

    match re.is_match(&expr) {
        true => eval_tokens(expr, re, vars),
        false => Value::String(expr.into()),
    }
}

pub fn eval_tokens(expr: String, re: Regex, vars: &HashMap<String, Value>) -> Value {
    let mut buf = expr.clone();

    while let Some(m) = re.find(&buf) {
        buf.replace_range(m.start()..m.end(), eval_token(m, vars).as_str());
    }

    yaml_to_value(&Yaml::from_str(&buf))
}

pub fn eval_token(token: Match<'_>, vars: &HashMap<String, Value>) -> String {
    let expr_str = token.as_str().replace("${", "").replace("}", "");
    let mut expr = Expr::new(expr_str);

    for (name, val) in vars {
        expr = expr.value(name, val);
    }

    value_to_string(expr.exec().unwrap())
}

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

#[cfg(test)]
mod tests {
    use crate::script::var::Var;

    use super::*;

    #[test]
    fn eval() {
        let mut var = Var::new();
        var.vars.insert("a".into(), Value::Number(1.into()));
        var.vars.insert("b".into(), Value::Number(2.into()));

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
            assert_eq!(e.1, super::eval(&Yaml::from_str(e.0), &var.vars), "{e:?}");
        }
    }
}
