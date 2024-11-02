use std::collections::HashMap;
use std::str;

use ruby_prism::{Node, Visit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Signature(HashMap<String, Method>),
    Alias(String),
}

impl Type {
    pub fn alias(name: &str) -> Self {
        Self::Alias(name.to_string())
    }

    pub fn sig<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (String, Method)>,
    {
        Self::Signature(HashMap::from_iter(iter))
    }

    pub fn as_sig(&self) -> Option<&HashMap<String, Method>> {
        match self {
            Self::Signature(sig) => Some(sig),
            _ => None,
        }
    }

    pub fn as_alias(&self) -> Option<&String> {
        match self {
            Self::Alias(name) => Some(name),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Method {
    args: Vec<Type>,
    ret: Type,
}

impl Method {
    pub fn new(args: Vec<Type>, ret: Type) -> Self {
        Self { args, ret }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    UndefinedVariable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    kind: ErrorKind,
    loc: (usize, usize),
}

impl TypeError {
    pub fn new(kind: ErrorKind, node: Node) -> Self {
        let loc = node.location();
        Self {
            kind,
            loc: (loc.start_offset(), loc.end_offset()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeChecker {
    types: HashMap<String, Type>,
    objects: HashMap<String, Type>,
    type_stack: Vec<Type>,
    local_variables: Vec<HashMap<String, Type>>,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut objects = HashMap::new();
        objects.insert("Object".to_string(), Type::sig([]));
        let mut types = HashMap::new();
        types.insert(
            "String".to_string(),
            Type::sig([(
                "upcase".to_string(),
                Method {
                    args: vec![],
                    ret: Type::alias("String"),
                },
            )]),
        );
        Self {
            types,
            objects,
            type_stack: Vec::new(),
            local_variables: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn get_object(&self, name: &str) -> Option<&Type> {
        self.objects.get(name)
    }
}

pub fn to_string(c: ruby_prism::ConstantId) -> String {
    str::from_utf8(c.as_slice()).unwrap().to_string()
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl<'pr> Visit<'pr> for TypeChecker {
    fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
        if let Some(body) = node.body() {
            self.local_variables.push(HashMap::new());
            self.visit(&body);
            self.local_variables.pop();
        }

        for param in node.parameters().iter() {
            self.visit(&param.as_node());
        }

        if let Some(Type::Signature(sig)) = self.objects.get_mut("Object") {
            sig.insert(
                to_string(node.name()),
                Method {
                    args: vec![],
                    ret: self.type_stack.pop().unwrap(),
                },
            );
        }
    }

    fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode<'pr>) {
        for stmt in node.body().iter() {
            self.visit(&stmt);
        }
    }

    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        let name = to_string(node.name());
        if let Some(ty) = self.local_variables.last().unwrap().get(&name) {
            self.type_stack.push(ty.clone());
        } else {
            self.errors.push(TypeError::new(
                ErrorKind::UndefinedVariable(name),
                node.as_node(),
            ));
        }
    }

    fn visit_required_parameter_node(&mut self, node: &ruby_prism::RequiredParameterNode<'pr>) {
        println!(
            "required parameter: {:?}",
            str::from_utf8(node.name().as_slice()).unwrap().to_string()
        );
    }

    fn visit_string_node(&mut self, _: &ruby_prism::StringNode<'pr>) {
        self.type_stack.push(Type::alias("String"));
    }

    fn visit_integer_node(&mut self, _: &ruby_prism::IntegerNode<'pr>) {
        self.type_stack.push(Type::alias("Integer"));
    }

    fn visit_class_node(&mut self, node: &ruby_prism::ClassNode<'pr>) {
        println!("class name: {:?}", node.name());
    }

    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        self.visit(&node.value());
        self.local_variables
            .last_mut()
            .unwrap()
            .insert(to_string(node.name()), self.type_stack.pop().unwrap());
    }

    fn visit_symbol_node(&mut self, node: &ruby_prism::SymbolNode<'pr>) {
        println!("Find symbol: {:?}", str::from_utf8(node.unescaped()))
    }
}
