use crate::process::{LuaValue, TupleValue};

const MAX_STRING_SIZE: usize = 100_000;

pub(crate) fn len(values: TupleValue) -> TupleValue {
    if let LuaValue::String(string_value) = values.into_one().string_coercion() {
        LuaValue::Number(string_value.len() as f64).into()
    } else {
        TupleValue::unknown()
    }
}

pub(crate) fn lower(values: TupleValue) -> TupleValue {
    mono_string_transform(values, |string_value| {
        string_value
            .into_iter()
            .map(|byte| match byte {
                b'A'..=b'Z' => byte + 32,
                _ => byte,
            })
            .collect()
    })
}

pub(crate) fn rep(values: TupleValue) -> TupleValue {
    let mut iterator = values.into_values();

    let string_value = iterator.next().unwrap_or_default().string_coercion();
    let replacement = iterator.next().unwrap_or_default().number_coercion();

    if let (LuaValue::String(string_value), LuaValue::Number(replacement)) =
        (string_value, replacement)
    {
        let replacement = if replacement.is_nan() || replacement < 0.0 {
            0
        } else {
            replacement.floor() as usize
        };
        let expected_length = string_value.len().saturating_mul(replacement);

        if expected_length >= MAX_STRING_SIZE {
            return TupleValue::unknown();
        }

        LuaValue::String(string_value.repeat(replacement)).into()
    } else {
        TupleValue::unknown()
    }
}

pub(crate) fn reverse(values: TupleValue) -> TupleValue {
    mono_string_transform(values, |string_value| {
        string_value.into_iter().rev().collect()
    })
}

pub(crate) fn upper(values: TupleValue) -> TupleValue {
    mono_string_transform(values, |string_value| {
        string_value
            .into_iter()
            .map(|byte| match byte {
                b'a'..=b'z' => byte - 32,
                _ => byte,
            })
            .collect()
    })
}

fn mono_string_transform(values: TupleValue, transform: fn(Vec<u8>) -> Vec<u8>) -> TupleValue {
    if let LuaValue::String(string_value) = values.into_one().string_coercion() {
        LuaValue::String(transform(string_value)).into()
    } else {
        TupleValue::unknown()
    }
}
