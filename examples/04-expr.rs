fn main() {
    let input = "123";
    println!("source: {:?}, parsed: {:?}", input, expr(input));

    let input = "Hello + world";
    println!("source: {:?}, parsed: {:?}", input, expr(input));

    let input = "(123 + 456 ) + world";
    println!("source: {:?}, parsed: {:?}", input, expr(input));

    let input = "car + cdr + cdr";
    println!("source: {:?}, parsed: {:?}", input, expr(input));

    let input = "((1 + 2) + (3 + 4)) + 5 + 6";
    println!("source: {:?}, parsed: {:?}", input, expr(input));
}

fn advance_char(input: &str) -> &str {
    let mut chars = input.chars();
    chars.next();
    chars.as_str()
}

fn peek_char(input: &str) -> Option<char> {
    input.chars().next()
}

fn expr(input: &str) -> Option<(&str, Expression)> {
    if let Some(res) = add(input) {
        return Some(res);
    }

    if let Some(res) = term(input) {
        return Some(res);
    }

    None
}

fn paren(input: &str) -> Option<(&str, Expression)> {
    let next_input = lparen(whitespace(input))?;

    let (next_input, expr) = expr(next_input)?;

    let next_input = rparen(whitespace(next_input))?;

    Some((next_input, expr))
}

fn add_term(input: &str) -> Option<(&str, Expression)> {
    let (next_input, lhs) = term(input)?;

    let (next_input, _) = plus(whitespace(next_input))?;

    Some((next_input, lhs))
}

fn add(mut input: &str) -> Option<(&str, Expression)> {
    let mut left = None;
    while let Some((next_input, expr)) = add_term(input) {
        if let Some(prev_left) = left {
            left = Some(Expression::Add(Box::new(prev_left), Box::new(expr)));
        } else {
            left = Some(expr);
        }
        input = next_input;
    }

    let left = left?;

    let (next_input, rhs) = expr(input)?;

    Some((next_input, Expression::Add(Box::new(left), Box::new(rhs))))
}

fn term(input: &str) -> Option<(&str, Expression)> {
    if let Some(res) = paren(input) {
        return Some(res);
    }

    if let Some((next_input, expr)) = token(input) {
        return Some((next_input, Expression::Value(expr)));
    }

    None
}

#[derive(Debug, PartialEq)]
enum Token<'src> {
    Ident(&'src str),
    Number(f64),
    Plus,
}

#[derive(Debug, PartialEq)]
enum Expression<'src> {
    Value(Token<'src>),
    Add(Box<Expression<'src>>, Box<Expression<'src>>),
}

fn token(input: &str) -> Option<(&str, Token)> {
    if let (next_input, Some(res)) = ident(whitespace(input)) {
        return Some((next_input, res));
    }
    if let (next_input, Some(res)) = number(whitespace(input)) {
        return Some((next_input, res));
    }
    None
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

fn lparen(input: &str) -> Option<&str> {
    if matches!(peek_char(input), Some('(')) {
        Some(advance_char(input))
    } else {
        None
    }
}

fn rparen(input: &str) -> Option<&str> {
    if matches!(peek_char(input), Some(')')) {
        Some(advance_char(input))
    } else {
        None
    }
}

fn plus(input: &str) -> Option<(&str, Token)> {
    if matches!(peek_char(input), Some('+')) {
        Some((advance_char(input), Token::Plus))
    } else {
        None
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
