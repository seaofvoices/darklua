use crate::process::{LuaValue, TupleValue};

pub fn tonumber(parameters: TupleValue) -> TupleValue {
    if parameters.is_empty() {
        return LuaValue::Unknown.into();
    }

    match parameters.coerce_to_single_value().number_coercion() {
        LuaValue::Number(number) => LuaValue::Number(number),
        LuaValue::String(_)
        | LuaValue::Table(_)
        | LuaValue::TableRef(_)
        | LuaValue::Function
        | LuaValue::Function2(_)
        | LuaValue::Nil
        | LuaValue::False
        | LuaValue::True => LuaValue::Nil,
        LuaValue::Tuple(_) | LuaValue::Unknown => LuaValue::Unknown,
    }
    .into()
}

pub fn tostring(parameters: TupleValue) -> TupleValue {
    if parameters.is_empty() {
        return LuaValue::Unknown.into();
    }

    match parameters.coerce_to_single_value().string_coercion() {
        LuaValue::String(string) => LuaValue::String(string),
        LuaValue::False => LuaValue::from("false"),
        LuaValue::Nil => LuaValue::from("nil"),
        LuaValue::True => LuaValue::from("true"),
        LuaValue::Function
        | LuaValue::Function2(_)
        | LuaValue::Table(_)
        | LuaValue::TableRef(_)
        | LuaValue::Number(_)
        | LuaValue::Tuple(_)
        | LuaValue::Unknown => LuaValue::Unknown,
    }
    .into()
}

pub fn lua_type(parameters: TupleValue) -> TupleValue {
    if parameters.is_empty() {
        return LuaValue::Unknown.into();
    }

    LuaValue::from(match parameters.coerce_to_single_value() {
        LuaValue::True | LuaValue::False => "boolean",
        LuaValue::Function | LuaValue::Function2(_) => "function",
        LuaValue::Nil => "nil",
        LuaValue::Number(_) => "number",
        LuaValue::String(_) => "string",
        LuaValue::Table(_) | LuaValue::TableRef(_) => "table",
        LuaValue::Tuple(_) | LuaValue::Unknown => return LuaValue::Unknown.into(),
    })
    .into()
}
