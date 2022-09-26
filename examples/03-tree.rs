fn main() {
    let input = "Hello world";
    println!("source: {:?}, parsed: {:?}", input, source(input));

    let input = "(123  456 ) world";
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

fn source(mut input: &str) -> (&str, TokenTree) {
    let mut tokens = vec![];
    while !input.is_empty() {
        input = if let Some((next_input, token)) = token(input) {
            if let Some(token) = token {
                match token {
                    Token::LParen => {
                        let (next_input, tt) = source(next_input);
                        tokens.push(tt);
                        next_input
                    }
                    Token::RParen => return (next_input, TokenTree::Tree(tokens)),
                    _ => {
                        tokens.push(TokenTree::Token(token));
                        next_input
                    }
                }
            } else {
                next_input
            }
        } else {
            break;
        }
    }
    (input, TokenTree::Tree(tokens))
}

#[derive(Debug, PartialEq)]
enum Token<'src> {
    Ident(&'src str),
    Number(f64),
    LParen,
    RParen,
}

#[derive(Debug, PartialEq)]
enum TokenTree<'src> {
    Token(Token<'src>),
    Tree(Vec<TokenTree<'src>>),
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
    (
        input,
        Some(Token::Ident(&start[..(start.len() - input.len())])),
    )
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
        assert_eq!(ident("Adam"), ("", Some(Token::Ident("Adam"))));
    }

    #[test]
    fn test_number() {
        assert_eq!(number("123.45 "), (" ", Some(Token::Number(123.45))));
    }
}
