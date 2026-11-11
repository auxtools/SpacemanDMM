extern crate dreammaker as dm;

use dm::FileId;
use dm::Token;
use dm::lexer::Token::*;
use dm::lexer::*;

fn lex(f: &str) -> Vec<Token> {
    let context = Default::default();
    let result = Lexer::new(&context, FileId::UNKNOWN, f.as_bytes())
        .map(|t| t.token)
        .collect();
    context.assert_success();
    result
}

fn one_token(f: &str) -> Token {
    let mut v = lex(f);
    assert_eq!(v.len(), 2, "not one token: {f:?} -> {v:?}");
    assert_eq!(v[1], Token!['\n']);
    v.remove(0)
}

fn float(f: &str) -> f32 {
    match one_token(f) {
        Token::Float(f) => f,
        other => panic!("{f:?}: expected float, got {other:?}"),
    }
}

#[test]
fn number_literals() {
    assert_eq!(lex("0.08"), vec![Float(0.08), Token!['\n']]);
    assert_eq!(lex("0xABCDE"), vec![Int(703710), Token!['\n']]);
    assert_eq!(lex("1e4"), vec![Float(10000.0), Token!['\n']]);

    // Hexadecimal values above i32::MAX become floats.
    assert_eq!(lex("0x7FFFFFFF"), vec![Int(2147483647), Token!['\n']]);
    assert_eq!(float("0x80000000"), 2147483648.0);
    assert_eq!(float("0xFFFFFFFF"), 4294967295.0);
    assert_eq!(float("0xDEADBEEF"), 3735928559.0);

    let f = float("1.#INF");
    assert!(f.is_infinite() && f > 0.);
    let f = float("1.#IND");
    assert!(f.is_nan());
    let f = float("1#INF");
    assert!(f.is_infinite() && f > 0.);
    let f = float("1#IND");
    assert!(f.is_nan());
}

#[test]
fn integer_literals_before_515_1644() {
    let lex_one = |text: &str, clamp: bool| {
        let context = dm::Context::default();
        context.set_clamp_integer_literals(clamp);
        let tokens: Vec<Token> = Lexer::new(&context, FileId::UNKNOWN, text.as_bytes())
            .map(|t| t.token)
            .collect();
        context.assert_success();
        tokens[0].clone()
    };
    for text in ["3000000000", "0xFFFFFFFF", "0x123456789", "037777777777", "99999999999999999999"] {
        assert_eq!(lex_one(text, true), Float(2147483647.0), "{text}");
    }
    assert_eq!(lex_one("3000000000", false), Float(3000000000.0));
    assert_eq!(lex_one("2147483647", true), Int(2147483647));
    assert_eq!(lex_one("3e9", true), Float(3e9));
    assert_eq!(lex_one("3000000000.0", true), Float(3e9));
}

#[test]
fn nested_interpolation() {
    assert_eq!(
        lex(r#""A[B"C"D]E""#),
        vec![
            InterpStringBegin("A".into()),
            Ident("B".into(), false),
            Token::String("C".into(), StringKind::Normal),
            Ident("D".into(), false),
            InterpStringEnd("E".into()),
            Token!['\n'],
        ]
    );
}

#[test]
fn empty_block_comment() {
    // This is legal. It should not do either of the following:
    // - Error with "still skipping comments at end of file"
    // - Yield a DocComment { text: "", .. }
    assert_eq!(lex(r#"/**/"#), vec![Token!['\n']])
}

#[test]
fn raw_strings() {
    let desired = Token::String("content".into(), StringKind::Raw);
    let stuff = lex(r#"
@"content"
@xcontentx
@/content/
@(x)contentx
@(EOD)contentEOD
@(very long terminator)contentvery long terminator
@{"content"}
@{content{
"#);
    for each in stuff.iter() {
        if each == &Token!['\n'] {
            continue;
        }
        assert_eq!(each, &desired);
    }
}

#[test]
fn heredoc_with_quotes() {
    // 1-3 quotes in the middle of ordinary characters
    assert_eq!(
        lex(r#"{"foo"bar"}"#),
        vec![
            Token::String(r#"foo"bar"#.into(), StringKind::Document),
            Token!['\n']
        ]
    );
    assert_eq!(
        lex(r#"{"foo""bar"}"#),
        vec![
            Token::String(r#"foo""bar"#.into(), StringKind::Document),
            Token!['\n']
        ]
    );
    assert_eq!(
        lex(r#"{"foo"""bar"}"#),
        vec![
            Token::String(r#"foo"""bar"#.into(), StringKind::Document),
            Token!['\n']
        ]
    );

    // 0-5 quotes at the start/end
    assert_eq!(
        lex(r#"{""}"#),
        vec![
            Token::String(r#""#.into(), StringKind::Document),
            Token!['\n']
        ]
    );
    assert_eq!(
        lex(r#"{"""}"#),
        vec![
            Token::String(r#"""#.into(), StringKind::Document),
            Token!['\n']
        ]
    );
    assert_eq!(
        lex(r#"{""""}"#),
        vec![
            Token::String(r#""""#.into(), StringKind::Document),
            Token!['\n']
        ]
    );
    assert_eq!(
        lex(r#"{"""""}"#),
        vec![
            Token::String(r#"""""#.into(), StringKind::Document),
            Token!['\n']
        ]
    );
}
