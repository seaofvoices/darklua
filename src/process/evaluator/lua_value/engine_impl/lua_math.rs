use std::num::FpCategory;

use crate::process::LuaValue;

fn operate_on_number<F: Fn(f64) -> f64>(parameters: Vec<LuaValue>, function: F) -> Vec<LuaValue> {
    if let Some(value) = parameters.into_iter().next() {
        if let LuaValue::Number(value) = value.number_coercion() {
            return vec![function(value).into()];
        }
    }
    vec![LuaValue::Unknown]
}

pub fn abs(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.abs())
}

pub fn acos(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.acos())
}

pub fn asin(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.asin())
}

pub fn atan(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.atan())
}

pub fn atan2(_parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    // TODO
    vec![LuaValue::Unknown]
}

pub fn ceil(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.ceil())
}

pub fn clamp(_parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    // TODO
    vec![LuaValue::Unknown]
}

pub fn cos(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.cos())
}

pub fn cosh(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.cosh())
}

pub fn deg(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.to_degrees())
}

pub fn exp(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.exp())
}

pub fn floor(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.floor())
}

pub fn fmod(_parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    // TODO
    vec![LuaValue::Unknown]
}

pub fn frexp(_parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    // TODO
    vec![LuaValue::Unknown, LuaValue::Unknown]
}

pub fn log(_parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    // TODO
    vec![LuaValue::Unknown]
}

pub fn log10(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.log10())
}

pub fn max(_parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    // TODO
    vec![LuaValue::Unknown]
}

pub fn min(_parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    // TODO
    vec![LuaValue::Unknown]
}

pub fn modf(_parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    // TODO
    vec![LuaValue::Unknown]
}

pub fn pow(_parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    // TODO
    vec![LuaValue::Unknown]
}

pub fn rad(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.to_radians())
}

pub fn round(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.round())
}

pub fn sign(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
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

pub fn sin(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.sin())
}

pub fn sinh(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.sinh())
}

pub fn sqrt(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.sqrt())
}

pub fn tan(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.tan())
}

pub fn tanh(parameters: Vec<LuaValue>) -> Vec<LuaValue> {
    operate_on_number(parameters, |value| value.tanh())
}

#[cfg(test)]
mod test {
    use super::*;

    mod sign {
        use super::*;

        #[test]
        fn nan_returns_zero() {
            assert_eq!(sign(vec![f64::NAN.into()]), vec![0.0.into()])
        }

        #[test]
        fn zero_plus_returns_zero() {
            assert_eq!(sign(vec![(1.0 / f64::INFINITY).into()]), vec![0.0.into()])
        }

        #[test]
        fn negative_zero_returns_zero() {
            assert_eq!(sign(vec![(-1.0 / f64::INFINITY).into()]), vec![0.0.into()])
        }
    }
}
