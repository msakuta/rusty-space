use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alpha1, alphanumeric1, anychar, char, multispace0, multispace1,
        newline, none_of, one_of, space0,
    },
    combinator::{not, opt, recognize},
    multi::{fold_many0, many0, many1},
    number::complete::recognize_float,
    sequence::{delimited, pair},
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Expression<'src> {
    Ident(&'src str),
    NumLiteral(f64),
    FnInvoke(&'src str, Vec<Expression<'src>>),
    Add(Box<Expression<'src>>, Box<Expression<'src>>),
    Sub(Box<Expression<'src>>, Box<Expression<'src>>),
    Mul(Box<Expression<'src>>, Box<Expression<'src>>),
    Div(Box<Expression<'src>>, Box<Expression<'src>>),
}

fn unary_fn(f: fn(f64) -> f64) -> impl Fn(&Vec<Expression>) -> f64 {
    move |args| {
        f(eval(
            args.into_iter().next().expect("function missing argument"),
        ))
    }
}

fn binary_fn(f: fn(f64, f64) -> f64) -> impl Fn(&Vec<Expression>) -> f64 {
    move |args| {
        let mut args = args.into_iter();
        let lhs =
            eval(args.next().expect("function missing the first argument"));
        let rhs =
            eval(args.next().expect("function missing the second argument"));
        f(lhs, rhs)
    }
}

pub(crate) fn eval(expr: &Expression) -> f64 {
    match expr {
        Expression::Ident("pi") => std::f64::consts::PI,
        Expression::Ident(id) => panic!("Unknown name {:?}", id),
        Expression::NumLiteral(n) => *n,
        Expression::FnInvoke("sqrt", args) => unary_fn(f64::sqrt)(args),
        Expression::FnInvoke("sin", args) => unary_fn(f64::sin)(args),
        Expression::FnInvoke("cos", args) => unary_fn(f64::cos)(args),
        Expression::FnInvoke("tan", args) => unary_fn(f64::tan)(args),
        Expression::FnInvoke("asin", args) => unary_fn(f64::asin)(args),
        Expression::FnInvoke("acos", args) => unary_fn(f64::acos)(args),
        Expression::FnInvoke("atan", args) => unary_fn(f64::atan)(args),
        Expression::FnInvoke("atan2", args) => binary_fn(f64::atan2)(args),
        Expression::FnInvoke("pow", args) => binary_fn(f64::powf)(args),
        Expression::FnInvoke("exp", args) => unary_fn(f64::exp)(args),
        Expression::FnInvoke("log", args) => binary_fn(f64::log)(args),
        Expression::FnInvoke("log10", args) => unary_fn(f64::log10)(args),
        Expression::FnInvoke(name, _) => panic!("Unknown function {name:?}"),
        Expression::Add(lhs, rhs) => eval(lhs) + eval(rhs),
        Expression::Sub(lhs, rhs) => eval(lhs) - eval(rhs),
        Expression::Mul(lhs, rhs) => eval(lhs) * eval(rhs),
        Expression::Div(lhs, rhs) => eval(lhs) / eval(rhs),
    }
}

fn factor(i: &str) -> IResult<&str, Expression> {
    alt((number, func_call, ident, parens))(i)
}

fn func_call(i: &str) -> IResult<&str, Expression> {
    let (r, ident) = delimited(multispace0, identifier, multispace0)(i)?;
    // println!("func_invoke ident: {}", ident);
    let (r, args) = delimited(
        multispace0,
        delimited(
            tag("("),
            many0(delimited(
                multispace0,
                expr,
                delimited(multispace0, opt(tag(",")), multispace0),
            )),
            tag(")"),
        ),
        multispace0,
    )(r)?;
    Ok((r, Expression::FnInvoke(ident, args)))
}

