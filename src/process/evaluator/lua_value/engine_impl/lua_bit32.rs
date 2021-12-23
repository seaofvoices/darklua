use crate::process::{LuaValue, TupleValue};

const MAX_U32: f64 = 4294967296.0;

fn to_int_32(number: f64) -> u32 {
    let integer = number.trunc();
    (integer - MAX_U32 * f64::floor(integer / MAX_U32)) as u32
}

pub fn _arcshift(_parameters: TupleValue) -> TupleValue {
    LuaValue::Unknown.into()
}

pub fn band(parameters: TupleValue) -> TupleValue {
    let mut result = u32::MAX;
    for value in parameters {
        if let LuaValue::Number(number) = value.number_coercion() {
            result &= to_int_32(number);
        } else {
            return LuaValue::Unknown.into();
        }
    }
    LuaValue::from(result as f64).into()
}

pub fn _bnot(_parameters: TupleValue) -> TupleValue {
    LuaValue::Unknown.into()
}

pub fn _bor(_parameters: TupleValue) -> TupleValue {
    LuaValue::Unknown.into()
}

pub fn _btest(_parameters: TupleValue) -> TupleValue {
    LuaValue::Unknown.into()
}

pub fn _bxor(_parameters: TupleValue) -> TupleValue {
    LuaValue::Unknown.into()
}

pub fn _extract(_parameters: TupleValue) -> TupleValue {
    LuaValue::Unknown.into()
}

pub fn _replace(_parameters: TupleValue) -> TupleValue {
    LuaValue::Unknown.into()
}

pub fn _lrotate(_parameters: TupleValue) -> TupleValue {
    LuaValue::Unknown.into()
}

pub fn _lshift(_parameters: TupleValue) -> TupleValue {
    LuaValue::Unknown.into()
}

pub fn _rrotate(_parameters: TupleValue) -> TupleValue {
    LuaValue::Unknown.into()
}

pub fn _rshift(_parameters: TupleValue) -> TupleValue {
    LuaValue::Unknown.into()
}
