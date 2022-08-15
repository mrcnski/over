//! Module for types.

use std::fmt;

/// Enum of possible types for `Value`s.
#[derive(Clone, Debug)]
pub enum Type {
    /// A type used to indicate an empty Arr.
    Any,
    /// Null value.
    Null,

    /// A boolean type.
    Bool,
    /// A signed integer type.
    Int,
    /// A fractional type.
    Frac,
    /// A string type.
    Str,

    /// An array type, containing the type of its sub-elements.
    Arr(Box<Type>),
    /// A tuple type, containing the types of its sub-elements.
    Tup(Vec<Type>),
    /// An object type.
    Obj,
}

impl Type {
    /// Returns true if this type is strictly the same as `other`.
    /// Usually you want to use `eq()` instead.
    pub fn is(&self, other: &Type) -> bool {
        use self::Type::*;

        match *self {
            Any => {
                if let Any = *other {
                    true
                } else {
                    false
                }
            }

            Null => {
                if let Null = *other {
                    true
                } else {
                    false
                }
            }
            Bool => {
                if let Bool = *other {
                    true
                } else {
                    false
                }
            }
            Int => {
                if let Int = *other {
                    true
                } else {
                    false
                }
            }
            Frac => {
                if let Frac = *other {
                    true
                } else {
                    false
                }
            }
            Str => {
                if let Str = *other {
                    true
                } else {
                    false
                }
            }
            Obj => {
                if let Obj = *other {
                    true
                } else {
                    false
                }
            }

            Arr(ref t1) => {
                if let Arr(ref t2) = *other {
                    t1.is(t2)
                } else {
                    false
                }
            }

            Tup(ref tvec1) => {
                if let Tup(ref tvec2) = *other {
                    if tvec1.len() != tvec2.len() {
                        return false;
                    }
                    tvec1.iter().zip(tvec2.iter()).all(|(t1, t2)| t1.is(t2))
                } else {
                    false
                }
            }
        }
    }

    /// Returns true if this `Type` contains `Any`.
    pub fn has_any(&self) -> bool {
        match *self {
            Self::Any => true,
            Self::Arr(ref t) => Self::has_any(t),
            Self::Tup(ref tvec) => tvec.iter().any(Self::has_any),
            _ => false,
        }
    }

    /// Returns a type with the most specificity that can be applied to the two input types as well
    /// as `true` if the returned type is not maximally specific, that is, it contains `Any`. If no
    /// single type can be applied to both input types (e.g. the types are `Str` and `Int`), returns
    /// `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate over;
    /// # fn main() {
    ///
    /// use over::types::Type;
    /// use over::types::Type::*;
    /// use over::value::Value;
    ///
    /// let val1: Value = tup!(arr![], arr![2]).into();
    /// let val2: Value = tup!(arr!["c"], arr![]).into();
    ///
    /// let (specific_type, has_any) =
    ///     Type::most_specific(&val1.get_type(), &val2.get_type()).unwrap();
    ///
    /// assert_eq!(specific_type, Tup(vec![Arr(Box::new(Str)), Arr(Box::new(Int))]));
    /// assert!(!has_any);
    ///
    /// # }
    /// ```
    pub fn most_specific(type1: &Type, type2: &Type) -> Option<(Type, bool)> {
        use self::Type::*;

        if let Any = *type2 {
            return Some((type1.clone(), type1.has_any()));
        }

        match *type1 {
            Any => Some((type2.clone(), type2.has_any())),

            Arr(ref t1) => {
                if let Arr(ref t2) = *type2 {
                    Self::most_specific(t1, t2).map(|(t, any)| (Arr(Box::new(t)), any))
                } else {
                    None
                }
            }

            Tup(ref tvec1) => {
                if let Tup(ref tvec2) = *type2 {
                    if tvec1.len() == tvec2.len() {
                        let mut has_any = false;

                        let tvec: Option<Vec<Type>> = tvec1
                            .iter()
                            .zip(tvec2.iter())
                            .map(|(t1, t2)| {
                                Self::most_specific(t1, t2).map(|(t, any)| {
                                    if !has_any && any {
                                        has_any = any;
                                    }
                                    t
                                })
                            })
                            .collect();

                        tvec.map(|tvec| (Tup(tvec), has_any))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            ref t => {
                if t == type2 {
                    Some((t.clone(), false))
                } else {
                    None
                }
            }
        }
    }
}

/// Two types are considered equal if one of them is Any or they have the same variant.
/// In the case of `Arr` and `Tup`, the inner types are recursively checked for equality.
impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        use self::Type::*;

        // If either is Any, always return `true`.
        if let Any = *other {
            return true;
        }

        match *self {
            Any => true,
            Arr(ref box1) => {
                if let Arr(ref box2) = *other {
                    box1 == box2
                } else {
                    false
                }
            }
            Tup(ref tvec1) => {
                if let Tup(ref tvec2) = *other {
                    tvec1 == tvec2
                } else {
                    false
                }
            }
            _ => self.is(other),
        }
    }
}
impl Eq for Type {}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Type::*;

        match *self {
            Any => write!(f, "Any"),
            Null => write!(f, "Null"),
            Bool => write!(f, "Bool"),
            Int => write!(f, "Int"),
            Frac => write!(f, "Frac"),
            Str => write!(f, "Str"),
            Arr(ref boxxy) => write!(f, "Arr({})", boxxy),
            Tup(ref tvec) => write!(
                f,
                "Tup({})",
                match tvec.get(0) {
                    Some(t1) => tvec
                        .iter()
                        .skip(1)
                        .fold(format!("{}", t1), |s, t| format!("{}, {}", s, t)),
                    None => String::from(""),
                }
            ),
            Obj => write!(f, "Obj"),
        }
    }
}
