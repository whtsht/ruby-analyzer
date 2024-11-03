use std::collections::HashMap;

use ruby_analyzer::typecheck::{Method, Type, TypeChecker, TypeError};
use ruby_prism::{Node, ParseResult, Visit};
use yaml_rust::YamlLoader;

pub struct Scenario<'pr> {
    pub parse_result: ParseResult<'pr>,
    pub ruby_code: Node<'pr>,
    pub update_type: HashMap<String, Type>,
    pub errors: Vec<TypeError>,
}

fn run_scenario(ruby_node: Node, methods: HashMap<String, Method>, _errors: Vec<TypeError>) {
    let mut checker = TypeChecker::new();
    checker.visit(&ruby_node);
    let object = checker.get_object("#main").unwrap().as_sig().unwrap();
    for (name, ty) in methods {
        assert_eq!(object.get(&name), Some(&ty));
    }
}

#[test]
fn test_scenario() {
    for entry in glob::glob("tests/scenario/**/*.yml").unwrap() {
        let path = entry.unwrap();
        let scenario = std::fs::read_to_string(path).unwrap();
        let scenario = YamlLoader::load_from_str(&scenario).unwrap();
        let parse_result =
            ruby_prism::parse(scenario[0]["ruby"]["code"].as_str().unwrap().as_bytes());
        let ruby_code = parse_result.node();
        let methods =
            HashMap::from_iter(scenario[0]["type"].as_hash().unwrap().iter().map(|(k, v)| {
                let params = v["params"]
                    .as_vec()
                    .unwrap()
                    .iter()
                    .map(|x| Type::alias(x.as_str().unwrap()))
                    .collect();
                (
                    k.as_str().unwrap().to_string(),
                    Method::new(params, Type::alias(v["return"].as_str().unwrap())),
                )
            }));
        run_scenario(ruby_code, methods, vec![]);
    }
}
