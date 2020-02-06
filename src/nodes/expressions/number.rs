use crate::lua_generator::{LuaGenerator, ToLua};

#[derive(Clone, Debug, PartialEq)]
pub struct DecimalNumber {
    float: f64,
    exponent: Option<(u32, bool)>,
}

impl Eq for DecimalNumber {}

impl DecimalNumber {
    pub fn new(value: f64) -> Self {
        Self {
            float: value,
            exponent: None,
        }
    }

    pub fn with_exponent(mut self, exponent: u32, is_uppercase: bool) -> Self {
        self.exponent.replace((exponent, is_uppercase));
        self
    }

    pub fn set_uppercase(&mut self, is_uppercase: bool) {
        self.exponent.map(|(exponent, _)| (exponent, is_uppercase));
    }
}

impl ToLua for DecimalNumber {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        let mut number = format!("{}", self.float);

        if let Some((exponent, is_uppercase)) = &self.exponent {
            number.push(if *is_uppercase { 'E' } else { 'e' });
            number.push_str(&format!("{}", exponent));
        };

        generator.push_str(&number);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HexNumber {
    integer: u32,
    exponent: Option<(u32, bool)>,
    is_x_uppercase: bool,
}

impl HexNumber {
    pub fn new(
        integer: u32,
        is_x_uppercase: bool,
    ) -> Self {
        Self {
            integer,
            exponent: None,
            is_x_uppercase,
        }
    }

    pub fn with_exponent(mut self, exponent: u32, is_uppercase: bool) -> Self {
        self.exponent.replace((exponent, is_uppercase));
        self
    }

    pub fn set_uppercase(&mut self, is_uppercase: bool) {
        self.exponent.map(|(expression, _)| (expression, is_uppercase));
        self.is_x_uppercase = is_uppercase;
    }
}

impl ToLua for HexNumber {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        let mut number = format!(
            "0{}{}",
            if self.is_x_uppercase { 'X' } else { 'x' },
            self.integer
        );

        if let Some((exponent, is_uppercase)) = &self.exponent {
            number.push(if *is_uppercase { 'P' } else { 'p' });
            number.push_str(&format!("{}", exponent));
        };

        generator.push_str(&number);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NumberExpression {
    Decimal(DecimalNumber),
    Hex(HexNumber),
}

impl NumberExpression {
    pub fn set_uppercase(&mut self, is_uppercase: bool) {
        match self {
            Self::Decimal(number) => number.set_uppercase(is_uppercase),
            Self::Hex(number) => number.set_uppercase(is_uppercase),
        }
    }
}

impl From<DecimalNumber> for NumberExpression {
    fn from(number: DecimalNumber) -> Self {
        Self::Decimal(number)
    }
}

impl From<HexNumber> for NumberExpression {
    fn from(number: HexNumber) -> Self {
        Self::Hex(number)
    }
}

impl ToLua for NumberExpression {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        match self {
            Self::Decimal(value) => value.to_lua(generator),
            Self::Hex(value) => value.to_lua(generator),
        }
    }
}
