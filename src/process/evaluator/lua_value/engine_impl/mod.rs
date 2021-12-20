use super::{EngineFunction, LuaValue, TupleValue};

mod lua_globals;
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

pub fn create_tonumber() -> LuaValue {
    EngineFunction::new(lua_globals::tonumber).into()
}

pub fn create_tostring() -> LuaValue {
    EngineFunction::new(lua_globals::tostring).into()
}

pub fn create_type() -> LuaValue {
    EngineFunction::new(lua_globals::lua_type).into()
}

pub fn create_roblox_math_library() -> LuaValue {
    create_library!(
        lua_math, abs, acos, asin, atan, atan2, ceil, clamp, cos, cosh, deg, exp, floor, fmod,
        frexp, ldexp, log, log10, max, min, modf, pow, rad, round, sign, sin, sinh, sqrt, tan,
        tanh,
    )
    .with_entry("huge", f64::INFINITY)
    .with_entry("pi", std::f64::consts::PI)
    // TODO: mark table as with unknown mutations instead of having to provide these callbacks
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

            test_libraries!(
                stringify!($library_name),
                $library_name,
                $library,
                $(
                    $name ($code) => [$( $result ),*] ,
                )*
            );
        };
        ( $global_name:expr, $library_name:ident, $library:expr,
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
                            .with_global_value($global_name, $library);

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
        tonumber,
        create_tonumber(),
        one("return tonumber(1)") => [1.0],
        string_one("return tonumber('1')") => [1.0],
        nil("return tonumber(nil)") => [LuaValue::Nil],
        function("return tonumber(function() end)") => [LuaValue::Nil],
        bool_true("return tonumber(true)") => [LuaValue::Nil],
        bool_false("return tonumber(false)") => [LuaValue::Nil],
        empty_table("return tonumber({})") => [LuaValue::Nil],
        nothing("return tonumber()") => [LuaValue::Unknown],
        unknown_value("return tonumber(variable)") => [LuaValue::Unknown],
    );

    test_libraries!(
        tostring,
        create_tostring(),
        one("return tostring(1)") => ["1"],
        string_one("return tostring('1')") => ["1"],
        string_hello("return tostring('hello')") => ["hello"],
        nil("return tostring(nil)") => ["nil"],
        bool_true("return tostring(true)") => ["true"],
        bool_false("return tostring(false)") => ["false"],
        function("return tostring(function() end)") => [LuaValue::Unknown],
        empty_table("return tostring({})") => [LuaValue::Unknown],
        nothing("return tostring()") => [LuaValue::Unknown],
        unknown_value("return tostring(variable)") => [LuaValue::Unknown],
    );

    test_libraries!(
        "type",
        lua_type,
        create_type(),
        one("return type(1)") => ["number"],
        string_one("return type('1')") => ["string"],
        string_hello("return type('hello')") => ["string"],
        nil("return type(nil)") => ["nil"],
        bool_true("return type(true)") => ["boolean"],
        bool_false("return type(false)") => ["boolean"],
        function("return type(function() end)") => ["function"],
        empty_table("return type({})") => ["table"],
        nothing("return type()") => [LuaValue::Unknown],
        unknown_value("return type(variable)") => [LuaValue::Unknown],
    );

    test_libraries!(
        math,
        create_roblox_math_library(),
        // constants
        pi("return math.pi") => [std::f64::consts::PI],
        huge("return math.huge") => [std::f64::INFINITY],
        // functions
        abs_10("return math.abs(10)") => [10.0],
        abs_negative_11("return math.abs(-11)") => [11.0],
        acos_1("return math.acos(1)") => [0.0],
        asin_1("return math.asin(0)") => [0.0],
        atan_0("return math.atan(0)") => [0.0],
        atan2_0_0("return math.atan2(0, 0)") => [0.0],
        atan2_0_1("return math.atan2(0, 1)") => [0.0],
        ceil_0("return math.ceil(0)") => [0.0],
        ceil_minus_1_5("return math.ceil(-1.5)") => [-1.0],
        ceil_1_5("return math.ceil(1.5)") => [2.0],
        clamp_3_0_1("return math.clamp(3, 0, 1)") => [1.0],
        clamp_3_1_0("return math.clamp(3, 1, 0)") => [LuaValue::Unknown],
        cos("return math.cos(0)") => [1.0],
        cosh("return math.cosh(0)") => [1.0],
        deg_0("return math.deg(0)") => [0.0],
        deg_pi("return math.deg(math.pi)") => [180.0],
        exp_0("return math.exp(0)") => [1.0],
        exp_1("return math.exp(1)") => [std::f64::consts::E],
        floor_0("return math.floor(0)") => [0.0],
        floor_minus_1_5("return math.floor(-1.5)") => [-2.0],
        floor_1_5("return math.floor(1.5)") => [1.0],
        fmod_minus_1_2("return math.fmod(-1, 2)") => [-1.0],
        fmod_1_2("return math.fmod(1, 2)") => [1.0],
        fmod_2_4("return math.fmod(2, 4)") => [2.0],
        fmod_24_5("return math.fmod(24, 5)") => [4.0],
        fmod_24_22("return math.fmod(24, 22)") => [2.0],
        fmod_minus_15_7("return math.fmod(-15, 7)") => [-1.0],
        frexp_0("return math.frexp(0)") => [0.0, 0.0],
        frexp_1("return math.frexp(1)") => [0.5, 1.0],
        frexp_minus_4("return math.frexp(-4)") => [-0.5, 3.0],
        frexp_minus_0_2("return math.frexp(-0.2)") => [-0.8, -2.0],
        ldexp_0_5_1("return math.ldexp(0.5, 1)") => [1.0],
        ldexp_1_1("return math.ldexp(1, 1)") => [2.0],
        ldexp_2_4("return math.ldexp(2, 4)") => [32.0],
        log_e("return math.log(1)") => [0.0],
        log_10_base_10("return math.log(10, 10)") => [1.0],
        log_4_base_16("return math.log(4, 16)") => [0.5],
        log10_1("return math.log10(1)") => [0.0],
        log10_10("return math.log10(10)") => [1.0],
        log10_100("return math.log10(100)") => [2.0],
        max_of_one_number("return math.max(5)") => [5.0],
        max_of_two_numbers("return math.max(10, 4)") => [10.0],
        max_of_three_numbers("return math.max(-5, 4.5, 4)") => [4.5],
        min_of_one_number("return math.min(5)") => [5.0],
        min_of_two_numbers("return math.min(10, 4)") => [4.0],
        min_of_three_numbers("return math.min(-5, 4.5, 4)") => [-5.0],
        modf_0("return math.modf(0)") => [0.0, 0.0],
        modf_0_33("return math.modf(0.33)") => [0.0, 0.33],
        modf_1("return math.modf(1)") => [1.0, 0.0],
        modf_10_5("return math.modf(10.5)") => [10.0, 0.5],
        pow_3_2("return math.pow(3, 2)") => [9.0],
        rad_0("return math.rad(0)") => [0.0],
        rad_360("return math.rad(360)") => [2.0 * std::f64::consts::PI],
        round_0_5("return math.round(0.5)") => [1.0],
        round_minus_0_5("return math.round(-0.5)") => [-1.0],
        round_minus_5_8("return math.round(5.8)") => [6.0],
        sign_zero("return math.sign(0)") => [0.0],
        sign_12_5("return math.sign(12.5)") => [1.0],
        sign_0_0001("return math.sign(-0.0001)") => [-1.0],
        sin("return math.sin(0)") => [0.0],
        sinh("return math.sinh(0)") => [0.0],
        sqrt_4("return math.sqrt(4)") => [2.0],
        sqrt_16("return math.sqrt(16)") => [4.0],
        tan("return math.tan(0)") => [0.0],
        tanh("return math.tanh(0)") => [0.0],
    );
}
