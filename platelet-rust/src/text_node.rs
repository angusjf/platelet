use std::borrow::Cow;

use regex::{Captures, Regex};
use serde_json::Value;

use crate::{
    expression_eval::{eval, EvalError},
    expression_parser::expr,
    types::Type,
};

#[derive(Debug, PartialEq)]
pub enum RenderError {
    EvalError(EvalError),
    ParserError(String),
    RenderError(Type),
}

pub(crate) fn render_text_node<'a>(
    txt: &'a str,
    vars: &Value,
) -> Result<Cow<'a, str>, RenderError> {
    let hole_re = Regex::new(r"\{\{(.*?)\}\}").unwrap();

    let mut error = Ok(());

    let out = hole_re.replace_all(txt, |captures: &Captures| {
        let exp_s = captures[1].to_string();
        match expr(&mut exp_s.as_str()) {
            Ok(exp) => match eval(&exp, vars) {
                Ok(s) => match stringify(&s) {
                    Ok(s) => s,
                    Err(t) => {
                        error = Err(RenderError::RenderError(t));
                        exp_s
                    }
                },
                Err(e) => {
                    error = Err(RenderError::EvalError(e));
                    exp_s
                }
            },
            Err(e) => {
                error = Err(RenderError::ParserError(e.to_string()));
                exp_s
            }
        }
    });

    error.map(|()| out)
}

fn stringify(v: &Value) -> Result<String, Type> {
    match v {
        Value::Null => Ok("".to_owned()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Number(n) => Ok(n.to_string()),
        Value::String(s) => Ok(s.to_owned()),
        Value::Array(_) => Err(Type::Array),
        Value::Object(_) => Err(Type::Object),
    }
}
