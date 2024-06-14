use serde_json::{Map, Number, Value};

use crate::expression_parser::{BinaryOperator, Expression, UnaryOperator};

#[derive(Debug, PartialEq)]
pub enum EvalError {
    TypeMismatch,
    BadArrayIndexError,
    ArrayOutOfBounds,
    UndefinedProperty,
    UndefinedFunction(String),
    Undefined(String),
}

pub(crate) fn eval(exp: &Expression, vars: &Value) -> Result<Value, EvalError> {
    match exp {
        Expression::Indexed(indexed_exp) => {
            let (subject, index) = indexed_exp.as_ref();
            let subject = eval(subject, vars)?;
            let index = eval(index, vars)?;
            match (subject, index) {
                (Value::Array(a), Value::Number(n)) => {
                    let n: usize = n.as_f64().ok_or(EvalError::BadArrayIndexError)? as usize;
                    let v: &Value = a.get(n).ok_or(EvalError::ArrayOutOfBounds)?;
                    Ok(v.clone())
                }
                (Value::Object(o), Value::String(s)) => {
                    let v = o.get(&s).ok_or(EvalError::UndefinedProperty)?;
                    Ok(v.clone())
                }
                (Value::String(s), Value::Number(n)) => {
                    let n: usize = n.as_f64().ok_or(EvalError::BadArrayIndexError)? as usize;
                    let v = s.chars().nth(n).ok_or(EvalError::ArrayOutOfBounds)?;
                    Ok(v.to_string().into())
                }
                _ => Err(EvalError::TypeMismatch),
            }
        }
        Expression::BinaryOperation(bin_op_exp) => {
            let (a, op, b) = bin_op_exp.as_ref();
            let a = eval(a, vars)?;
            let b = eval(b, vars)?;
            match op {
                BinaryOperator::Add => match (a, b) {
                    (Value::Number(n), Value::Number(m)) => {
                        if !n.is_f64() && !m.is_f64() {
                            let sum = n.as_i64().unwrap() + m.as_i64().unwrap();
                            Ok(Value::Number(sum.into()))
                        } else {
                            let sum = n.as_f64().unwrap() + m.as_f64().unwrap();
                            Ok(Value::Number(Number::from_f64(sum).unwrap()))
                        }
                    }
                    (Value::Number(n), Value::String(s)) => Ok(Value::String(n.to_string() + &s)),
                    (Value::String(s), Value::Number(n)) => Ok(Value::String(s + &n.to_string())),
                    (Value::String(n), Value::String(m)) => Ok(Value::String(n + &m)),
                    (Value::Array(mut a), Value::Array(mut b)) => {
                        a.append(&mut b);
                        Ok(Value::Array(a))
                    }
                    (Value::Object(mut a), Value::Object(mut b)) => {
                        a.append(&mut b);
                        Ok(Value::Object(a))
                    }
                    _ => Err(EvalError::TypeMismatch),
                },
                BinaryOperator::Subtract => match (a, b) {
                    (Value::Number(n), Value::Number(m)) => {
                        if !n.is_f64() && !m.is_f64() {
                            let sum = n.as_i64().unwrap() - m.as_i64().unwrap();
                            Ok(Value::Number(sum.into()))
                        } else {
                            let sum = n.as_f64().unwrap() - m.as_f64().unwrap();
                            Ok(Value::Number(Number::from_f64(sum).unwrap()))
                        }
                    }
                    _ => Err(EvalError::TypeMismatch),
                },
                BinaryOperator::Multiply => match (a, b) {
                    (Value::Number(n), Value::Number(m)) => {
                        if !n.is_f64() && !m.is_f64() {
                            let sum = n.as_i64().unwrap() * m.as_i64().unwrap();
                            Ok(Value::Number(sum.into()))
                        } else {
                            let sum = n.as_f64().unwrap() * m.as_f64().unwrap();
                            Ok(Value::Number(Number::from_f64(sum).unwrap()))
                        }
                    }
                    _ => Err(EvalError::TypeMismatch),
                },
                BinaryOperator::Divide => match (a, b) {
                    (Value::Number(n), Value::Number(m)) => {
                        if !n.is_f64() && !m.is_f64() {
                            let sum = n.as_i64().unwrap() / m.as_i64().unwrap();
                            Ok(Value::Number(sum.into()))
                        } else {
                            let sum = n.as_f64().unwrap() / m.as_f64().unwrap();
                            Ok(Value::Number(Number::from_f64(sum).unwrap()))
                        }
                    }
                    _ => Err(EvalError::TypeMismatch),
                },
                BinaryOperator::Modulo => match (a, b) {
                    (Value::Number(n), Value::Number(m)) => {
                        if !n.is_f64() && !m.is_f64() {
                            let sum = n.as_i64().unwrap() % m.as_i64().unwrap();
                            Ok(Value::Number(sum.into()))
                        } else {
                            let sum = n.as_f64().unwrap() % m.as_f64().unwrap();
                            Ok(Value::Number(Number::from_f64(sum).unwrap()))
                        }
                    }
                    _ => Err(EvalError::TypeMismatch),
                },
                BinaryOperator::EqualTo => Ok(Value::Bool(are_equal(&a, &b))),
                BinaryOperator::NotEqualTo => Ok(Value::Bool(!are_equal(&a, &b))),
                BinaryOperator::GreaterThan => match (a, b) {
                    (Value::Number(n), Value::Number(m)) => {
                        if !n.is_f64() && !m.is_f64() {
                            let result = n.as_i64().unwrap() > m.as_i64().unwrap();
                            Ok(Value::Bool(result))
                        } else {
                            let result = n.as_f64().unwrap() > m.as_f64().unwrap();
                            Ok(Value::Bool(result))
                        }
                    }
                    _ => Err(EvalError::TypeMismatch),
                },
                BinaryOperator::GreterThanOrEqualTo => match (a, b) {
                    (Value::Number(n), Value::Number(m)) => {
                        if !n.is_f64() && !m.is_f64() {
                            let result = n.as_i64().unwrap() >= m.as_i64().unwrap();
                            Ok(Value::Bool(result))
                        } else {
                            let result = n.as_f64().unwrap() >= m.as_f64().unwrap();
                            Ok(Value::Bool(result))
                        }
                    }
                    _ => Err(EvalError::TypeMismatch),
                },
                BinaryOperator::LessThan => match (a, b) {
                    (Value::Number(n), Value::Number(m)) => {
                        if !n.is_f64() && !m.is_f64() {
                            let result = n.as_i64().unwrap() < m.as_i64().unwrap();
                            Ok(Value::Bool(result))
                        } else {
                            let result = n.as_f64().unwrap() < m.as_f64().unwrap();
                            Ok(Value::Bool(result))
                        }
                    }
                    _ => Err(EvalError::TypeMismatch),
                },
                BinaryOperator::LessThanOrEqualTo => match (a, b) {
                    (Value::Number(n), Value::Number(m)) => {
                        if !n.is_f64() && !m.is_f64() {
                            let result = n.as_i64().unwrap() <= m.as_i64().unwrap();
                            Ok(Value::Bool(result))
                        } else {
                            let result = n.as_f64().unwrap() <= m.as_f64().unwrap();
                            Ok(Value::Bool(result))
                        }
                    }
                    _ => Err(EvalError::TypeMismatch),
                },
                BinaryOperator::Or => {
                    if truthy(&a) {
                        Ok(a)
                    } else {
                        Ok(b)
                    }
                }
                BinaryOperator::And => {
                    if truthy(&a) {
                        Ok(b)
                    } else {
                        Ok(false.into())
                    }
                }
            }
        }
        Expression::FunctionCall(fn_call) => {
            let (id, arg) = fn_call.as_ref();
            match id.as_str() {
                "len" => match eval(arg, vars)? {
                    Value::Array(a) => Ok(a.len().into()),
                    Value::String(s) => Ok(s.len().into()),
                    Value::Object(o) => Ok(o.len().into()),
                    _ => Err(EvalError::TypeMismatch),
                },
                _ => Err(EvalError::UndefinedFunction(id.clone())),
            }
        }
        Expression::UnaryOperation(un_op) => {
            let (UnaryOperator::Not, exp) = un_op.as_ref();
            let exp = eval(exp, vars)?;
            Ok(Value::Bool(!truthy(&exp)))
        }
        Expression::Conditional(cond_exp) => {
            let (cond, tru, fal) = cond_exp.as_ref();
            let cond = eval(cond, vars)?;
            eval(if truthy(&cond) { tru } else { fal }, vars)
        }
        Expression::Null => Ok(Value::Null),
        Expression::Boolean(v) => Ok(Value::Bool(*v)),
        Expression::Str(s) => Ok(Value::String(s.clone())),
        Expression::Num(n) => Ok(Value::Number(n.clone())),
        Expression::Array(a) => Ok(a.iter().map(|e| eval(e, vars)).collect::<Result<_, _>>()?),
        Expression::Object(o) => {
            let o: Map<_, _> = o
                .iter()
                .map(|(k, v)| {
                    let v = eval(v, vars)?;
                    Ok((k.clone(), v))
                })
                .collect::<Result<_, EvalError>>()?;
            Ok(Value::Object(o))
        }
        Expression::Identifier(id) => match vars {
            Value::Object(o) => {
                let v = o.get(id).unwrap_or(&Value::Null);
                Ok(v.clone())
            }
            _ => Err(EvalError::TypeMismatch),
        },
    }
}

