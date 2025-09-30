use crate::process::{native_functions, TupleValue};

macro_rules! define_native_functions {
    ($( $name:ident => $path:path ),+ $(,)?) => {
        impl NativeFunction {
            $(
                pub(crate) fn $name() -> Self {
                    NativeFunction {
                        native_function_id: stringify!($name),
                        function: $path,
                    }
                }
            )+
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionValue {
    inner: InternalFunction,
}

impl FunctionValue {
    pub(crate) fn new_lua() -> Self {
        Self {
            inner: InternalFunction::Lua,
        }
    }

    pub(crate) fn call(&self, args: TupleValue) -> TupleValue {
        match &self.inner {
            InternalFunction::Native(function) => function.call(args),
            _ => TupleValue::unknown(),
        }
    }

    pub(crate) fn has_side_effects(&self) -> bool {
        match &self.inner {
            InternalFunction::Native(_) => false,
            _ => true,
        }
    }
}

impl From<NativeFunction> for FunctionValue {
    fn from(v: NativeFunction) -> Self {
        Self {
            inner: InternalFunction::Native(v),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InternalFunction {
    Native(NativeFunction),
    Lua,
}

#[derive(Debug, Clone)]
pub(crate) struct NativeFunction {
    native_function_id: &'static str,
    function: fn(args: TupleValue) -> TupleValue,
}

impl NativeFunction {
    pub(crate) fn call(&self, args: TupleValue) -> TupleValue {
        (self.function)(args)
    }
}

define_native_functions!(
    math_abs => native_functions::math::abs,
    math_cos => native_functions::math::cos,
    math_sin => native_functions::math::sin,
    math_tan => native_functions::math::tan,
    math_sqrt => native_functions::math::sqrt,
    math_pow => native_functions::math::pow,
    math_exp => native_functions::math::exp,
    math_rad => native_functions::math::rad,
    math_deg => native_functions::math::deg,
    math_sign => native_functions::math::sign,
);

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.native_function_id == other.native_function_id
    }
}

impl Eq for NativeFunction {}
