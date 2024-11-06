use super::expr;
use eval::Value;
use std::collections::HashMap;
use yaml_rust2::Yaml;

pub struct Var {
    pub vars: HashMap<String, Value>,
}

impl Var {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn run(&mut self, name: &String, yaml: &Yaml) {
        // ???: Need validation. Name must be identifier.
        let val = expr::yaml_to_value(yaml);
        self.vars.insert(name.into(), val);
    }
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run() {
        let key = "a".to_string();
        let mut var = Var::new();

        var.run(&key, &Yaml::from_str("42"));
        assert_eq!(42, *var.vars.get(&key).unwrap());
    }
}
