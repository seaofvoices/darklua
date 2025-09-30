use crate::process::{LuaValue, TupleValue};

fn mono_calculation(values: TupleValue, operation: fn(f64) -> f64) -> TupleValue {
    if let LuaValue::Number(number) = values.into_one().number_coercion() {
        LuaValue::Number(operation(number)).into()
    } else {
        TupleValue::unknown()
    }
}

pub(crate) fn abs(values: TupleValue) -> TupleValue {
    mono_calculation(values, |number| number.abs())
}

pub(crate) fn sin(values: TupleValue) -> TupleValue {
    mono_calculation(values, |number| number.sin())
}

pub(crate) fn cos(values: TupleValue) -> TupleValue {
    mono_calculation(values, |number| number.cos())
}

pub(crate) fn tan(values: TupleValue) -> TupleValue {
    mono_calculation(values, |number| number.tan())
}

pub(crate) fn sqrt(values: TupleValue) -> TupleValue {
    mono_calculation(values, |number| number.sqrt())
}

pub(crate) fn pow(values: TupleValue) -> TupleValue {
    let mut values = values.into_values();
    let base = values.next().unwrap_or_default().number_coercion();
    let exponent = values.next().unwrap_or_default().number_coercion();
    if let (LuaValue::Number(base), LuaValue::Number(exponent)) = (base, exponent) {
        LuaValue::Number(base.powf(exponent)).into()
    } else {
        TupleValue::unknown()
    }
}

pub(crate) fn exp(values: TupleValue) -> TupleValue {
    mono_calculation(values, |number| number.exp())
}

pub(crate) fn rad(values: TupleValue) -> TupleValue {
    mono_calculation(values, |number| number.to_radians())
}

pub(crate) fn deg(values: TupleValue) -> TupleValue {
    mono_calculation(values, |number| number.to_degrees())
}

pub(crate) fn sign(values: TupleValue) -> TupleValue {
    mono_calculation(values, |number| {
        if number > 0.0 {
            1.0
        } else if number < 0.0 {
            -1.0
        } else {
            0.0
        }
    })
}
