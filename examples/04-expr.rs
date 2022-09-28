fn main() {
    let input = "123";
    println!("source: {:?}, parsed: {:?}", input, source(input));

    let input = "Hello + world";
    println!("source: {:?}, parsed: {:?}", input, source(input));

    let input = "(123 + 456 ) + world";
    println!("source: {:?}, parsed: {:?}", input, source(input));

    let input = "car + cdr + cdr";
    println!("source: {:?}, parsed: {:?}", input, source(input));

    let input = "((1 + 2) + (3 + 4)) + 5 + 6";
    println!("source: {:?}, parsed: {:?}", input, source(input));
}

fn advance_char(input: &str) -> &str {
    let mut chars = input.chars();
    chars.next();
    chars.as_str()
}

fn peek_char(input: &str) -> Option<char> {
    input.chars().next()
}

fn source(input: &str) -> (&str, Option<Expression>) {
    if let (next_input, Some(expr)) = add(input) {
        return (next_input, Some(expr));
    }

    if let (next_input, Some(expr)) = term(input) {
        return (next_input, Some(expr));
    }

    (input, None)
}

fn paren(input: &str) -> (&str, Option<Expression>) {
    let next_input = if let (next_input, Some(_)) = lparen(whitespace(input)) {
        next_input
    } else {
        return (input, None);
    };

    let (next_input, expr) = if let (next_input, Some(expr)) = source(next_input) {
        (next_input, expr)
    } else {
        return (input, None);
    };

    let next_input = if let (next_input, Some(_)) = rparen(whitespace(next_input)) {
        next_input
    } else {
        return (input, None);
    };

    (next_input, Some(expr))
}

fn add_term(input: &str) -> (&str, Option<Expression>) {
    let (next_input, lhs) = if let (next_input, Some(lhs)) = term(input) {
        (next_input, lhs)
    } else {
        return (input, None);
    };

    let next_input = if let (next_input, Some(_)) = plus(whitespace(next_input)) {
        next_input
    } else {
        return (input, None);
    };

    (next_input, Some(lhs))
}

fn add(mut input: &str) -> (&str, Option<Expression>) {
    let mut left = None;
    while let (next_input, Some(expr)) = add_term(input) {
        if let Some(prev_left) = left {
            left = Some(Expression::Add(Box::new(prev_left), Box::new(expr)));
        } else {
            left = Some(expr);
        }
        input = next_input;
    }

    if left.is_none() {
        return (input, None);
    }

    let (next_input, rhs) = if let (next_input, Some(rhs)) = source(input) {
        (next_input, rhs)
    } else {
        return (input, None);
    };

    (
        next_input,
        Some(Expression::Add(Box::new(left.unwrap()), Box::new(rhs))),
    )
}

fn term(input: &str) -> (&str, Option<Expression>) {
    if let (next_input, Some(expr)) = paren(input) {
        return (next_input, Some(expr));
    }

    if let (next_input, Some(expr)) = token(input) {
        return (next_input, Some(Expression::Value(expr)));
    }

    (input, None)
}

#[derive(Debug, PartialEq)]
enum Token<'src> {
    Ident(&'src str),
    Number(f64),
    LParen,
    RParen,
    Plus,
}

#[derive(Debug, PartialEq)]
enum Expression<'src> {
    Value(Token<'src>),
    Add(Box<Expression<'src>>, Box<Expression<'src>>),
}

fn token(input: &str) -> (&str, Option<Token>) {
    if let (next_input, Some(res)) = ident(whitespace(input)) {
        return (next_input, Some(res));
    }
    if let (next_input, Some(res)) = number(whitespace(input)) {
        return (next_input, Some(res));
    }
    (input, None)
}

fn whitespace(mut input: &str) -> &str {
    while matches!(peek_char(input), Some(' ')) {
        input = advance_char(input);
    }
    input
}

fn ident(mut input: &str) -> (&str, Option<Token>) {
    let start = input;
    if matches!(peek_char(input), Some(_x @ ('a'..='z' | 'A'..='Z'))) {
        input = advance_char(input);
        while matches!(
            peek_char(input),
            Some(_x @ ('a'..='z' | 'A'..='Z' | '0'..='9'))
        ) {
            input = advance_char(input);
        }
    }
    if start.len() == input.len() {
        (input, None)
    } else {
        (
            input,
            Some(Token::Ident(&start[..(start.len() - input.len())])),
        )
    }
}

fn number(mut input: &str) -> (&str, Option<Token>) {
    let start = input;
    if matches!(peek_char(input), Some(_x @ ('-' | '+' | '.' | '0'..='9'))) {
        input = advance_char(input);
        while matches!(peek_char(input), Some(_x @ ('.' | '0'..='9'))) {
            input = advance_char(input);
        }
    }
    if let Ok(num) = start[..(start.len() - input.len())].parse::<f64>() {
        (input, Some(Token::Number(num)))
    } else {
        (input, None)
    }
}

fn lparen(input: &str) -> (&str, Option<Token>) {
    if matches!(peek_char(input), Some(_x @ '(')) {
        (advance_char(input), Some(Token::LParen))
    } else {
        (input, None)
    }
}

fn rparen(input: &str) -> (&str, Option<Token>) {
    if matches!(peek_char(input), Some(_x @ ')')) {
        (advance_char(input), Some(Token::RParen))
    } else {
        (input, None)
    }
}

fn plus(input: &str) -> (&str, Option<Token>) {
    if matches!(peek_char(input), Some(_x @ '+')) {
        (advance_char(input), Some(Token::Plus))
    } else {
        (input, None)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_whitespace() {
        assert_eq!(whitespace("    "), "");
    }

    #[test]
    fn test_ident() {
        assert_eq!(ident("Adam"), ("", Some(Token::Ident("Adam"))));
    }

    #[test]
    fn test_number() {
        assert_eq!(number("123.45 "), (" ", Some(Token::Number(123.45))));
    }
}
