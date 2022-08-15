//! Values.

use crate::{
    arr, error::OverError, obj, parse::format::Format, tup, types::Type, OverResult, INDENT_STEP,
};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::ToPrimitive;
use std::fmt;

/// Enum of possible values and their inner types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    /// A null value.
    Null,

    // Copy values.
    /// A boolean value.
    Bool(bool),
    /// A signed integer value.
    Int(BigInt),
    /// A fractional value.
    Frac(BigRational),
    /// A string value.
    Str(String),

    // Reference values.
    /// An array value.
    Arr(arr::Arr),
    /// A tuple value.
    Tup(tup::Tup),
    /// An object value.
    Obj(obj::Obj),
}

macro_rules! get_fn {
    ( $doc:expr, $name:tt, $type:ty, $variant:ident ) => {
        #[doc=$doc]
        pub fn $name(&self) -> OverResult<$type> {
            if let Self::$variant(ref inner) = *self {
                Ok(inner.clone())
            } else {
                Err(OverError::TypeMismatch(Type::$variant, self.get_type()))
            }
        }
    };
}

impl Value {
    get_fn!(
        "Returns the `bool` contained in this `Value`. Returns an error if this `Value` is not \
         `Bool`.",
        get_bool,
        bool,
        Bool
    );

    get_fn!(
        "Returns the `BigInt` contained in this `Value`. Returns an error if this `Value` is not \
         `Int`.",
        get_int,
        BigInt,
        Int
    );

    get_fn!(
        "Returns the `String` contained in this `Value`. Returns an error if this `Value` is not \
         `Str`.",
        get_str,
        String,
        Str
    );

    get_fn!(
        "Returns the `Obj` contained in this `Value`. Returns an error if this `Value` is not \
         `Obj`.",
        get_obj,
        obj::Obj,
        Obj
    );

    /// Returns true if this `Value` is null.
    pub fn is_null(&self) -> bool {
        if let Self::Null = *self {
            true
        } else {
            false
        }
    }

    /// Returns the `Type` of this `Value`.
    pub fn get_type(&self) -> Type {
        use self::Value::*;

        match *self {
            Null => Type::Null,
            Bool(_) => Type::Bool,
            Int(_) => Type::Int,
            Frac(_) => Type::Frac,
            Str(_) => Type::Str,
            Arr(ref arr) => Type::Arr(Box::new(arr.inner_type())),
            Tup(ref tup) => Type::Tup(tup.inner_type_vec()),
            Obj(_) => Type::Obj,
        }
    }

    /// Returns the `BigRational` contained in this `Value`.
    /// Returns an error if this `Value` is not `Frac`.
    pub fn get_frac(&self) -> OverResult<BigRational> {
        match *self {
            Self::Frac(ref inner) => Ok(inner.clone()),
            Self::Int(ref inner) => Ok(frac!(inner.clone(), 1)),
            _ => Err(OverError::TypeMismatch(Type::Frac, self.get_type())),
        }
    }

    /// Returns the `Arr` contained in this `Value`.
    /// Returns an error if this `Value` is not `Arr`.
    pub fn get_arr(&self) -> OverResult<arr::Arr> {
        if let Self::Arr(ref inner) = *self {
            Ok(inner.clone())
        } else {
            Err(OverError::TypeMismatch(
                Type::Arr(Box::new(Type::Any)),
                self.get_type(),
            ))
        }
    }

    /// Returns the `Tup` contained in this `Value`.
    /// Returns an error if this `Value` is not `Tup`.
    pub fn get_tup(&self) -> OverResult<tup::Tup> {
        if let Self::Tup(ref inner) = *self {
            Ok(inner.clone())
        } else {
            Err(OverError::TypeMismatch(Type::Tup(vec![]), self.get_type()))
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format(true, INDENT_STEP))
    }
}

// impl PartialEq

macro_rules! impl_eq {
    ($valtype:ident, $type:ty) => {
        impl PartialEq<$type> for Value {
            fn eq(&self, other: &$type) -> bool {
                match *self {
                    Self::$valtype(ref value) => value == other,
                    _ => false,
                }
            }
        }

        impl PartialEq<Value> for $type {
            fn eq(&self, other: &Value) -> bool {
                match *other {
                    Value::$valtype(ref value) => value == self,
                    _ => false,
                }
            }
        }
    };
}

impl_eq!(Bool, bool);
impl_eq!(Int, BigInt);
impl_eq!(Frac, BigRational);
impl_eq!(Str, String);
impl_eq!(Arr, arr::Arr);
impl_eq!(Tup, tup::Tup);
impl_eq!(Obj, obj::Obj);

impl<'a> PartialEq<&'a str> for Value {
    fn eq(&self, other: &&str) -> bool {
        match *self {
            Self::Str(ref value) => value == other,
            _ => false,
        }
    }
}

impl<'a> PartialEq<Value> for &'a str {
    fn eq(&self, other: &Value) -> bool {
        match *other {
            Value::Str(ref value) => value == self,
            _ => false,
        }
    }
}

// PartialEq for integers

macro_rules! impl_eq_int {
    ($type:ty, $fn:tt) => {
        impl PartialEq<$type> for Value {
            fn eq(&self, other: &$type) -> bool {
                match *self {
                    Self::Int(ref value) => match value.$fn() {
                        Some(value) => value == *other,
                        None => false,
                    },
                    _ => false,
                }
            }
        }

        impl PartialEq<Value> for $type {
            fn eq(&self, other: &Value) -> bool {
                match *other {
                    Value::Int(ref value) => match value.$fn() {
                        Some(value) => value == *self,
                        None => false,
                    },
                    _ => false,
                }
            }
        }
    };
}

impl_eq_int!(usize, to_usize);
impl_eq_int!(u8, to_u8);
impl_eq_int!(u16, to_u16);
impl_eq_int!(u32, to_u32);
impl_eq_int!(u64, to_u64);
impl_eq_int!(i8, to_i8);
impl_eq_int!(i16, to_i16);
impl_eq_int!(i32, to_i32);
impl_eq_int!(i64, to_i64);

// impl From

macro_rules! impl_from {
    ($type:ty, $fn:tt) => {
        impl From<$type> for Value {
            fn from(inner: $type) -> Self {
                Self::$fn(inner.into())
            }
        }
    };
}

impl_from!(bool, Bool);

impl_from!(usize, Int);
impl_from!(u8, Int);
impl_from!(u16, Int);
impl_from!(u32, Int);
impl_from!(u64, Int);
impl_from!(i8, Int);
impl_from!(i16, Int);
impl_from!(i32, Int);
impl_from!(i64, Int);
impl_from!(BigInt, Int);

// This is commented because the resultant values don't pass equality checks.
//
// impl From<f32> for Value {
//     fn from(inner: f32) -> Self {
//         Value::Frac(BigRational::from_f32(inner).unwrap())
//     }
// }
// impl From<f64> for Value {
//     fn from(inner: f64) -> Self {
//         Value::Frac(BigRational::from_f64(inner).unwrap())
//     }
// }
impl_from!(BigRational, Frac);

impl_from!(String, Str);
impl<'a> From<&'a str> for Value {
    fn from(inner: &str) -> Self {
        Self::Str(inner.into())
    }
}

impl_from!(arr::Arr, Arr);

impl_from!(tup::Tup, Tup);

impl_from!(obj::Obj, Obj);
