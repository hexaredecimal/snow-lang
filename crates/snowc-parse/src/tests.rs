use super::{precedence::Precedence, ParserBuilder, Span, Token};
use snowc_error_messages::report;

use pretty_assertions::assert_eq;

fn parse_or_report(test_name: &str, src: &str) -> Vec<String> {
    let result = ParserBuilder::default()
        .debug_parser(true)
        .build(src)
        .parse();
    let Ok(ast) = result else {
        eprintln!("{:?}", result);
        if let Err(errors) = result {
            report(test_name, src, &errors);
        }
        return vec![];
    };
    ast.iter().map(ToString::to_string).collect()
}

#[test]
fn expression() {
    let src = "1";
    let left = ParserBuilder::default()
        .build(src)
        .expression(Precedence::None)
        .to_string();
    assert_eq!(left, src);

    let src = "1.2";
    let left = ParserBuilder::default()
        .build(src)
        .expression(Precedence::None)
        .to_string();
    assert_eq!(left, src);

    let src = "a";
    let left = ParserBuilder::default()
        .build(src)
        .expression(Precedence::None)
        .to_string();
    assert_eq!(left, src);
}
#[test]
fn unary() {
    let src = "-1";
    let left = ParserBuilder::default()
        .build(src)
        .expression(Precedence::None)
        .to_string();
    assert_eq!(left, "(- 1)");

    let src = "(- 1.2)";
    let left = ParserBuilder::default()
        .build(src)
        .expression(Precedence::None)
        .to_string();
    assert_eq!(left, "(- 1.2)");

    let src = "-a";
    let left = ParserBuilder::default()
        .build(src)
        .expression(Precedence::None)
        .to_string();
    assert_eq!(left, "(- a)");
}

#[test]
fn binary() {
    let src = "1 + 2 * 3";
    let left = ParserBuilder::default()
        .build(src)
        .expression(Precedence::None)
        .to_string();
    assert_eq!(left, "(+ 1 (* 2 3))");
}

#[test]
fn binary_ids() {
    let src = "a + b * c * d + e";
    let left = ParserBuilder::default()
        .build(src)
        .expression(Precedence::None)
        .to_string();
    assert_eq!(left, "(+ (+ a (* (* b c) d)) e)");

    let src = "a + b";
    let left = ParserBuilder::default()
        .build(src)
        .expression(Precedence::None)
        .to_string();
    assert_eq!(left, "(+ a b)");
}

#[test]
fn changing_precedence() {
    let src = "(-1 + 2) * 3 - -4";
    let left = ParserBuilder::default()
        .build(src)
        .expression(Precedence::None)
        .to_string();
    assert_eq!(left, "(- (* (+ (- 1) 2) 3) (- 4))");

    let src = "(((a)))";
    let left = ParserBuilder::default()
        .build(src)
        .expression(Precedence::None)
        .to_string();
    assert_eq!(left, "a");
}

#[test]
fn calling_operator() {
    let src = "(+) 1 2";
    let left = ParserBuilder::default()
        .build(src)
        .call(Precedence::None)
        .to_string();
    assert_eq!(left, "<(+): (1, 2)>");
}

#[test]
fn call() {
    let src = "add 1 2";
    let left = ParserBuilder::default()
        .build(src)
        .call(Precedence::None)
        .to_string();
    assert_eq!(left, "<add: (1, 2)>");
}

#[test]
fn pipe_call() {
    let src = "2 |> add 1";
    let left = ParserBuilder::default()
        .build(src)
        .call(Precedence::None)
        .to_string();
    assert_eq!(left, "(|> 2 <add: (1)>)");
}

#[test]
fn conditional() {
    let src = "if x > y then x else y;";
    let left = ParserBuilder::default()
        .build(src)
        .conditional()
        .to_string();
    assert_eq!(left, "(if ((> x y)) then x else y)");
}

#[test]
fn function_def_from_parse_funtion() {
    let src = "x y = x + y;";
    let left = ParserBuilder::default()
        .build(src)
        .function(&Token::Id("add".into(), Span::default()))
        .to_string();
    assert_eq!(left, r#"<add: (\x -> (\y -> (+ x y)))>"#);
}

#[test]
fn function_def() {
    let src = "add x y = x + y;";
    let left = parse_or_report("function_def", src);
    let right = vec![r#"<add: (\x -> (\y -> (+ x y)))>"#];
    assert_eq!(left, right);
}

#[test]
fn super_duper_function_def() {
    let src = "main = print (max ((add 1 2) + (sub 1 2)) 20);";
    let right = vec!["<main: <print: (<max: ((+ <add: (1, 2)> <sub: (1, 2)>), 20)>)>>"];
    let left = parse_or_report("super_duper_function_def", src);
    assert_eq!(left, right);
}

#[test]
fn multi_function_def() {
    let src = "add x y = x + y; sub x y = x - y;";
    let right = vec![
        r#"<add: (\x -> (\y -> (+ x y)))>"#,
        r#"<sub: (\x -> (\y -> (- x y)))>"#,
    ];
    let left = parse_or_report("muli_function_def", src);
    assert_eq!(left, right);
}

#[test]
fn closures() {
    let src = "add = (λx -> (λy -> x + y));";
    let right = vec![r#"<add: (\x -> (\y -> (+ x y)))>"#];
    let left = parse_or_report("closures 1", src);
    assert_eq!(left, right);

    let src = r#"add = (\x -> (\y -> x + y));"#;
    let right = vec![r#"<add: (\x -> (\y -> (+ x y)))>"#];
    let left = parse_or_report("closures 2", src);
    assert_eq!(left, right);
}

#[test]
fn enum_def() {
    let src = r#"enum Option = Some Int | None;"#;
    let right = vec![r#"<Option: (Some, [Int]), (None, [])>"#];
    let left = parse_or_report("user_type_def", src);
    assert_eq!(left, right);
}

#[test]
fn type_dec() {
    let src = r#"add :: Int -> Int -> Int;"#;
    let right = vec![r#"<add :: Int -> Int -> Int>"#];
    let left = parse_or_report("type_dec", src);
    assert_eq!(left, right);
}
