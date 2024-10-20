use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::complete::{alphanumeric1, char, digit1, multispace0},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair},
    IResult,
};
use nom_locate::LocatedSpan;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Node {
    pub node_type: NodeType,
    pub location: Location,
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
        delimited(multispace0, char('='), multispace0),
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
    separated_list0(
        alt((char(';'), char('\n'))),
        preceded(multispace0, parse_expression),
    )(input)
}

pub fn parse(input: &str) -> Result<Vec<Node>, String> {
    match parse_expressions(Span::new(input)) {
        Ok((_, nodes)) => Ok(nodes),
        Err(e) => Err(format!("{:?}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(offset: usize, line: u32, fragment: &str) -> Span {
        unsafe { Span::new_from_raw_offset(offset, line, fragment, ()) }
    }

    #[test]
    fn test_integer() {
        assert_eq!(
            parse_integer(Span::new("123")),
            Ok((span(3, 1, ""), NodeType::Integer(123)))
        );
    }

    #[test]
    fn test_string() {
        assert_eq!(
            parse_string(Span::new("\"hello\"")),
            Ok((span(7, 1, ""), NodeType::String("hello".to_string())))
        );
    }

    #[test]
    fn test_assignment() {
        assert_eq!(
            parse_assignment(Span::new("x = 123")),
            Ok((
                span(7, 1, ""),
                NodeType::Assignment(
                    "x".to_string(),
                    Box::new(Node {
                        node_type: NodeType::Integer(123),
                        location: Location::new(1, 5)
                    })
                )
            ))
        );
        assert_eq!(
            parse_assignment(Span::new("x=y=123")),
            Ok((
                span(7, 1, ""),
                NodeType::Assignment(
                    "x".to_string(),
                    Box::new(Node {
                        node_type: NodeType::Assignment(
                            "y".to_string(),
                            Box::new(Node {
                                node_type: NodeType::Integer(123),
                                location: Location::new(1, 5)
                            })
                        ),
                        location: Location::new(1, 3)
                    })
                )
            ))
        );
    }

    #[test]
    fn test_variable() {
        assert_eq!(
            parse_variable(Span::new("x")),
            Ok((span(1, 1, ""), NodeType::Variable("x".to_string())))
        );
    }

    #[test]
    fn test_expression() {
        assert_eq!(
            parse_expression(Span::new("x = 123")),
            Ok((
                span(7, 1, ""),
                Node {
                    node_type: NodeType::Assignment(
                        "x".to_string(),
                        Box::new(Node {
                            node_type: NodeType::Integer(123),
                            location: Location::new(1, 5)
                        })
                    ),
                    location: Location::new(1, 1)
                }
            ))
        );
    }
}
