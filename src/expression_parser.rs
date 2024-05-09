use serde_json::Number;
use std::collections::HashMap;
use winnow::combinator::opt;

use winnow::ascii;
use winnow::error::{ContextError, ParseError};
use winnow::prelude::*;
use winnow::{
    combinator::alt,
    combinator::cut_err,
    combinator::{delimited, preceded, separated_pair, terminated},
    combinator::{repeat, separated},
    error::ParserError,
    token::{any, none_of, take, take_while},
};

pub(crate) fn expr<'a>(
    input: &'a mut &str,
) -> Result<Expression, ParseError<&'a str, ContextError>> {
    delimited(ws, expression, ws).parse(input)
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Expression {
    Indexed(Box<(Expression, Expression)>),
    BinaryOperation(Box<(Expression, BinaryOperator, Expression)>),
    FunctionCall(Box<(String, Expression)>),
    UnaryOperation(Box<(UnaryOperator, Expression)>),
    Conditional(Box<(Expression, Expression, Expression)>),
    Null,
    Boolean(bool),
    Str(String),
    Num(Number),
    Array(Vec<Expression>),
    Object(HashMap<String, Expression>),
    Identifier(String),
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    EqualTo,
    NotEqualTo,
    GreaterThan,
    GreterThanOrEqualTo,
    LessThan,
    LessThanOrEqualTo,
    Or,
    And,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum UnaryOperator {
    Not,
}

pub(crate) fn expression(input: &mut &str) -> PResult<Expression> {
    conditional_expression.parse_next(input)
}

fn conditional_expression(input: &mut &str) -> PResult<Expression> {
    let cond = or_expression.parse_next(input)?;
    if let Ok((x, y)) = (
        preceded((ws, '?', ws), expression),
        preceded((ws, ':', ws), conditional_expression),
    )
        .parse_next(input)
    {
        Ok(Expression::Conditional(Box::new((cond, x, y))))
    } else {
        Ok(cond)
    }
}

fn or_expression(input: &mut &str) -> PResult<Expression> {
    let x = and_expression.parse_next(input)?;
    if let Ok(op) = delimited(ws, "||".value(BinaryOperator::Or), ws).parse_next(input) {
        let y = or_expression(input)?;
        return Ok(Expression::BinaryOperation(Box::new((x, op, y))));
    } else {
        return Ok(x);
    }
}

fn and_expression(input: &mut &str) -> PResult<Expression> {
    let x = comparison_expression.parse_next(input)?;
    if let Ok(op) = delimited(ws, "&&".value(BinaryOperator::And), ws).parse_next(input) {
        let y = and_expression(input)?;
        return Ok(Expression::BinaryOperation(Box::new((x, op, y))));
    } else {
        return Ok(x);
    }
}

fn comparison_expression(input: &mut &str) -> PResult<Expression> {
    let x = modulo_expression.parse_next(input)?;
    if let Ok(op) = delimited(
        ws,
        alt((
            ">=".value(BinaryOperator::GreterThanOrEqualTo),
            ">".value(BinaryOperator::GreaterThan),
            "<=".value(BinaryOperator::LessThanOrEqualTo),
            "<".value(BinaryOperator::LessThan),
            "==".value(BinaryOperator::EqualTo),
            "!=".value(BinaryOperator::NotEqualTo),
        )),
        ws,
    )
    .parse_next(input)
    {
        let y = comparison_expression(input)?;
        return Ok(Expression::BinaryOperation(Box::new((x, op, y))));
    } else {
        return Ok(x);
    }
}

fn modulo_expression(input: &mut &str) -> PResult<Expression> {
    let x = additive_expression.parse_next(input)?;
    if let Ok(op) = delimited(ws, "%".value(BinaryOperator::Modulo), ws).parse_next(input) {
        let y = modulo_expression(input)?;
        return Ok(Expression::BinaryOperation(Box::new((x, op, y))));
    } else {
        return Ok(x);
    }
}

fn additive_expression(input: &mut &str) -> PResult<Expression> {
    let x = multiplicative_expression.parse_next(input)?;
    if let Ok(op) = delimited(
        ws,
        alt((
            "+".value(BinaryOperator::Add),
            "-".value(BinaryOperator::Subtract),
        )),
        ws,
    )
    .parse_next(input)
    {
        let y = additive_expression(input)?;
        return Ok(Expression::BinaryOperation(Box::new((x, op, y))));
    } else {
        return Ok(x);
    }
}

fn multiplicative_expression(input: &mut &str) -> PResult<Expression> {
    let x = unary_expression.parse_next(input)?;
    if let Ok(op) = delimited(
        ws,
        alt((
            "*".value(BinaryOperator::Multiply),
            "/".value(BinaryOperator::Divide),
        )),
        ws,
    )
    .parse_next(input)
    {
        let y = multiplicative_expression(input)?;
        return Ok(Expression::BinaryOperation(Box::new((x, op, y))));
    } else {
        return Ok(x);
    }
}

fn unary_expression(input: &mut &str) -> PResult<Expression> {
    if let Ok(exp) = preceded(('!', ws), indexed_expression).parse_next(input) {
        return Ok(Expression::UnaryOperation(Box::new((
            UnaryOperator::Not,
            exp,
        ))));
    } else {
        return indexed_expression.parse_next(input);
    }
}

fn indexed_expression(input: &mut &str) -> PResult<Expression> {
    let exp = dot_access_expression.parse_next(input)?;
    if let Ok(index) =
        preceded(ws, delimited('[', delimited(ws, expression, ws), ']')).parse_next(input)
    {
        return Ok(Expression::Indexed(Box::new((exp, index))));
    } else {
        return Ok(exp);
    }
}

fn dot_access_expression(input: &mut &str) -> PResult<Expression> {
    let mut exp = primary_expression.parse_next(input)?;
    while let Ok(index) = preceded((ws, '.', ws), identifier).parse_next(input) {
        exp = Expression::Indexed(Box::new((exp, Expression::Str(index))));
    }
    return Ok(exp);
}

fn primary_expression(input: &mut &str) -> PResult<Expression> {
    alt((
        delimited('(', delimited(ws, expression, ws), ')'),
        null.value(Expression::Null),
        boolean.map(Expression::Boolean),
        string.map(Expression::Str),
        number.map(Expression::Num),
        array.map(Expression::Array),
        object.map(Expression::Object),
        identifier.map(Expression::Identifier),
    ))
    .parse_next(input)
}

//     ascii_float.map(|f| {
//         Number::from_f64(f)
//             .expect("number should not be NaN or Infinite as this is not valid JSON")
//     }),
//     ascii::dec_uint::<_, u32, _>.map(Number::from),
//     ascii::dec_int::<_, i32, _>.map(Number::from),

pub fn number(input: &mut &str) -> PResult<Number> {
    (|input: &mut &str| {
        let s = recognize_float(input)?;
        categorize_num(s).ok_or_else(|| {
            winnow::error::ErrMode::from_error_kind(input, winnow::error::ErrorKind::Verify)
        })
    })
    .parse_next(input)
}

fn categorize_num(s: &str) -> Option<Number> {
    if s.contains('.') || s.contains('e') || s.contains('E') {
        return s.parse().map(|x: f64| Number::from_f64(x).unwrap()).ok();
    } else if s.starts_with('-') {
        return s.parse::<i32>().map(|x| Number::from(x)).ok();
    } else {
        return s.parse::<u32>().map(|x| Number::from(x)).ok();
    }
}

fn recognize_float<'a>(input: &mut &'a str) -> PResult<&'a str> {
    (
        opt(winnow::token::one_of(['+', '-'])),
        alt((
            (ascii::digit1, opt(('.', opt(ascii::digit1)))).void(),
            ('.', ascii::digit1).void(),
        )),
        opt((
            winnow::token::one_of(['e', 'E']),
            opt(winnow::token::one_of(['+', '-'])),
            cut_err(ascii::digit1),
        )),
    )
        .recognize()
        .parse_next(input)
}

