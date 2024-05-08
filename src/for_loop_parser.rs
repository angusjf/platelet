use serde_json::Number;
use std::collections::HashMap;
use std::str::{self, FromStr};
use winnow::combinator::{opt, seq};
use winnow::stream::ParseSlice;

use winnow::ascii::{self, alpha1, alphanumeric1, multispace0};
use winnow::error::{ContextError, ParseError};
use winnow::prelude::*;
use winnow::{
    ascii::float,
    combinator::alt,
    combinator::cut_err,
    combinator::{delimited, preceded, separated_pair, terminated},
    combinator::{repeat, separated},
    error::{AddContext, ParserError},
    token::{any, none_of, take, take_while},
};

use crate::expression_parser::{expression, identifier, ws, Expression};

#[derive(Debug, PartialEq)]
pub(crate) enum ForLoop {
    // item in items
    Simple(String, Expression),
    // (item, index) in items
    // (value, key) in object
    IndexedObjectOrKeyValue((String, String), Expression),
    // (value, name, index) in object
    IndexedKeyValue(String, String, Expression),
}

pub(crate) fn for_loop<'a>(
    input: &'a mut &str,
) -> Result<ForLoop, ParseError<&'a str, ContextError>> {
    delimited(ws, for_, ws).parse(input)
}

fn for_(input: &mut &str) -> PResult<ForLoop> {
    alt((
        separated_pair(identifier, (ws, "in", ws), expression)
            .map(|(id, exp)| ForLoop::Simple(id, exp)),
        delimited(
            ('(', ws),
            separated_pair(
                separated_pair(identifier, (ws, ',', ws), identifier),
                (ws, "in", ws),
                expression,
            ),
            (ws, ')'),
        )
        .map(|(p, exp)| ForLoop::IndexedObjectOrKeyValue(p, exp)),
    ))
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
}
