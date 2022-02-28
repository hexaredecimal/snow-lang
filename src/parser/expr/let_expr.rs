use super::mini_parse::{either, pair, right, zero_or_more, Parser};
use super::{app, constant, local, next_token, parse_name, Expr, KeyWord, Span, Spanned, Token};

fn let_block_expr<'a>() -> impl Parser<'a, Token, Spanned<Expr>> {
    either(local(), either(app(), constant()))
}

fn let_token<'a>() -> impl Parser<'a, Token, Spanned<Token>> {
    next_token(Token::KeyWord(KeyWord::Let))
}

fn indent_token<'a>() -> impl Parser<'a, Token, Spanned<Token>> {
    next_token(Token::InDent(4))
}

fn ident_let_token<'a>() -> impl Parser<'a, Token, Spanned<Token>> {
    right(next_token(Token::InDent(4)), let_token())
}

fn parse_let<'a>() -> impl Parser<'a, Token, Span> {
    move |input: &'a [Spanned<Token>]| {
        let (i, t) = either(let_token(), ident_let_token()).parse(input)?;
        Ok((i, t.span()))
    }
}

fn parse_body<'a>() -> impl Parser<'a, Token, Spanned<Expr>> {
    either(app(), either(local(), constant()))
}

fn binding<'a>() -> impl Parser<'a, Token, (Spanned<String>, Spanned<Expr>)> {
    // Id Op("=") (app, constant)
    pair(
        right(zero_or_more(indent_token()), parse_name()),
        right(next_token(Token::Op("=")), let_block_expr()),
    )
}
fn comma<'a>() -> impl Parser<'a, Token, Spanned<Token>> {
    either(
        next_token(Token::Ctrl(',')),
        right(zero_or_more(indent_token()), next_token(Token::Ctrl(','))),
    )
}

// TODO: Rename this to let_expr_func
pub(crate) fn let_expr_app<'a>() -> impl Parser<'a, Token, Spanned<Expr>> {
    move |input: &'a [Spanned<Token>]| {
        // Let
        let (i, _start) = parse_let().parse(input)?;
        // bindings
        let (i, (first, mut args)) =
            pair(binding(), zero_or_more(right(comma(), binding()))).parse(i)?;
        args.insert(0, first);
        // In
        let (i, _) = either(
            next_token(Token::KeyWord(KeyWord::In)),
            right(indent_token(), next_token(Token::KeyWord(KeyWord::In))),
        )
        .parse(i)?;

        // Let(String, Box<Spanned<Self>>, Box<Spanned<Self>>),
        // Return/Body?
        let (i, body) = right(zero_or_more(indent_token()), parse_body()).parse(i)?;
        let r#let = args.iter().rev().fold(body, |acc, (name, expr)| {
            (
                Expr::Let(
                    name.node.clone(),
                    Box::new(expr.clone()),
                    Box::new(acc.clone()),
                ),
                (name.span(), expr.span()).into(),
            )
                .into()
        });
        Ok((i, r#let))
    }
}

pub(crate) fn let_expr_do<'a>() -> impl Parser<'a, Token, Spanned<Expr>> {
    move |input: &'a [Spanned<Token>]| {
        let (i, _start) = parse_let().parse(input)?;
        // bindings
        let (i, (first, mut args)) =
            pair(binding(), zero_or_more(right(comma(), binding()))).parse(i)?;
        args.insert(0, first);

        // Let(String, Box<Spanned<Self>>, Box<Spanned<Self>>),
        // Return/Body?
        let (i, body) = right(zero_or_more(indent_token()), parse_body()).parse(i)?;
        let r#let = args.iter().rev().fold(body, |acc, (name, expr)| {
            (
                Expr::Let(
                    name.node.clone(),
                    Box::new(expr.clone()),
                    Box::new(acc.clone()),
                ),
                (name.span(), expr.span()).into(),
            )
                .into()
        });
        Ok((i, r#let))
    }
}
