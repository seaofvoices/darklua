use std::num::FpCategory;

use crate::process::{LuaValue, TupleValue};

fn operate_on_number<R: Into<LuaValue>, F: Fn(f64) -> R>(
    parameters: TupleValue,
    function: F,
) -> TupleValue {
    if let Some(value) = parameters.into_iter().next() {
        if let LuaValue::Number(value) = value.number_coercion() {
            return function(value).into().into();
        }
    }
    LuaValue::Unknown.into()
}

fn operate_on_two_numbers<F: Fn(f64, f64) -> f64>(
    parameters: TupleValue,
    function: F,
) -> TupleValue {
    let mut iter = parameters.into_iter();
    match (
        iter.next().map(LuaValue::number_coercion),
        iter.next().map(LuaValue::number_coercion),
    ) {
        (Some(LuaValue::Number(first)), Some(LuaValue::Number(second))) => {
            TupleValue::singleton(function(first, second))
        }
        _ => LuaValue::Unknown.into(),
    }
}

fn operate_on_three_numbers<R: Into<LuaValue>, F: Fn(f64, f64, f64) -> R>(
    parameters: TupleValue,
    function: F,
) -> TupleValue {
    let mut iter = parameters.into_iter();
    match (
        iter.next().map(LuaValue::number_coercion),
        iter.next().map(LuaValue::number_coercion),
        iter.next().map(LuaValue::number_coercion),
    ) {
        (
            Some(LuaValue::Number(first)),
            Some(LuaValue::Number(second)),
            Some(LuaValue::Number(third)),
        ) => TupleValue::singleton(function(first, second, third)),
        _ => LuaValue::Unknown.into(),
    }
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

pub fn atan2(parameters: TupleValue) -> TupleValue {
    operate_on_two_numbers(parameters, |y, x| y.atan2(x))
}

pub fn ceil(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.ceil())
}

pub fn clamp(parameters: TupleValue) -> TupleValue {
    operate_on_three_numbers(parameters, |value, min, max| {
        if max < min {
            LuaValue::Unknown
        } else if value < min {
            min.into()
        } else if value > max {
            max.into()
        } else {
            value.into()
        }
    })
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

pub fn fmod(parameters: TupleValue) -> TupleValue {
    operate_on_two_numbers(parameters, |x, y| x - (x / y).trunc() * y)
}

pub fn frexp(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| {
        if 0.0 == value {
            TupleValue::from((value.into(), 0.0.into()))
        } else {
            let lg = value.abs().log2();
            let x = (lg - lg.floor() - 1.0).exp2();
            let exp = lg.floor() + 1.0;
            TupleValue::from(((value.signum() * x).into(), exp.into()))
        }
    })
}

pub fn ldexp(parameters: TupleValue) -> TupleValue {
    operate_on_two_numbers(parameters, |value, exp| value * 2.0_f64.powf(exp))
}

pub fn log(parameters: TupleValue) -> TupleValue {
    if parameters.len() == 1 {
        operate_on_number(parameters, |value| value.log(std::f64::consts::E))
    } else {
        operate_on_two_numbers(parameters, |value, base| value.log(base))
    }
}

pub fn log10(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| value.log10())
}

pub fn max(parameters: TupleValue) -> TupleValue {
    if parameters.is_empty() {
        return LuaValue::Unknown.into();
    }

    let mut iter = parameters
        .into_iter()
        .map(|value| match value.number_coercion() {
            LuaValue::Number(number) => Some(number),
            _ => None,
        });

    if let Some(initial) = iter.next().flatten() {
        iter.try_fold(initial, |max, value| {
            value.map(|current| if current >= max { current } else { max })
        })
        .map(LuaValue::Number)
        .unwrap_or(LuaValue::Unknown)
    } else {
        LuaValue::Unknown
    }
    .into()
}

pub fn min(parameters: TupleValue) -> TupleValue {
    if parameters.is_empty() {
        return LuaValue::Unknown.into();
    }

    let mut iter = parameters
        .into_iter()
        .map(|value| match value.number_coercion() {
            LuaValue::Number(number) => Some(number),
            _ => None,
        });

    if let Some(initial) = iter.next().flatten() {
        iter.try_fold(initial, |max, value| {
            value.map(|current| if current <= max { current } else { max })
        })
        .map(LuaValue::Number)
        .unwrap_or(LuaValue::Unknown)
    } else {
        LuaValue::Unknown
    }
    .into()
}

pub fn modf(parameters: TupleValue) -> TupleValue {
    operate_on_number(parameters, |value| {
        TupleValue::from((value.trunc().into(), value.fract().into()))
    })
}

pub fn pow(parameters: TupleValue) -> TupleValue {
    operate_on_two_numbers(parameters, |num, power| num.powf(power))
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
