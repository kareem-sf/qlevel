//! Fixed-point `DecimalValue` newtype with string JSON/IPC serialization.

use std::fmt;
use std::str::FromStr;

use rust_decimal::{Decimal, RoundingStrategy};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use ts_rs::TS;

use crate::error::{DomainError, DomainResult};

/// Exact base-10 fixed-point number serialized across JSON and Tauri IPC as a string.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, TS)]
#[ts(type = "string")]
pub struct DecimalValue(pub Decimal);

impl DecimalValue {
    /// Exact additive identity (`0`).
    pub const ZERO: Self = Self(Decimal::ZERO);
    /// Exact multiplicative identity (`1`).
    pub const ONE: Self = Self(Decimal::ONE);
    /// Exact constant `100` for percentage conversions.
    pub const ONE_HUNDRED: Self = Self(Decimal::ONE_HUNDRED);

    /// Wraps a raw `rust_decimal::Decimal`.
    #[must_use]
    pub const fn new(inner: Decimal) -> Self {
        Self(inner)
    }

    /// Constructs a `DecimalValue` from a signed 64-bit integer.
    #[must_use]
    pub fn from_i64(val: i64) -> Self {
        Self(Decimal::from(val))
    }

    /// Constructs a `DecimalValue` from an unsigned 64-bit integer.
    #[must_use]
    pub fn from_u64(val: u64) -> Self {
        Self(Decimal::from(val))
    }

    /// Constructs a `DecimalValue` from mantissa and base-10 scale (`mantissa * 10^-scale`).
    #[must_use]
    pub fn from_parts_i64(mantissa: i64, scale: u32) -> Self {
        Self(Decimal::from_i128_with_scale(i128::from(mantissa), scale))
    }

    /// Parses an exact base-10 decimal string, stripping optional thousands commas and spaces.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidDecimal`] if the string is not a valid base-10 number.
    pub fn parse_clean(raw: &str) -> DomainResult<Self> {
        let cleaned: String = raw
            .trim()
            .chars()
            .filter(|c| *c != ',' && *c != '_' && !c.is_whitespace())
            .collect();
        if cleaned.is_empty() {
            return Err(DomainError::InvalidDecimal {
                raw: raw.to_owned(),
                reason: "empty decimal input".to_owned(),
            });
        }
        Decimal::from_str(&cleaned)
            .map(Self)
            .map_err(|err| DomainError::InvalidDecimal {
                raw: raw.to_owned(),
                reason: err.to_string(),
            })
    }

    /// Returns `true` when equal to zero.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Returns `true` when strictly less than zero.
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.0.is_sign_negative() && !self.0.is_zero()
    }

    /// Returns `true` when strictly greater than zero.
    #[must_use]
    pub const fn is_positive(&self) -> bool {
        self.0.is_sign_positive() && !self.0.is_zero()
    }

    /// Returns the absolute value.
    #[must_use]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// Checked base-10 addition.
    ///
    /// # Errors
    /// Returns [`DomainError::ArithmeticOverflow`] on 96-bit mantissa overflow.
    pub fn checked_add(self, rhs: Self) -> DomainResult<Self> {
        self.0
            .checked_add(rhs.0)
            .map(Self)
            .ok_or(DomainError::ArithmeticOverflow {
                operation: "decimal addition",
            })
    }

    /// Checked base-10 subtraction.
    ///
    /// # Errors
    /// Returns [`DomainError::ArithmeticOverflow`] on 96-bit mantissa overflow.
    pub fn checked_sub(self, rhs: Self) -> DomainResult<Self> {
        self.0
            .checked_sub(rhs.0)
            .map(Self)
            .ok_or(DomainError::ArithmeticOverflow {
                operation: "decimal subtraction",
            })
    }

    /// Checked base-10 multiplication.
    ///
    /// # Errors
    /// Returns [`DomainError::ArithmeticOverflow`] on 96-bit mantissa overflow.
    pub fn checked_mul(self, rhs: Self) -> DomainResult<Self> {
        self.0
            .checked_mul(rhs.0)
            .map(Self)
            .ok_or(DomainError::ArithmeticOverflow {
                operation: "decimal multiplication",
            })
    }

    /// Checked base-10 division.
    ///
    /// # Errors
    /// Returns [`DomainError::DivisionByZero`] if `rhs == 0` or [`DomainError::ArithmeticOverflow`].
    pub fn checked_div(self, rhs: Self) -> DomainResult<Self> {
        if rhs.is_zero() {
            return Err(DomainError::DivisionByZero {
                operation: "decimal division",
            });
        }
        self.0
            .checked_div(rhs.0)
            .map(Self)
            .ok_or(DomainError::ArithmeticOverflow {
                operation: "decimal division",
            })
    }

    /// Commercial half-up rounding (`MidpointAwayFromZero`) to `scale` fractional digits.
    #[must_use]
    pub fn round_commercial(self, scale: u32) -> Self {
        Self(
            self.0
                .round_dp_with_strategy(scale, RoundingStrategy::MidpointAwayFromZero),
        )
    }

    /// Normalizes trailing fractional zeros while preserving exact value (`10.500` -> `10.5`).
    #[must_use]
    pub fn normalize(self) -> Self {
        Self(self.0.normalize())
    }

    /// Formats with fixed fractional digits (`scale`) for deterministic reports.
    #[must_use]
    pub fn to_fixed_string(self, scale: u32) -> String {
        let rounded = self.round_commercial(scale);
        let scale_usize = scale as usize;
        format!("{:.prec$}", rounded.0, prec = scale_usize)
    }
}

impl fmt::Display for DecimalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for DecimalValue {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_clean(s)
    }
}

impl From<Decimal> for DecimalValue {
    fn from(value: Decimal) -> Self {
        Self(value)
    }
}

impl From<i64> for DecimalValue {
    fn from(value: i64) -> Self {
        Self::from_i64(value)
    }
}

impl Serialize for DecimalValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for DecimalValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DecimalValueVisitor;

        impl de::Visitor<'_> for DecimalValueVisitor {
            type Value = DecimalValue;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a base-10 decimal string or integer")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                DecimalValue::parse_clean(value).map_err(de::Error::custom)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(DecimalValue::from_i64(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(DecimalValue::from_u64(value))
            }
        }

        deserializer.deserialize_any(DecimalValueVisitor)
    }
}