fn null<'s>(input: &mut &'s str) -> PResult<&'s str> {
    "null".parse_next(input)
}

fn boolean(input: &mut &str) -> PResult<bool> {
    alt(("true".value(true), "false".value(false))).parse_next(input)
}

fn string(input: &mut &str) -> PResult<String> {
    preceded(
        '\"',
        // `cut_err` transforms an `ErrMode::Backtrack(e)` to `ErrMode::Cut(e)`, signaling to
        // combinators like  `alt` that they should not try other parsers. We were in the
        // right branch (since we found the `"` character) but encountered an error when
        // parsing the string
        cut_err(terminated(
            repeat(0.., character).fold(String::new, |mut string, c| {
                string.push(c);
                string
            }),
            '\"',
        )),
    )
    // `context` lets you add a static string to errors to provide more information in the
    // error chain (to indicate which parser had an error)
    .parse_next(input)
}

// fn multi_identifier(input: &mut &str) -> PResult<Vec<String>> {
//     separated(1.., identifier, ".")
//         // .map(|x: Vec<_>| x.iter().map(|s| s.to_string()).collect::<Vec<_>>())
//         .parse_next(input)
// }

pub(crate) fn identifier<'s>(input: &'s mut &str) -> PResult<String> {
    take_while(1.., ('a'..='z', 'A'..='Z'))
        .parse_next(input)
        .map(|s| s.to_string())
}