fn ident(input: &str) -> IResult<&str, Expression> {
    let (r, res) = delimited(multispace0, identifier, multispace0)(input)?;
    Ok((r, Expression::Ident(res)))
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn number(input: &str) -> IResult<&str, Expression> {
    let (r, v) = recognize_float(input)?;
    Ok((
        r,
        Expression::NumLiteral(v.parse().map_err(|_| {
            nom::Err::Error(nom::error::Error {
                input,
                code: nom::error::ErrorKind::Digit,
            })
        })?),
    ))
}

fn parens(i: &str) -> IResult<&str, Expression> {
    delimited(
        multispace0,
        delimited(tag("("), expr, tag(")")),
        multispace0,
    )(i)
}

fn term(i: &str) -> IResult<&str, Expression> {
    let (i, init) = factor(i)?;

    fold_many0(
        pair(
            delimited(multispace0, alt((char('*'), char('/'))), multispace0),
            factor,
        ),
        move || init.clone(),
        |acc, (op, val): (char, Expression)| match op {
            '*' => Expression::Mul(Box::new(acc), Box::new(val)),
            '/' => Expression::Div(Box::new(acc), Box::new(val)),
            _ => panic!(
                "Multiplicative expression should have '*' or '/' operator"
            ),
        },
    )(i)
}

pub fn expr(i: &str) -> IResult<&str, Expression> {
    let (i, init) = term(i)?;

    fold_many0(
        pair(
            delimited(multispace0, alt((char('+'), char('-'))), multispace0),
            term,
        ),
        move || init.clone(),
        |acc, (op, val): (char, Expression)| match op {
            '+' => Expression::Add(Box::new(acc), Box::new(val)),
            '-' => Expression::Sub(Box::new(acc), Box::new(val)),
            _ => panic!("Additive expression should have '+' or '-' operator"),
        },
    )(i)
}

pub fn block_str(i: &str) -> IResult<&str, String> {
    fn inner_str(i: &str) -> IResult<&str, String> {
        let (rest, c) = none_of("}")(i)?;
        Ok((rest, c.to_string()))
    }
    let (rest, ex) =
        delimited(char('{'), many0(alt((block_str, inner_str))), char('}'))(i)?;
    Ok((rest, ex.join("")))
}

pub fn block(i: &str) -> IResult<&str, Arg> {
    let (rest, ex) = delimited(char('{'), many0(command), char('}'))(i)?;
    Ok((rest, Arg::Block(ex)))
}

fn identifier_string(i: &str) -> IResult<&str, Arg> {
    let (rest, id) = identifier(i).map(|(r, s)| (r, s))?;
    Ok((rest, Arg::Str(id)))
}

fn arg(i: &str) -> IResult<&str, Arg> {
    delimited(space0, alt((block, identifier_string)), space0)(i)
}

#[derive(Debug, PartialEq)]
pub enum Arg<'src> {
    Str(&'src str),
    Block(Vec<Command<'src>>),
}

#[derive(Debug, PartialEq)]
pub enum Property<'src> {
    Str(String),
    Expr(Expression<'src>),
}

#[derive(Debug, PartialEq)]
pub enum Command<'src> {
    Com(Vec<Arg<'src>>),
    Prop(&'src str, Property<'src>),
}

fn newlines(i: &str) -> IResult<&str, ()> {
    delimited(space0, many1(one_of("\r\n")), space0)(i)
        .map(|(rest, _)| (rest, ()))
}

fn string(i: &str) -> IResult<&str, String> {
    let (i, s) = delimited(char('"'), many0(none_of("\"")), char('"'))(i)?;
    Ok((
        i,
        String::from_utf8(s.into_iter().map(|c| c as u8).collect()).unwrap(),
    ))
}

pub fn command(i: &str) -> IResult<&str, Command> {
    fn com(i: &str) -> IResult<&str, Command> {
        let (rest, res) = many1(arg)(i)?;
        Ok((rest, Command::Com(res)))
    }

    fn prop(i: &str) -> IResult<&str, Command> {
        fn string_prop(i: &str) -> IResult<&str, Property> {
            let (i, s) = string(i)?;
            Ok((i, Property::Str(s.to_string())))
        }
        fn expr_prop(i: &str) -> IResult<&str, Property> {
            let (i, ex) = expr(i)?;
            Ok((i, Property::Expr(ex)))
        }
        let (i, res) = identifier(i)?;
        let (i, _) = delimited(space0, char(':'), space0)(i)?;
        let (i, ex) = alt((string_prop, expr_prop))(i)?;
        Ok((i, Command::Prop(res, ex)))
    }

    delimited(opt(newlines), alt((prop, com)), opt(newlines))(i)
}

pub fn commands(i: &str) -> IResult<&str, Vec<Command>> {
    many0(command)(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_number() {
        assert_eq!(
            number("123.45 "),
            Ok((" ", Expression::NumLiteral(123.45)))
        );
    }

    #[test]
    fn test_block() {
        assert_eq!(
            block("{p}"),
            Ok(("", Arg::Block(vec![Command::Com(vec![Arg::Str("p")])])))
        );
    }

    #[test]
    fn test_multiline() {
        assert_eq!(
            block(
                "{
            p
}"
            ),
            Ok(("", Arg::Block(vec![Command::Com(vec![Arg::Str("p")])])))
        );
    }

    #[test]
    fn test_command() {
        assert_eq!(
            command(
                r#"a  {
            p
        }"#
            ),
            Ok((
                "",
                Command::Com(vec![
                    Arg::Str("a"),
                    Arg::Block(vec![Command::Com(vec![Arg::Str("p")])])
                ])
            ))
        );
    }

    #[test]
    fn test_prop() {
        assert_eq!(
            command("hello: world"),
            Ok((
                "",
                Command::Prop(
                    "hello",
                    Property::Expr(Expression::Ident("world"))
                )
            ))
        );
    }

    #[test]
    fn test_string_prop() {
        assert_eq!(
            command(r#"hello: "world""#),
            Ok((
                "",
                Command::Prop("hello", Property::Str("world".to_owned()))
            ))
        );
    }

    #[test]
    fn test_command_prop() {
        assert_eq!(
            command(
                r#"
astro Moon {
    radius: 0.5
}"#
            ),
            Ok((
                "",
                Command::Com(vec![
                    Arg::Str("astro"),
                    Arg::Str("Moon"),
                    Arg::Block(vec![Command::Prop(
                        "radius",
                        Property::Expr(Expression::NumLiteral(0.5))
                    )])
                ])
            ))
        );
    }
}
