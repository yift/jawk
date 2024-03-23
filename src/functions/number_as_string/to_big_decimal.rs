use std::str::FromStr;

use bigdecimal::BigDecimal;

use crate::json_value::JsonValue;

pub trait BigDecimalConvert {
    fn to_big_decimal(&self) -> Option<BigDecimal>;
}

impl From<BigDecimal> for JsonValue {
    fn from(value: BigDecimal) -> Self {
        value.normalized().to_string().into()
    }
}
impl BigDecimalConvert for Option<JsonValue> {
    fn to_big_decimal(&self) -> Option<BigDecimal> {
        match self {
            Some(JsonValue::String(str)) => BigDecimal::from_str(str).ok(),
            _ => None,
        }
    }
}
