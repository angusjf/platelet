use crate::expression_parser::{self, Expression};

pub(crate) fn eval(exp: &Expression) -> Expression {
    match exp {
        Expression::Indexed(_) => todo!(),
        Expression::BinaryOperation(_) => todo!(),
        Expression::FunctionCall(_) => todo!(),
        Expression::UnaryOperation(_) => todo!(),
        Expression::Conditional(_) => todo!(),
        Expression::Length(_) => todo!(),
        Expression::Expression(_) => todo!(),
        Expression::Null => todo!(),
        Expression::Boolean(_) => todo!(),
        Expression::Str(_) => todo!(),
        Expression::Num(_) => todo!(),
        Expression::Array(_) => todo!(),
        Expression::Object(_) => todo!(),
        Expression::MultiIdentifier(_) => todo!(),
    }
}

#[cfg(test)]
mod test {
    #[allow(clippy::useless_attribute)]
    #[allow(dead_code)] // its dead for benches
    use super::*;

    #[test]
    fn null() {
        let mut null = "null";
        let null = expression_parser::expr(&mut null).unwrap();
        assert_eq!(eval(&null), null);
    }
}
