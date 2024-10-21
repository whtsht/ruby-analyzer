use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::complete::{alphanumeric1, char, digit1, multispace0, space0},
    combinator::map,
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};
use nom_locate::LocatedSpan;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Node {
    pub node_type: NodeType,
    pub location: Location,
}

impl Node {
    pub fn new(node_type: NodeType, (line, column): (u32, usize)) -> Self {
        Self {
            node_type,
            location: Location::new(line, column),
        }
    }
}

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Location {
    pub line: u32,
    pub column: usize,
}

impl Location {
    pub fn new(line: u32, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeType {
    Integer(i64),
    String(String),
    Variable(String),
    Assignment(String, Box<Node>),
}

#[macro_export]
macro_rules! node {
    (Integer { value: $value:expr, location: ($line:expr, $column:expr) }) => {
        Node::new(NodeType::Integer($value), ($line, $column))
    };

    (String { value: $value:expr, location: ($line:expr, $column:expr) }) => {
        Node::new(NodeType::String($value.to_string()), ($line, $column))
    };

    (Variable { name: $name:expr, location: ($line:expr, $column:expr) }) => {
        Node::new(NodeType::Variable($name.to_string()), ($line, $column))
    };

    (Assignment { name: $name:expr, location: ($line:expr, $column:expr), value: $($value:tt)+ }) => {
        Node::new(
            NodeType::Assignment($name.to_string(), Box::new(node!($($value)+))),
            ($line, $column),
        )
    };
}

fn location(input: Span) -> IResult<Span, Location> {
    let (input, pos) = nom_locate::position(input)?;
    Ok((input, Location::new(pos.location_line(), pos.get_column())))
}

fn with_location<'a, F>(mut parser: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Node>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, NodeType>,
{
    move |input: Span<'a>| {
        let (input, loc) = location(input)?;
        let (input, node_type) = parser(input)?;
        Ok((
            input,
            Node {
                node_type,
                location: loc,
            },
        ))
    }
}

fn parse_comment(input: Span) -> IResult<Span, ()> {
    let (input, _) = preceded(char('#'), take_while(|c| c != '\n'))(input)?;
    Ok((input, ()))
}

fn parse_ignore(input: Span) -> IResult<Span, ()> {
    let (input, _) = alt((parse_comment, map(multispace0, |_| ())))(input)?;
    Ok((input, ()))
}

fn parse_integer(input: Span) -> IResult<Span, NodeType> {
    let (input, digits) = digit1(input)?;
    let value = digits.to_string().parse().unwrap();
    Ok((input, NodeType::Integer(value)))
}

fn parse_string(input: Span) -> IResult<Span, NodeType> {
    let (input, content) = delimited(char('"'), take_while(|c| c != '"'), char('"'))(input)?;
    Ok((input, NodeType::String(content.to_string())))
}

fn parse_variable(input: Span) -> IResult<Span, NodeType> {
    let (input, var_name) = alphanumeric1(input)?;
    Ok((input, NodeType::Variable(var_name.to_string())))
}

fn parse_assignment(input: Span) -> IResult<Span, NodeType> {
    let (input, (var_name, expr)) = separated_pair(
        alphanumeric1,
        delimited(parse_ignore, char('='), parse_ignore),
        parse_expression,
    )(input)?;
    Ok((
        input,
        NodeType::Assignment(var_name.to_string(), Box::new(expr)),
    ))
}

fn parse_expression(input: Span) -> IResult<Span, Node> {
    with_location(alt((
        parse_assignment,
        parse_string,
        parse_integer,
        parse_variable,
    )))(input)
}

fn parse_expressions(input: Span) -> IResult<Span, Vec<Node>> {
    let input = parse_ignore(input)?.0;
    separated_list0(
        alt((char(';'), char('\n'), map(parse_comment, |_| ' '))),
        preceded(parse_ignore, terminated(parse_expression, space0)),
    )(input)
}

pub fn parse(input: &str) -> Result<Vec<Node>, String> {
    match parse_expressions(Span::new(input)) {
        Ok((ignore, nodes)) => match parse_ignore(ignore) {
            Ok((_, ())) => Ok(nodes),
            _ => Err("parse error".to_string()),
        },
        Err(e) => Err(format!("{:?}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer() {
        assert_eq!(
            parse_integer(Span::new("123")).unwrap().1,
            NodeType::Integer(123)
        );
    }

    #[test]
    fn test_string() {
        assert_eq!(
            parse_string(Span::new("\"hello\"")).unwrap().1,
            NodeType::String("hello".to_string())
        );
    }

    #[test]
    fn test_assignment() {
        assert_eq!(
            parse_assignment(Span::new("x = 123")).unwrap().1,
            NodeType::Assignment(
                "x".to_string(),
                Box::new(Node {
                    node_type: NodeType::Integer(123),
                    location: Location::new(1, 5)
                })
            )
        );
        assert_eq!(
            parse_assignment(Span::new("x=y=123")).unwrap().1,
            NodeType::Assignment(
                "x".to_string(),
                Box::new(node!(Assignment {
                    name: "y",
                    location: (1, 3),
                    value: Integer {
                        value: 123,
                        location: (1, 5)
                    }
                }))
            )
        );
    }

    #[test]
    fn test_variable() {
        assert_eq!(
            parse_variable(Span::new("x")).unwrap().1,
            NodeType::Variable("x".to_string())
        );
    }

    #[test]
    fn test_expression() {
        assert_eq!(
            parse_expression(Span::new("x = 123")).unwrap().1,
            node!(Assignment {
                name: "x",
                location: (1, 1),
                value: Integer {
                    value: 123,
                    location: (1, 5)
                }
            })
        );
    }

    #[test]
    fn test_expressions() {
        assert_eq!(
            parse_expressions(Span::new("a = 1; b = 2")).unwrap().1,
            vec![
                node!(Assignment {
                    name: "a",
                    location: (1, 1),
                    value: Integer {
                        value: 1,
                        location: (1, 5)
                    }
                }),
                node!(Assignment {
                    name: "b",
                    location: (1, 8),
                    value: Integer {
                        value: 2,
                        location: (1, 12)
                    }
                })
            ]
        );
    }

    #[test]
    fn test_comment() {
        assert!(parse_comment(Span::new("# hello")).is_ok());
        assert_eq!(
            parse_expressions(Span::new("a = 1 # hello\nb = 2"))
                .unwrap()
                .1,
            vec![
                node!(Assignment {
                    name: "a",
                    location: (1, 1),
                    value: Integer {
                        value: 1,
                        location: (1, 5)
                    }
                }),
                node!(Assignment {
                    name: "b",
                    location: (2, 1),
                    value: Integer {
                        value: 2,
                        location: (2, 5)
                    }
                })
            ]
        );
    }
}
