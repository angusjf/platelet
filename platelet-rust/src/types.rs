use serde_json::Value;

#[derive(Debug, PartialEq)]
pub(crate) enum Type {
    String,
    Number,
    Object,
    Array,
    Bool,
    Null,
}

impl Type {
    pub(crate) fn to_string(self: &Self) -> &'static str {
        match self {
            Type::String => "string",
            Type::Number => "number",
            Type::Object => "object",
            Type::Array => "array",
            Type::Bool => "bool",
            Type::Null => "null",
        }
    }
}

pub(crate) fn type_of(val: &Value) -> Type {
    match val {
        Value::Null => Type::Null,
        Value::Bool(_) => Type::Bool,
        Value::Number(_) => Type::Number,
        Value::String(_) => Type::String,
        Value::Array(_) => Type::Array,
        Value::Object(_) => Type::Object,
    }
}