/// You can mix the above declarative parsing with an imperative style to handle more unique cases,
/// like escaping
fn character(input: &mut &str) -> PResult<char> {
    let c = none_of('\"').parse_next(input)?;
    if c == '\\' {
        alt((
            any.verify_map(|c| {
                Some(match c {
                    '"' | '\\' | '/' => c,
                    'b' => '\x08',
                    'f' => '\x0C',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    _ => return None,
                })
            }),
            preceded('u', unicode_escape),
        ))
        .parse_next(input)
    } else {
        Ok(c)
    }
}

fn unicode_escape<'s>(input: &mut &'s str) -> PResult<char> {
    alt((
        // Not a surrogate
        u16_hex
            .verify(|cp| !(0xD800..0xE000).contains(cp))
            .map(|cp| cp as u32),
        // See https://en.wikipedia.org/wiki/UTF-16#Code_points_from_U+010000_to_U+10FFFF for details
        separated_pair(u16_hex, "\\u", u16_hex)
            .verify(|(high, low)| (0xD800..0xDC00).contains(high) && (0xDC00..0xE000).contains(low))
            .map(|(high, low)| {
                let high_ten = (high as u32) - 0xD800;
                let low_ten = (low as u32) - 0xDC00;
                (high_ten << 10) + low_ten + 0x10000
            }),
    ))
    .verify_map(
        // Could be probably replaced with .unwrap() or _unchecked due to the verify checks
        std::char::from_u32,
    )
    .parse_next(input)
}

fn u16_hex(input: &mut &str) -> PResult<u16> {
    take(4usize)
        .verify_map(|s| u16::from_str_radix(s, 16).ok())
        .parse_next(input)
}

/// Some combinators, like `separated` or `repeat`, will call a parser repeatedly,
/// accumulating results in a `Vec`, until it encounters an error.
/// If you want more control on the parser application, check out the `iterator`
/// combinator (cf `examples/iterator.rs`)
fn array(input: &mut &str) -> PResult<Vec<Expression>> {
    preceded(
        ('[', ws),
        cut_err(terminated(
            separated(0.., expression, (ws, ',', ws)),
            (ws, ']'),
        )),
    )
    .parse_next(input)
}

fn object(input: &mut &str) -> PResult<HashMap<String, Expression>> {
    preceded(
        ('{', ws),
        cut_err(terminated(
            separated(0.., key_value, (ws, ',', ws)),
            (ws, '}'),
        )),
    )
    .parse_next(input)
}

fn key_value(input: &mut &str) -> PResult<(String, Expression)> {
    separated_pair(string, cut_err((ws, ':', ws)), expression).parse_next(input)
}

pub(crate) fn ws<'s>(input: &mut &'s str) -> PResult<&'s str> {
    take_while(0.., WS).parse_next(input)
}