fn are_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            if a.is_f64() && b.is_f64() {
                a.as_f64().unwrap() == b.as_f64().unwrap()
            } else {
                a.as_f64().unwrap() as i64 == b.as_f64().unwrap() as i64
            }
        }
        _ => a == b,
    }
}

pub(crate) fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(v) => *v,
        Value::Number(n) => n.as_f64().unwrap() != 0.0,
        Value::String(s) => s != "",
        Value::Array(a) => a.len() != 0,
        Value::Object(o) => !o.is_empty(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::expression_parser::expr;
    use serde_json::{json, Map};

    #[test]
    fn null() {
        let mut null = "null";
        let vars = Map::new().into();
        let null = expr(&mut null).unwrap();
        assert_eq!(eval(&null, &vars), Ok(Value::Null));
    }

    #[test]
    fn bool() {
        let mut fal = "false";
        let vars = Map::new().into();
        let fal = expr(&mut fal).unwrap();
        assert_eq!(eval(&fal, &vars), Ok(Value::Bool(false)));
    }

    #[test]
    fn string() {
        let mut s = "\"hello world\"";
        let vars = Map::new().into();
        let s = expr(&mut s).unwrap();
        assert_eq!(eval(&s, &vars), Ok(Value::String("hello world".to_owned())));
    }

    #[test]
    fn number() {
        let mut n = "99";
        let vars = Map::new().into();
        let n = expr(&mut n).unwrap();
        assert_eq!(eval(&n, &vars), Ok(Value::Number(Number::from(99))));
    }

    #[test]
    fn equality() {
        let mut n = "99.0 == 99";
        let vars = Map::new().into();
        let n = expr(&mut n).unwrap();
        assert_eq!(eval(&n, &vars), Ok(Value::Bool(true)));
    }

    #[test]
    fn string_concat() {
        let mut n = "\"hello\" + \" \" + \"world\"";
        let vars = Map::new().into();
        let n = expr(&mut n).unwrap();
        assert_eq!(eval(&n, &vars), Ok(Value::String("hello world".to_owned())));
    }

    #[test]
    fn unary_not() {
        let mut n = "!false";
        let vars = Map::new().into();
        let n = expr(&mut n).unwrap();
        assert_eq!(eval(&n, &vars), Ok(Value::Bool(true)));
    }

    #[test]
    fn array() {
        let mut n = "[!false, !true]";
        let vars = Map::new().into();
        let n = expr(&mut n).unwrap();
        assert_eq!(eval(&n, &vars), Ok(vec![true, false].into()));
    }

    #[test]
    fn conditional() {
        let mut n = "1 == 2 ? [3, 2, 1] : [1,2,3]";
        let vars = Map::new().into();
        let n = expr(&mut n).unwrap();
        assert_eq!(eval(&n, &vars), Ok(vec![1, 2, 3].into()));
    }

    #[test]
    fn array_index() {
        let mut n = "[0,1,2,3,4,5,6,7,8,9][4]";
        let vars = Map::new().into();
        let n = expr(&mut n).unwrap();
        assert_eq!(eval(&n, &vars), Ok(4.into()));
    }

    #[test]
    fn object_index() {
        let mut obj = "{ \"hello\" : \"world\" }[\"hello\"]";
        let vars = Map::new().into();
        let obj = expr(&mut obj).unwrap();
        assert_eq!(eval(&obj, &vars), Ok("world".into()));
    }

    #[test]
    fn indentifier() {
        let vars = json!({ "hello" : "world" });
        let mut id = "hello";
        let id = expr(&mut id).unwrap();
        assert_eq!(eval(&id, &vars), Ok("world".into()));
    }

    #[test]
    fn multi_indentifier() {
        let vars =
            json!({ "and" : { "i": { "think": {"to": {"myself": "what a wonderful world"} } } } });
        let mut id = "and.i.think.to.myself";
        let id = expr(&mut id).unwrap();
        assert_eq!(eval(&id, &vars), Ok("what a wonderful world".into()));
    }

    #[test]
    fn boolean_and() {
        let vars = json!({ "a" : true, "b": false, "score": 101 });
        let mut exp = "a && !b && score == 101";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok(true.into()));
    }

    #[test]
    fn boolean_or() {
        let vars = Map::new().into();
        let mut exp = "false || 0 || \"\" || {} || [] || false";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok(false.into()));
    }

    #[test]
    fn sum_numbers() {
        let vars = Map::new().into();
        let mut exp = "99 + 1 + 100";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok(200.into()));
    }

    #[test]
    fn mul_numbers() {
        let vars = Map::new().into();
        let mut exp = "99 * 1 * 100";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok(9900.into()));
    }

    #[test]
    fn gt_numbers() {
        let vars = json!({ "n" : 99 });
        let mut exp = "n > 1";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok(true.into()));
    }

    #[test]
    fn dot_access() {
        let vars = Map::new().into();
        let mut exp = "{ \"data\": { \"hello\" : \"world\" } }.data.hello";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok("world".into()));
    }

    #[test]
    fn string_index() {
        let vars = Map::new().into();
        let mut exp = "\"abcdefg\"[5]";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok("f".into()));
    }

    #[test]
    fn nested_iff() {
        let vars = Map::new().into();
        let mut exp = "1 ? 0 ? 3 : 4 : 5";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok(4.into()));
    }

    #[test]
    fn nested_iff_complex() {
        let vars = Map::new().into();
        let mut exp = "0 ? 0 : true ? \"here\" : false";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok("here".into()));
    }

    #[test]
    fn weak_type_numbers() {
        let vars = Map::new().into();
        let mut exp = "\"id-\" + 123";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok("id-123".into()));
    }

    #[test]
    fn maths() {
        let vars = Map::new().into();
        let mut exp = "100 + 200 - 99 * 44 / 2";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok((-1878).into()));
    }

    #[test]
    fn modulo() {
        let vars = Map::new().into();
        let mut exp = "101 % 17";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok(16.into()));
    }

    #[test]
    fn comparsons() {
        let vars = Map::new().into();
        let mut exp = "1 == 1 && 2 != 1 && 2 < 3 && 4 > 3 && 3 <= 3 && 4 >= 4 && 4 >= 0 && -1 <= 2";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok(true.into()));
    }

    #[test]
    fn join_array() {
        let vars = Map::new().into();
        let mut exp = "[0, 1, 2] + [3, 4, 5]";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(
            eval(&exp, &vars),
            Ok(Value::Array(vec![
                0.into(),
                1.into(),
                2.into(),
                3.into(),
                4.into(),
                5.into()
            ]))
        );
    }

    #[test]
    fn join_object() {
        let vars = Map::new().into();
        let mut exp = "{ 'a': 0, 'hello': 'world', 'nested': {'object': null, 'this': 'here'} } + { 'a': 2, 'nested': {'object': 1} }";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(
            eval(&exp, &vars),
            Ok(Value::Object(
                vec![
                    ("a".into(), 2.into()),
                    ("hello".into(), "world".into()),
                    (
                        "nested".into(),
                        Value::Object(vec![("object".into(), 1.into())].into_iter().collect())
                    )
                ]
                .into_iter()
                .collect()
            ))
        );
    }

    #[test]
    fn array_length() {
        let vars = Map::new().into();
        let mut exp = "len ( [1, 2, 3] )";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok(3.into()));
    }

    #[test]
    fn obj_length() {
        let vars = Map::new().into();
        let mut exp = "len ( {'a': 2, 'b': { 'c': 1}, 'd': 4} )";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok(3.into()));
    }

    #[test]
    fn str_length() {
        let vars = Map::new().into();
        let mut exp = "len ( 'abc' )";
        let exp = expr(&mut exp).unwrap();
        assert_eq!(eval(&exp, &vars), Ok(3.into()));
    }
}
