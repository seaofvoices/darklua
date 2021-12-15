use std::num::FpCategory;

use crate::process::{LuaValue, TupleValue};

fn operate_on_number<F: Fn(f64) -> f64>(parameters: TupleValue, function: F) -> TupleValue {
    if let Some(value) = parameters.into_iter().next() {
        if let LuaValue::Number(value) = value.number_coercion() {
            return TupleValue::singleton(function(value));
        }
    }
    LuaValue::Unknown.into()
}

pub fn abs(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.abs())
}

pub fn acos(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.acos())
}

pub fn asin(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.asin())
}

pub fn atan(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.atan())
}

pub fn atan2(_parameters: TupleValue) -> TupleValue {
    // TODO
    LuaValue::Unknown.into()
}

pub fn ceil(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.ceil())
}

pub fn clamp(_parameters: TupleValue) -> TupleValue {
    // TODO
    LuaValue::Unknown.into()
}

pub fn cos(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.cos())
}

pub fn cosh(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.cosh())
}

pub fn deg(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.to_degrees())
}

pub fn exp(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.exp())
}

pub fn floor(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.floor())
}

pub fn fmod(_parameters: TupleValue) -> TupleValue {
    // TODO
    LuaValue::Unknown.into()
}

pub fn frexp(_parameters: TupleValue) -> TupleValue {
    // TODO
    // vec![LuaValue::Unknown, LuaValue::Unknown].into()

    LuaValue::Unknown.into()
}

pub fn log(_parameters: TupleValue) -> TupleValue {
    // TODO
    LuaValue::Unknown.into()
}

pub fn log10(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.log10())
}

pub fn max(_parameters: TupleValue) -> TupleValue {
    // TODO
    LuaValue::Unknown.into()
}

pub fn min(_parameters: TupleValue) -> TupleValue {
    // TODO
    LuaValue::Unknown.into()
}

pub fn modf(_parameters: TupleValue) -> TupleValue {
    // TODO
    LuaValue::Unknown.into()
}

pub fn pow(_parameters: TupleValue) -> TupleValue {
    // TODO
    LuaValue::Unknown.into()
}

pub fn rad(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.to_radians())
}

pub fn round(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.round())
}

pub fn sign(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| match value.classify() {
        FpCategory::Nan => 0.0,
        FpCategory::Zero => 0.0,
        FpCategory::Subnormal | FpCategory::Normal | FpCategory::Infinite => {
            if value > 0.0 {
                1.0
            } else {
                -1.0
            }
        }
    })
}

pub fn sin(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.sin())
}

pub fn sinh(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.sinh())
}

pub fn sqrt(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.sqrt())
}

pub fn tan(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.tan())
}

pub fn tanh(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.tanh())
}

#[cfg(test)]
mod test {
    use super::*;

    mod sign {
        use super::*;

        #[test]
        fn nan_returns_zero() {
            assert_eq!(
                sign(TupleValue::singleton(f64::NAN)),
                TupleValue::singleton(0.0)
            )
        }

        #[test]
        fn zero_plus_returns_zero() {
            assert_eq!(
                sign(TupleValue::singleton(1.0 / f64::INFINITY)),
                TupleValue::singleton(0.0)
            )
        }

        #[test]
        fn negative_zero_returns_zero() {
            assert_eq!(
                sign(TupleValue::singleton(-1.0 / f64::INFINITY)),
                TupleValue::singleton(0.0)
            )
        }
    }
}
