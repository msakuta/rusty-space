fn main() {
    let input = "(123  456  world)";
    println!("source: {:?}, parsed: {:?}", input, source(input));

    let input = "((car cdr) cdr)";
    println!("source: {:?}, parsed: {:?}", input, source(input));

    let input = "()())))((()))";
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

fn source(mut input: &str) -> Vec<Token> {
    let mut tokens = vec![];
    while !input.is_empty() {
        input = if let Some((next_input, token)) = token(input) {
            if let Some(token) = token {
                tokens.push(token);
            }
            next_input
        } else {
            break;
        }
    }
    tokens
}

#[derive(Debug, PartialEq, Eq)]
enum Token {
    Ident,
    Number,
    LParen,
    RParen,
}

fn token(input: &str) -> Option<(&str, Option<Token>)> {
    let whitespace_res = whitespace(input);
    let ident_res = ident(input);
    let number_res = number(input);
    let lparen_res = lparen(input);
    let rparen_res = rparen(input);
    [
        whitespace_res,
        ident_res,
        number_res,
        lparen_res,
        rparen_res,
    ]
    .into_iter()
    .min_by(|x, y| x.0.len().cmp(&y.0.len()))
}

fn whitespace(mut input: &str) -> (&str, Option<Token>) {
    while matches!(input.chars().next(), Some(' ')) {
        let mut chars = input.chars();
        chars.next();
        input = chars.as_str();
    }
    (input, None)
}

fn ident(mut input: &str) -> (&str, Option<Token>) {
    if matches!(peek_char(input), Some(_x @ ('a'..='z' | 'A'..='Z'))) {
        input = advance_char(input);
        while matches!(
            peek_char(input),
            Some(_x @ ('a'..='z' | 'A'..='Z' | '0'..='9'))
        ) {
            input = advance_char(input);
        }
    }
    (input, Some(Token::Ident))
}

fn number(mut input: &str) -> (&str, Option<Token>) {
    if matches!(peek_char(input), Some(_x @ ('-' | '+' | '.' | '0'..='9'))) {
        input = advance_char(input);
        while matches!(peek_char(input), Some(_x @ ('.' | '0'..='9'))) {
            input = advance_char(input);
        }
    }
    (input, Some(Token::Number))
}

fn lparen(mut input: &str) -> (&str, Option<Token>) {
    if matches!(peek_char(input), Some(_x @ '(')) {
        input = advance_char(input);
    }
    (input, Some(Token::LParen))
}

fn rparen(mut input: &str) -> (&str, Option<Token>) {
    if matches!(peek_char(input), Some(_x @ ')')) {
        input = advance_char(input);
    }
    (input, Some(Token::RParen))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_whitespace() {
        assert_eq!(whitespace("    "), ("", None));
    }

    #[test]
    fn test_ident() {
        assert_eq!(ident("Adam"), ("", Some(Token::Ident)));
    }

    #[test]
    fn test_number() {
        assert_eq!(number("123.45 "), (" ", Some(Token::Number)));
    }
}
