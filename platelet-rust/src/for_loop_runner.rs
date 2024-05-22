use serde_json::Value;

use crate::{
    expression_eval::{eval, EvalError},
    for_loop_parser::ForLoop,
    types::{type_of, Type},
};

#[derive(Debug)]
pub(crate) enum Error {
    TypeMismatch { expected: Vec<Type>, found: Type },
    Eval(EvalError),
}

pub(crate) fn for_loop_runner(
    for_loop: &ForLoop,
    base_context: &Value,
) -> Result<Vec<Value>, String> {
    fl(for_loop, base_context).map_err(|e| match e {
        Error::TypeMismatch { expected, found } => {
            format!(
                "Expected {}, found {}",
                expected
                    .iter()
                    .map(Type::to_string)
                    .collect::<Vec<_>>()
                    .join(" or "),
                found.to_string()
            )
        }

        Error::Eval(e) => format!("{:?}", e),
    })
}

fn fl(for_loop: &ForLoop, base_context: &Value) -> Result<Vec<Value>, Error> {
    match for_loop {
        ForLoop::Simple(id, exp) => {
            let val = eval(exp, base_context).map_err(Error::Eval)?;
            match val {
                Value::Array(vec) => Ok(vec
                    .iter()
                    .map(|v| {
                        let mut obj = base_context.as_object().unwrap().clone();
                        obj.insert(id.clone(), v.clone());
                        Value::Object(obj)
                    })
                    .collect()),
                _ => Err(Error::TypeMismatch {
                    expected: vec![Type::Array],
                    found: type_of(&val),
                }),
            }
        }
        ForLoop::IndexedObjectOrKeyValue(ids, exp) => {
            let val = eval(exp, base_context).map_err(Error::Eval)?;
            match val {
                Value::Array(vec) => {
                    let (id, indexer) = ids;
                    Ok(vec
                        .iter()
                        .enumerate()
                        .map(|(index, v)| {
                            let mut obj = base_context.as_object().unwrap().clone();
                            obj.insert(id.clone(), v.clone());
                            obj.insert(indexer.clone(), index.clone().into());
                            Value::Object(obj)
                        })
                        .collect())
                }
                Value::Object(vec) => {
                    let (key_id, value_id) = ids;
                    Ok(vec
                        .iter()
                        .map(|(k, v)| {
                            let mut obj = base_context.as_object().unwrap().clone();
                            obj.insert(key_id.clone(), k.clone().into());
                            obj.insert(value_id.clone(), v.clone());
                            Value::Object(obj)
                        })
                        .collect())
                }
                _ => Err(Error::TypeMismatch {
                    expected: vec![Type::Array, Type::Object],
                    found: type_of(&val),
                }),
            }
        }
        ForLoop::IndexedKeyValue(ids, exp) => {
            let val = eval(exp, base_context).map_err(Error::Eval)?;
            match val {
                Value::Object(vec) => {
                    let (key_id, value_id, indexer) = ids;
                    Ok(vec
                        .iter()
                        .enumerate()
                        .map(|(index, (k, v))| {
                            let mut obj = base_context.as_object().unwrap().clone();
                            obj.insert(key_id.clone(), k.clone().into());
                            obj.insert(value_id.clone(), v.clone());
                            obj.insert(indexer.clone(), index.clone().into());
                            Value::Object(obj)
                        })
                        .collect())
                }
                _ => Err(Error::TypeMismatch {
                    expected: vec![Type::Object],
                    found: type_of(&val),
                }),
            }
        }
    }
}
