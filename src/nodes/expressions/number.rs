use crate::lua_generator::{LuaGenerator, ToLua};

#[derive(Clone, Debug, PartialEq)]
pub struct DecimalNumber {
    float: f64,
    exponent: Option<(i64, bool)>,
}

impl Eq for DecimalNumber {}

impl DecimalNumber {
    pub fn new(value: f64) -> Self {
        Self {
            float: value,
            exponent: None,
        }
    }

    pub fn with_exponent(mut self, exponent: i64, is_uppercase: bool) -> Self {
        self.exponent.replace((exponent, is_uppercase));
        self
    }

    pub fn set_uppercase(&mut self, is_uppercase: bool) {
        self.exponent.map(|(exponent, _)| (exponent, is_uppercase));
    }

    pub fn compute_value(&self) -> f64 {
        if let Some((exponent, _)) = self.exponent {
            self.float * 10_f64.powf(exponent as f64)
        } else {
            self.float
        }
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
    integer: u64,
    exponent: Option<(u32, bool)>,
    is_x_uppercase: bool,
}

impl HexNumber {
    pub fn new(
        integer: u64,
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

    pub fn compute_value(&self) -> f64 {
        if let Some((exponent, _)) = self.exponent {
            (self.integer * 2_u64.pow(exponent)) as f64
        } else {
            self.integer as f64
        }
    }
}

impl ToLua for HexNumber {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        let mut number = format!(
            "0{}{:x}",
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

    pub fn compute_value(&self) -> f64 {
        match self {
            Self::Decimal(decimal) => decimal.compute_value(),
            Self::Hex(hex) => hex.compute_value(),
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

#[cfg(test)]
mod test {
    use super::*;

    mod to_lua {
        use super::*;

        macro_rules! test_to_lua {
            ($($name:ident($input:literal) => $value:expr),*) => {
                $(
                    #[test]
                    fn $name() {
                        let number = NumberExpression::from($input.to_owned());
                        assert_eq!(number.to_lua_string(), $value);
                    }
                )*
            };
        }

        test_to_lua!(
            zero("0") => "0",
            one("1") => "1",
            integer("123") => "123",
            hex_number("0x12") => "0x12",
            hex_number_with_letter("0x12a") => "0x12a",
            hex_with_exponent("0x12p4") => "0x12p4"
        );
    }

    mod compute_value {
        use super::*;

        macro_rules! test_compute_value {
            ($($name:ident($input:literal) => $value:expr),*) => {
                $(
                    #[test]
                    fn $name() {
                        let number = NumberExpression::from($input.to_owned());
                        assert_eq!(number.compute_value(), $value as f64);
                    }
                )*
            };
        }

        test_compute_value!(
            zero("0") => 0,
            one("1") => 1,
            integer("123") => 123,
            multiple_decimal("0.512") => 0.512,
            integer_with_multiple_decimal("54.512") => 54.512,
            digit_with_exponent("1e5") => 1e5,
            number_with_exponent("123e4") => 123e4,
            number_with_negative_exponent("123e-4") => 123e-4,
            float_with_exponent("10.5e2") => 10.5e2,
            hex_number("0x12") => 0x12,
            hex_number_with_letter("0x12a") => 0x12a,
            hex_with_exponent("0x12p4") => 0x120
        );
    }
}
