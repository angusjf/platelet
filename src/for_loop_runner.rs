use serde_json::Value;

use crate::{expression_eval::eval, for_loop_parser::ForLoop};

pub(crate) fn for_loop_runner(for_loop: &ForLoop, base_context: &Value) -> Result<Vec<Value>, ()> {
    match for_loop {
        ForLoop::Simple(id, exp) => {
            let val = eval(exp, base_context).unwrap();
            match val {
                Value::Array(vec) => Ok(vec
                    .iter()
                    .map(|v| {
                        let mut obj = base_context.as_object().unwrap().clone();
                        obj.insert(id.clone(), v.clone());
                        Value::Object(obj)
                    })
                    .collect()),
                _ => todo!(),
            }
        }
        ForLoop::IndexedObjectOrKeyValue(ids, exp) => {
            let val = eval(exp, base_context).unwrap();
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
                _ => todo!(),
            }
        }
        ForLoop::IndexedKeyValue(_, _) => todo!(),
    }
}
