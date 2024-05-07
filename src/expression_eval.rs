use serde_json::{Number, Value};

use crate::expression_parser::{BinaryOperator, Expression, UnaryOperator};

#[derive(Debug, PartialEq)]
pub(crate) enum EvalError {
    TypeMismatch,
}

pub(crate) fn eval(exp: &Expression, vars: &Value) -> Result<Value, EvalError> {
    match exp {
        Expression::Indexed(_) => todo!(),
        Expression::BinaryOperation(bin_op) => {
            let (a, op, b) = bin_op.as_ref();
            let a = eval(a, vars)?;
            let b = eval(b, vars)?;
            match op {
                BinaryOperator::Add => match (a, b) {
                    (Value::Number(n), Value::Number(m)) => {
                        todo!()
                    }
                    (Value::Number(n), Value::String(s)) => todo!(),
                    (Value::String(s), Value::Number(n)) => todo!(),
                    (Value::String(n), Value::String(m)) => Ok(Value::String(n + &m)),
                    _ => Err(EvalError::TypeMismatch),
                },
                BinaryOperator::Subtract => todo!(),
                BinaryOperator::Multiply => todo!(),
                BinaryOperator::Divide => todo!(),
                BinaryOperator::Modulo => todo!(),
                BinaryOperator::EqualTo => Ok(Value::Bool(a == b)),
                BinaryOperator::NotEqualTo => todo!(),
                BinaryOperator::GreaterThan => todo!(),
                BinaryOperator::GreterThanOrEqualTo => todo!(),
                BinaryOperator::LessThan => todo!(),
                BinaryOperator::LessThanOrEqualTo => todo!(),
                BinaryOperator::Or => todo!(),
                BinaryOperator::And => todo!(),
            }
        }
        Expression::FunctionCall(_) => todo!(),
        Expression::UnaryOperation(un_op) => {
            let (UnaryOperator::Not, exp) = un_op.as_ref();
            let exp = eval(exp, vars)?;
            Ok(Value::Bool(!as_bool(&exp)))
        }
        Expression::Conditional(cond_exp) => {
            let (cond, tru, fal) = cond_exp.as_ref();
            let cond = eval(cond, vars)?;
            match cond {
                Value::Bool(b) => eval(if b { tru } else { fal }, vars),
                _ => Err(EvalError::TypeMismatch),
            }
        }
        Expression::Null => Ok(Value::Null),
        Expression::Boolean(v) => Ok(Value::Bool(*v)),
        Expression::Str(s) => Ok(Value::String(s.clone())),
        Expression::Num(n) => Ok(Value::Number(Number::from_f64(*n).unwrap())),
        Expression::Array(a) => Ok(a.iter().map(|e| eval(e, vars)).collect::<Result<_, _>>()?),
        Expression::Object(_) => todo!(),
        Expression::MultiIdentifier(_) => todo!(),
    }
}

fn as_bool(v: &Value) -> bool {
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
    use serde_json::Map;

    use crate::expression_parser::{self, expr};

    #[allow(clippy::useless_attribute)]
    #[allow(dead_code)] // its dead for benches
    use super::*;

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
        assert_eq!(
            eval(&n, &vars),
            Ok(Value::Number(Number::from_f64(99.0).unwrap()))
        );
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
        assert_eq!(eval(&n, &vars), Ok(vec![1.0, 2.0, 3.0].into()));
    }
}