const WS: &[char] = &[' ', '\t', '\r', '\n'];

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn json_string() {
        assert_eq!(string.parse_peek("\"\""), Ok(("", "".to_owned())));
        assert_eq!(string.parse_peek("\"abc\""), Ok(("", "abc".to_owned())));
        assert_eq!(
            string.parse_peek("\"abc\\\"\\\\\\/\\b\\f\\n\\r\\t\\u0001\\u2014\u{2014}def\""),
            Ok(("", "abc\"\\/\x08\x0C\n\r\t\x01——def".to_owned())),
        );
        assert_eq!(
            string.parse_peek("\"\\uD83D\\uDE10\""),
            Ok(("", "😐".to_owned()))
        );

        assert!(string.parse_peek("\"").is_err());
        assert!(string.parse_peek("\"abc").is_err());
        assert!(string.parse_peek("\"\\\"").is_err());
        assert!(string.parse_peek("\"\\u123\"").is_err());
        assert!(string.parse_peek("\"\\uD800\"").is_err());
        assert!(string.parse_peek("\"\\uD800\\uD800\"").is_err());
        assert!(string.parse_peek("\"\\uDC00\"").is_err());
    }

    #[test]
    fn json_object() {
        use Expression::{Num, Str};

        let input = r#"{"a":42,"b":"x"}"#;

        let expected = Expression::Object(
            vec![
                ("a".to_owned(), Num(42.into())),
                ("b".to_owned(), Str("x".to_owned())),
            ]
            .into_iter()
            .collect(),
        );

        assert_eq!(expression.parse_peek(input), Ok(("", expected)));
    }

    #[test]
    fn json_array() {
        use Expression::{Num, Str};

        let input = r#"[42,"x"]"#;

        let expected = Expression::Array(vec![Num(42.into()), Str("x".to_owned())]);

        assert_eq!(expression.parse_peek(input), Ok(("", expected)));
    }

    #[test]
    fn json_whitespace() {
        use Expression::{Array, Boolean, Null, Num, Object, Str};

        let input = r#"{
      "null" : null,
      "true"  :true ,
      "false":  false  ,
      "number" : 123e4 ,
      "string" : " abc 123 " ,
      "array" : [ false , 1 , "two" ] ,
      "object" : { "a" : 1.0 , "b" : "c" } ,
      "empty_array" : [  ] ,
      "empty_object" : {   }
    }"#;

        assert_eq!(
            expression.parse_peek(input),
            Ok((
                "",
                Expression::Object(
                    vec![
                        ("null".to_owned(), Null),
                        ("true".to_owned(), Boolean(true)),
                        ("false".to_owned(), Boolean(false)),
                        ("number".to_owned(), Num(Number::from_f64(123e4).unwrap())),
                        ("string".to_owned(), Str(" abc 123 ".to_owned())),
                        (
                            "array".to_owned(),
                            Array(vec![Boolean(false), Num(1.into()), Str("two".to_owned())])
                        ),
                        (
                            "object".to_owned(),
                            Object(
                                vec![
                                    ("a".to_owned(), Num(Number::from_f64(1.0).unwrap())),
                                    ("b".to_owned(), Str("c".to_owned())),
                                ]
                                .into_iter()
                                .collect()
                            )
                        ),
                        ("empty_array".to_owned(), Array(vec![]),),
                        ("empty_object".to_owned(), Object(HashMap::new()),),
                    ]
                    .into_iter()
                    .collect()
                )
            ))
        );
    }

    #[test]
    fn indexed_expressions() {
        use Expression::{Num, Object};

        let input = r#"{ "z": 1 }[0]"#;

        assert_eq!(
            expression.parse_peek(input),
            Ok((
                "",
                Expression::Indexed(Box::new((
                    Object(vec![("z".to_owned(), Num(1.into()))].into_iter().collect()),
                    Num(0.into())
                )))
            ))
        )
    }

    #[test]
    fn indexed_expressions_2() {
        use Expression::{Num, Object};

        let input = r#"{ "z": 1 } [ 0 ]"#;

        assert_eq!(
            expression.parse_peek(input),
            Ok((
                "",
                Expression::Indexed(Box::new((
                    Object(vec![("z".to_owned(), Num(1.into()))].into_iter().collect()),
                    Num(0.into())
                )))
            ))
        )
    }

    #[test]
    fn multi_identifiers_0() {
        let input = r#"window"#;

        assert_eq!(identifier.parse_peek(input), Ok(("", "window".to_owned())))
    }

    #[test]
    fn expression_multi_identifier() {
        let input = r#"props.user.name"#;

        assert_eq!(
            expression.parse_peek(input),
            Ok((
                "",
                Expression::Indexed(Box::new((
                    Expression::Indexed(Box::new((
                        Expression::Identifier("props".to_owned()),
                        Expression::Str("user".to_owned())
                    ))),
                    Expression::Str("name".to_owned())
                ))),
            ))
        )
    }

    #[test]
    fn expression_or() {
        use Expression::{BinaryOperation, Num};
        let input = r#"props.user.name || 1.0"#;

        assert_eq!(
            expression.parse_peek(input),
            Ok((
                "",
                BinaryOperation(Box::new((
                    Expression::Indexed(Box::new((
                        Expression::Indexed(Box::new((
                            Expression::Identifier("props".to_owned()),
                            Expression::Str("user".to_owned())
                        ))),
                        Expression::Str("name".to_owned())
                    ))),
                    BinaryOperator::Or,
                    Num(Number::from_f64(1.0).unwrap())
                )))
            ))
        )
    }

    #[test]
    fn expression_and() {
        use Expression::{Array, BinaryOperation, Indexed, Num, Object};
        let input = r#"[1] && {}[3]"#;

        assert_eq!(
            expression.parse_peek(input),
            Ok((
                "",
                BinaryOperation(Box::new((
                    Array(vec![Num(1.into())]),
                    BinaryOperator::And,
                    Indexed(Box::new((Object(HashMap::new()), Num(3.into()))))
                )))
            ))
        )
    }

    #[test]
    fn expression_add() {
        use Expression::{BinaryOperation, Identifier, Num};
        let input = r#"name + 1.0"#;

        assert_eq!(
            expression.parse_peek(input),
            Ok((
                "",
                BinaryOperation(Box::new((
                    Identifier("name".to_owned()),
                    BinaryOperator::Add,
                    Num(Number::from_f64(1.0).unwrap())
                )))
            ))
        )
    }

    #[test]
    fn expression_eq() {
        use Expression::{BinaryOperation, Identifier, Str};
        let input = r#"props == """#;

        assert_eq!(
            expression.parse_peek(input),
            Ok((
                "",
                BinaryOperation(Box::new((
                    Identifier("props".to_owned()),
                    BinaryOperator::EqualTo,
                    Str("".to_owned())
                )))
            ))
        )
    }

    #[test]
    fn expression_indexed() {
        use Expression::{BinaryOperation, Indexed, Object, Str};
        let input = r#"{ "hello": "world" }["hell" + "o"]"#;

        assert_eq!(
            expression.parse_peek(input),
            Ok((
                "",
                Indexed(Box::new((
                    Object(HashMap::from([(
                        "hello".to_owned(),
                        Str("world".to_owned())
                    )])),
                    BinaryOperation(Box::new((
                        Str("hell".to_owned()),
                        BinaryOperator::Add,
                        Str("o".to_owned())
                    )))
                )))
            ))
        )
    }

    #[test]
    fn expression_iif() {
        use Expression::{BinaryOperation, Conditional, Num};
        let input = r#"1 > 2 ? 1 : 2"#;

        assert_eq!(
            expression.parse_peek(input),
            Ok((
                "",
                Conditional(Box::new((
                    BinaryOperation(Box::new((
                        Num(1.into()),
                        BinaryOperator::GreaterThan,
                        Num(2.into())
                    ))),
                    Num(1.into()),
                    Num(2.into())
                )))
            ))
        )
    }

    #[test]
    fn expression_bidmas() {
        use Expression::{BinaryOperation, Num};
        let input = r#"(9 + 3) / 2 == 6"#;

        assert_eq!(
            expression.parse_peek(input),
            Ok((
                "",
                BinaryOperation(Box::new((
                    BinaryOperation(Box::new((
                        BinaryOperation(Box::new((
                            Num(9.into()),
                            BinaryOperator::Add,
                            Num(3.into())
                        ))),
                        BinaryOperator::Divide,
                        Num(2.into())
                    ))),
                    BinaryOperator::EqualTo,
                    Num(6.into())
                )))
            ))
        )
    }

    #[test]
    fn unary_not() {
        let input = r#"!this"#;

        assert_eq!(
            expression.parse_peek(input),
            Ok((
                "",
                Expression::UnaryOperation(Box::new((
                    UnaryOperator::Not,
                    Expression::Identifier("this".to_owned())
                )))
            ))
        )
    }

    #[test]
    fn expression_mod() {
        let input = r#"1%3"#;

        assert_eq!(
            expression.parse_peek(input),
            Ok((
                "",
                Expression::BinaryOperation(Box::new((
                    Expression::Num(1.into()),
                    BinaryOperator::Modulo,
                    Expression::Num(3.into())
                )))
            ))
        )
    }

    #[test]
    fn expression_in_object() {
        let input = r#"{"hello": [ 1 * 2 ] }"#;
        assert_eq!(
            expression.parse_peek(input),
            Ok((
                "",
                Expression::Object(HashMap::from([(
                    "hello".to_owned(),
                    Expression::Array(vec![Expression::BinaryOperation(Box::new((
                        Expression::Num(1.into()),
                        BinaryOperator::Multiply,
                        Expression::Num(2.into())
                    )))])
                )]))
            ))
        )
    }
}
