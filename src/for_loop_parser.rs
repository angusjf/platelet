use winnow::combinator::alt;
use winnow::combinator::{delimited, separated_pair};
use winnow::error::StrContext;
use winnow::prelude::*;

use crate::expression_parser::Expression;
use crate::expression_parser::{expression, identifier, ws};

#[derive(Debug, PartialEq)]
pub(crate) enum ForLoop {
    // item in items
    Simple(String, Expression),
    // (item, index) in items
    // (value, key) in object
    IndexedObjectOrKeyValue((String, String), Expression),
    // (value, name, index) in object
    IndexedKeyValue((String, String, String), Expression),
}

pub(crate) fn for_loop(input: &mut &str) -> Result<ForLoop, String> {
    delimited(ws, for_, ws)
        .parse(input)
        .map_err(|e| format!("{}", e.to_string()))
}

fn for_(input: &mut &str) -> PResult<ForLoop> {
    alt((
        separated_pair(identifier, (ws, "in", ws), expression)
            .map(|(id, exp)| ForLoop::Simple(id, exp)),
        separated_pair(
            delimited(
                ('(', ws),
                separated_pair(identifier, (ws, ',', ws), identifier),
                (ws, ')'),
            ),
            (ws, "in", ws),
            expression,
        )
        .map(|(p, exp)| ForLoop::IndexedObjectOrKeyValue(p, exp)),
        separated_pair(
            delimited(
                ('(', ws),
                separated_pair(
                    identifier,
                    (ws, ',', ws),
                    separated_pair(identifier, (ws, ',', ws), identifier),
                ),
                (ws, ')'),
            ),
            (ws, "in", ws),
            expression,
        )
        .map(|((a, (b, c)), exp)| ForLoop::IndexedKeyValue((a, b, c), exp)),
    ))
    .context(StrContext::Label("for loop"))
    .parse_next(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(
            for_.parse_peek("item in items"),
            Ok((
                "",
                ForLoop::Simple(
                    "item".to_owned(),
                    Expression::Identifier("items".to_owned())
                )
            ))
        );
    }

    #[test]
    fn complex() {
        assert_eq!(
            for_.parse_peek("(item, i) in items"),
            Ok((
                "",
                ForLoop::IndexedObjectOrKeyValue(
                    ("item".to_owned(), "i".to_owned()),
                    Expression::Identifier("items".to_owned())
                )
            ))
        );
    }

    #[test]
    fn indexed_key_value() {
        assert_eq!(
            for_.parse_peek("(key, value, index) in items"),
            Ok((
                "",
                ForLoop::IndexedKeyValue(
                    ("key".to_owned(), "value".to_owned(), "index".to_owned()),
                    Expression::Identifier("items".to_owned())
                )
            ))
        );
    }
}
