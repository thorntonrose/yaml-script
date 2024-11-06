use eval::Value;
use serde_json::Number;
use yaml_rust2::Yaml;

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
