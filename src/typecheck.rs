use std::collections::HashMap;

use crate::parser::{Node, NodeType};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Class {
    methods: HashMap<String, Type>,
}

impl Class {
    pub fn new(methods: HashMap<String, Type>) -> Self {
        Self { methods }
    }
}

pub type ClassName = String;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Variable(ClassName),
    Function(Vec<ClassName>, ClassName),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Env {
    instances: HashMap<String, Type>,
    classes: HashMap<String, Class>,
}

impl Env {
    pub fn new(instances: HashMap<String, Type>, classes: HashMap<String, Class>) -> Self {
        Self { instances, classes }
    }

    pub fn get_instance_type(&self, name: &str) -> Option<&Type> {
        self.instances.get(name)
    }

    pub fn get_class(&self, name: &str) -> Option<&Class> {
        self.classes.get(name)
    }
}

#[macro_export]
macro_rules! env {
    {
        instances {
            $($instance_name:ident => $instance_type:ident),* $(,)?
        },
        classes {
            $($class_name:ident {
                $($method_name:ident ($($arg:ident),*) => $ret_type:ident),* $(,)?
            }),* $(,)?
        }
    } => {{
        use std::collections::HashMap;
        use $crate::typecheck::{Class, Env, Type};

        let mut instances = HashMap::new();
        let mut classes = HashMap::new();

        $(
            instances.insert(
                stringify!($instance_name).to_string(),
                Type::Variable(stringify!($instance_type).to_string())
            );
        )*

        $(
            let mut methods = HashMap::new();

            $(
                methods.insert(
                    stringify!($method_name).to_string(),
                    Type::Function(
                        vec![$(stringify!($arg).to_string()),*],
                        stringify!($ret_type).to_string()
                    )
                );
            )*

            classes.insert(
                stringify!($class_name).to_string(),
                Class::new(methods)
            );
        )*

        Env::new(instances, classes)
    }};
}

fn default_class() -> HashMap<String, Class> {
    let mut classes = HashMap::new();
    classes.insert(
        "Integer".to_string(),
        Class {
            methods: {
                let mut methods = HashMap::new();
                methods.insert(
                    "to_s".to_string(),
                    Type::Function(vec![], "String".to_string()),
                );
                methods
            },
        },
    );
    classes
}

impl Default for Env {
    fn default() -> Self {
        Self {
            instances: HashMap::new(),
            classes: default_class(),
        }
    }
}

pub fn typecheck(node: &Node, env: &mut Env) -> Result<Type, String> {
    match &node.node_type {
        NodeType::Integer(_) => Ok(Type::Variable("Integer".to_string())),
        NodeType::String(_) => Ok(Type::Variable("String".to_string())),
        NodeType::Assignment(name, node) => {
            let ty = typecheck(node.as_ref(), env)?;
            env.instances.insert(name.clone(), ty.clone());
            Ok(ty)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typecheck() {
        let mut env = Env::default();
        let node = Node {
            node_type: NodeType::Assignment(
                "x".to_string(),
                Box::new(Node {
                    node_type: NodeType::Integer(42),
                    location: Default::default(),
                }),
            ),
            location: Default::default(),
        };
        assert_eq!(
            typecheck(&node, &mut env),
            Ok(Type::Variable("Integer".to_string()))
        );
        assert_eq!(
            env,
            env!(
                instances {
                    x => Integer,
                },
                classes {
                    Integer {
                        to_s() => String,
                    }
                }
            )
        );
    }
}
