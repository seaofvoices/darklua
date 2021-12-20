use crate::process::{LuaValue, TupleValue};

const MAX_STRING_REPETITION: f64 = 1024.0;

#[inline]
fn nan_to_zero(number: f64) -> f64 {
    if number.is_nan() {
        0.0
    } else {
        number
    }
}

fn operate_on_string<R: Into<LuaValue>, F: Fn(String) -> R>(
    parameters: TupleValue,
    function: F,
) -> TupleValue {
    if let Some(value) = parameters.into_iter().next() {
        if let LuaValue::String(value) = value.string_coercion() {
            return function(value).into().into();
        }
    }

    LuaValue::Unknown.into()
}

pub fn len(parameters: TupleValue) -> TupleValue {
    operate_on_string(parameters, |value| value.len() as f64)
}

pub fn lower(parameters: TupleValue) -> TupleValue {
    operate_on_string(parameters, |value| value.to_lowercase())
}

pub fn rep(parameters: TupleValue) -> TupleValue {
    let mut iter = parameters.into_iter();
    match (
        iter.next().map(LuaValue::string_coercion),
        iter.next().map(LuaValue::number_coercion),
    ) {
        (Some(LuaValue::String(string)), Some(LuaValue::Number(number))) => {
            if number > MAX_STRING_REPETITION {
                LuaValue::Unknown.into()
            } else {
                let repetition = nan_to_zero(number)
                    .clamp(0.0, MAX_STRING_REPETITION)
                    .trunc() as usize;
                LuaValue::from(string.repeat(repetition)).into()
            }
        }
        _ => LuaValue::Unknown.into(),
    }
}

pub fn reverse(parameters: TupleValue) -> TupleValue {
    operate_on_string(parameters, |value| value.chars().rev().collect::<String>())
}

pub fn upper(parameters: TupleValue) -> TupleValue {
    operate_on_string(parameters, |value| value.to_uppercase())
}
