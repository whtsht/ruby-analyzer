use ruby_analyzer::{
    parser::parse,
    typecheck::{typecheck, Env, Type},
};

#[test]
fn test_expressions() {
    let input = r#"
        a = 1
        b = "hello"
        c = d = e = f = g = h = 20
        i = b
    "#;
    let nodes = parse(input).unwrap();
    let mut env = Env::default();
    nodes.iter().for_each(|node| {
        typecheck(node, &mut env).unwrap();
    });

    assert_eq!(
        env.get_instance_type("a"),
        Some(&Type::Variable("Integer".to_string()))
    );
    assert_eq!(
        env.get_instance_type("b"),
        Some(&Type::Variable("String".to_string()))
    );
    ["c", "d", "e", "f", "g", "h"].iter().for_each(|name| {
        assert_eq!(
            env.get_instance_type(name),
            Some(&Type::Variable("Integer".to_string()))
        );
    });
    assert_eq!(
        env.get_instance_type("i"),
        Some(&Type::Variable("String".to_string()))
    );
}
