use crate::lua_generator::{LuaGenerator, ToLua};

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

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
        if self.float.is_nan() {
            generator.push_char('(');
            generator.push_char('0');
            generator.push_char('/');
            generator.push_char('0');
            generator.push_char(')');
        } else if self.float.is_infinite() {
            generator.push_char('(');
            if self.float.is_sign_negative() {
                generator.push_char('-');
            }
            generator.push_char('1');
            generator.push_char('/');
            generator.push_char('0');
            generator.push_char(')');
        } else {
            let mut number = format!("{:.}", self.float);

            if let Some((exponent, is_uppercase)) = &self.exponent {
                number.push(if *is_uppercase { 'E' } else { 'e' });
                number.push_str(&format!("{}", exponent));
            };

            generator.push_str(&number);
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberParsingError {
    InvalidHexadecimalNumber,
    InvalidHexadecimalExponent,
    InvalidDecimalNumber,
    InvalidDecimalExponent,
}

impl Display for NumberParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use NumberParsingError::*;

        match self {
            InvalidHexadecimalNumber => write!(f, "could not parse hexadecimal number"),
            InvalidHexadecimalExponent => write!(f, "could not parse hexadecimal exponent"),
            InvalidDecimalNumber => write!(f, "could not parse decimal number"),
            InvalidDecimalExponent => write!(f, "could not parse decimal exponent"),
        }
    }
}

impl FromStr for NumberExpression {
    type Err = NumberParsingError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let number = if value.starts_with("0x") || value.starts_with("0X") {
            let is_x_uppercase = value.chars().nth(1)
                .map(char::is_uppercase)
                .unwrap_or(false);

            if let Some(index) = value.find("p") {
                let exponent = value.get(index + 1..)
                    .and_then(|string| string.parse().ok())
                    .ok_or(Self::Err::InvalidHexadecimalExponent)?;
                let number = u64::from_str_radix(value.get(2..index).unwrap(), 16)
                    .map_err(|_| Self::Err::InvalidHexadecimalNumber)?;

                HexNumber::new(number, is_x_uppercase)
                    .with_exponent(exponent, false)

            } else if let Some(index) = value.find("P") {
                let exponent = value.get(index + 1..)
                    .and_then(|string| string.parse().ok())
                    .ok_or(Self::Err::InvalidHexadecimalExponent)?;
                let number = u64::from_str_radix(value.get(2..index).unwrap(), 16)
                    .map_err(|_| Self::Err::InvalidHexadecimalNumber)?;

                HexNumber::new(number, is_x_uppercase)
                    .with_exponent(exponent, true)
            } else {
                let number = u64::from_str_radix(value.get(2..)
                    .unwrap(), 16)
                    .map_err(|_| Self::Err::InvalidHexadecimalNumber)?;

                HexNumber::new(number, is_x_uppercase)
            }.into()

        } else {
            if let Some(index) = value.find("e") {
                let exponent = value.get(index + 1..)
                    .and_then(|string| string.parse().ok())
                    .ok_or(Self::Err::InvalidDecimalExponent)?;
                let number = value.get(0..index)
                    .and_then(|string| string.parse().ok())
                    .ok_or(Self::Err::InvalidDecimalNumber)?;

                DecimalNumber::new(number)
                    .with_exponent(exponent, false)

            } else if let Some(index) = value.find("E") {
                let exponent: i64 = value.get(index + 1..)
                    .and_then(|string| string.parse().ok())
                    .ok_or(Self::Err::InvalidDecimalExponent)?;
                let number = value.get(0..index)
                    .and_then(|string| string.parse().ok())
                    .ok_or(Self::Err::InvalidDecimalNumber)?;

                DecimalNumber::new(number)
                    .with_exponent(exponent, true)
            } else {
                let number = value.parse::<f64>()
                    .map_err(|_| Self::Err::InvalidDecimalNumber)?;

                DecimalNumber::new(number)
            }.into()
        };

        Ok(number)
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
                        let number = NumberExpression::from($input);
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

    mod parse_number {
        use super::*;

        macro_rules! test_numbers {
            ($($name:ident($input:literal) => $expect:expr),+) => {
                $(
                    #[test]
                    fn $name() {
                        let result: NumberExpression = $input.parse()
                            .expect("should be a valid number");

                        let expect: NumberExpression = $expect.into();

                        assert_eq!(result, expect);
                    }
                )+
            };
        }

        macro_rules! test_parse_errors {
            ($($name:ident($input:literal) => $expect:expr),+) => {
                $(
                    #[test]
                    fn $name() {
                        let result = $input.parse::<NumberExpression>()
                            .expect_err("should be an invalid number");

                        assert_eq!(result, $expect);
                    }
                )+
            };
        }

        test_numbers!(
            parse_zero("0") => DecimalNumber::new(0_f64),
            parse_integer("123") => DecimalNumber::new(123_f64),
            parse_multiple_decimal("123.24") => DecimalNumber::new(123.24_f64),
            parse_float_with_trailing_dot("123.") => DecimalNumber::new(123_f64),
            parse_starting_with_dot(".123") => DecimalNumber::new(0.123_f64),
            parse_digit_with_exponent("1e10") => DecimalNumber::new(1_f64).with_exponent(10, false),
            parse_number_with_exponent("123e456") => DecimalNumber::new(123_f64).with_exponent(456, false),
            parse_number_with_exponent_and_plus_symbol("123e+456") => DecimalNumber::new(123_f64).with_exponent(456, false),
            parse_number_with_negative_exponent("123e-456") => DecimalNumber::new(123_f64).with_exponent(-456, false),
            parse_number_with_upper_exponent("123E4") => DecimalNumber::new(123_f64).with_exponent(4, true),
            parse_number_with_upper_negative_exponent("123E-456") => DecimalNumber::new(123_f64).with_exponent(-456, true),
            parse_float_with_exponent("10.12e8") => DecimalNumber::new(10.12_f64).with_exponent(8, false),
            parse_trailing_dot_with_exponent("10.e8") => DecimalNumber::new(10_f64).with_exponent(8, false),
            parse_hex_number("0x12") => HexNumber::new(18, false),
            parse_uppercase_hex_number("0X12") => HexNumber::new(18, true),
            parse_hex_number_with_lowercase("0x12a") => HexNumber::new(298, false),
            parse_hex_number_with_uppercase("0x12A") => HexNumber::new(298, false),
            parse_hex_number_with_mixed_case("0x1bF2A") => HexNumber::new(114_474, false),
            parse_hex_with_exponent("0x12p4") => HexNumber::new(18, false).with_exponent(4, false),
            parse_hex_with_exponent_uppercase("0xABP3") => HexNumber::new(171, false).with_exponent(3, true)
        );

        test_parse_errors!(
            parse_empty_string("") => NumberParsingError::InvalidDecimalNumber,
            missing_exponent_value("1e") => NumberParsingError::InvalidDecimalExponent,
            missing_negative_exponent_value("1e-") => NumberParsingError::InvalidDecimalExponent,
            missing_hex_exponent_value("0x1p") => NumberParsingError::InvalidHexadecimalExponent,
            negative_hex_exponent_value("0x1p-3") => NumberParsingError::InvalidHexadecimalExponent
        );
    }

    mod compute_value {
        use super::*;

        macro_rules! test_compute_value {
            ($($name:ident($input:literal) => $value:expr),*) => {
                $(
                    #[test]
                    fn $name() {
                        let number = NumberExpression::from($input);
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
