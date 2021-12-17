use super::{EngineFunction, LuaValue, TupleValue};

mod lua_math;

fn unimplemented_callback(_parameters: TupleValue) -> TupleValue {
    LuaValue::Unknown.into()
}

macro_rules! create_library {
    ($module:ident, $( $function:ident ),* $(,)? ) => {
        super::TableValue::default()
            $(
                .with_entry(
                    stringify!($function),
                    EngineFunction::new($module::$function),
                )
            )*
    };
}

pub fn create_roblox_math_library() -> LuaValue {
    create_library!(
        lua_math, abs, acos, asin, atan, atan2, ceil, clamp, cos, cosh, deg, exp, floor, fmod,
        frexp, log, log10, max, min, modf, pow, rad, round, sign, sin, sinh, sqrt, tan, tanh,
    )
    .with_entry("huge", f64::INFINITY)
    .with_entry("pi", std::f64::consts::PI)
    .with_entry("noise", EngineFunction::new(unimplemented_callback))
    .with_entry("random", EngineFunction::new(unimplemented_callback))
    .with_entry("randomseed", EngineFunction::new(unimplemented_callback))
    .into()
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_libraries {
        ($library_name:ident, $library:expr,
            $($name:ident ($code:literal) => [$( $result:expr ),*] ),* $(,)?) => {

            mod $library_name {
                use super::*;
                $(
                    #[test]
                    fn $name() {
                        let mut block = crate::Parser::default()
                            .parse($code)
                            .expect("code should parse");

                        let mut state = crate::process::VirtualLuaExecution::default()
                            .with_global_value(stringify!($library_name), $library);

                        pretty_assertions::assert_eq!(
                            state.evaluate_chunk(&mut block),
                            crate::process::TupleValue::new(vec![$( LuaValue::from($result), )*])
                        );
                    }
                )*
            }
        };
    }

    test_libraries!(
        math,
        create_roblox_math_library(),
        abs_10("return math.abs(10)") => [10.0],
        abs_negative_11("return math.abs(-11)") => [11.0],
        sin("return math.sin(0)") => [0.0],
        cos("return math.cos(0)") => [1.0],
        sign_zero("return math.sign(0)") => [0.0],
        sign_12_5("return math.sign(12.5)") => [1.0],
        sign_0_0001("return math.sign(-0.0001)") => [-1.0],
        log10_10("return math.log10(10)") => [1.0],
    );
}