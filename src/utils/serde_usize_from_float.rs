use std::num::FpCategory;

use serde::{Deserialize, Deserializer};

pub(crate) fn deserialize_usize_from_float<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    let value = f64::deserialize(deserializer)?;

    match value.classify() {
        FpCategory::Nan => Err(serde::de::Error::custom("NaN is not a valid usize")),
        FpCategory::Infinite => {
            if value.is_sign_positive() {
                Ok(usize::MAX)
            } else {
                Err(serde::de::Error::custom(
                    "Negative infinity is not a valid usize",
                ))
            }
        }
        FpCategory::Zero => Ok(0),
        FpCategory::Subnormal | FpCategory::Normal => {
            if value < 0.0 {
                Err(serde::de::Error::custom(
                    "Negative numbers are not a valid usize",
                ))
            } else {
                Ok(value.trunc() as usize)
            }
        }
    }
}
