fn main() {
    let input = "123 world";
    println!("source: {:?}, parsed: {:?}", input, source(input));

    let input = "Hello world";
    println!("source: {:?}, parsed: {:?}", input, source(input));

    let input = "      world";
    println!("source: {:?}, parsed: {:?}", input, source(input));
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
}

fn token(input: &str) -> Option<(&str, Option<Token>)> {
    let whitespace_res = whitespace(input);
    let ident_res = ident(input);
    let number_res = number(input);
    [whitespace_res, ident_res, number_res]
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
    if matches!(input.chars().next(), Some(_x @ ('a'..='z' | 'A'..='Z'))) {
        while matches!(
            input.chars().next(),
            Some(_x @ ('a'..='z' | 'A'..='Z' | '0'..='9'))
        ) {
            let mut chars = input.chars();
            chars.next();
            input = chars.as_str();
        }
    }
    (input, Some(Token::Ident))
}

fn number(mut input: &str) -> (&str, Option<Token>) {
    if matches!(
        input.chars().next(),
        Some(_x @ ('-' | '+' | '.' | '0'..='9'))
    ) {
        while matches!(input.chars().next(), Some(_x @ ('.' | '0'..='9'))) {
            let mut chars = input.chars();
            chars.next();
            input = chars.as_str();
        }
    }
    (input, Some(Token::Number))
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
