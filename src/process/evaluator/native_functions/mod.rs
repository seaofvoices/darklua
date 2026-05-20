mod lua_math;
mod lua_string;

pub(crate) mod math {
    pub(crate) use super::lua_math::*;
}

pub(crate) mod string {
    pub(crate) use super::lua_string::*;
}
