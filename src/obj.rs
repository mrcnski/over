//! A map of keys to values, where values can be any type, including other objects.

use crate::{
    arr::Arr,
    error::OverError,
    parse::{self, format::Format},
    tup::Tup,
    util,
    value::Value,
    OverResult, ReferenceType, INDENT_STEP,
};
use num_bigint::BigInt;
use num_rational::BigRational;
use std::{fmt, slice::Iter, str::FromStr, sync::Arc};

/// Field-value pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pair(pub String, pub Value);

#[derive(Clone, Debug)]
struct ObjInner {
    // Field-value pairs. Stored in the order they are added.
    pairs: Vec<Pair>,
    // Optional parent.
    parent: Option<Obj>,
    // Unique ID.
    id: usize,
}

/// `Obj` struct.
///
/// This struct is immutable and cannot be modified once created.
#[derive(Clone, Debug)]
pub struct Obj {
    inner: Arc<ObjInner>,
}

macro_rules! get_fn {
    ( $doc:expr, $name:tt, $type:ty ) => {
        #[doc=$doc]
        pub fn $name(&self, field: &str) -> OverResult<$type> {
            match self.get(field) {
                Some(value) => match value.$name() {
                    Ok(result) => Ok(result),
                    e @ Err(_) => e,
                },
                None => Err(OverError::FieldNotFound(field.into())),
            }
        }
    };
}

impl Obj {
    get_fn!(
        "Returns the `bool` found at `field`. Returns an error if the field was not found or if \
         the `Value` at `field` is not `Bool`.",
        get_bool,
        bool
    );

    get_fn!(
        "Returns the `BigInt` found at `field`. Returns an error if the field was not found or if \
         the `Value` at `field` is not `Int`.",
        get_int,
        BigInt
    );

    get_fn!(
        "Returns the `BigRational` found at `field`. Returns an error if the field was not found \
         or if the `Value` at `field` is not `Frac`.",
        get_frac,
        BigRational
    );

    get_fn!(
        "Returns the `String` found at `field`. Returns an error if the field was not found or if \
         the `Value` at `field` is not `Str`.",
        get_str,
        String
    );

    get_fn!(
        "Returns the `Arr` found at `field`. Returns an error if the field was not found or if \
         the `Value` at `field` is not `Arr`.",
        get_arr,
        Arr
    );

    get_fn!(
        "Returns the `Tup` found at `field`. Returns an error if the field was not found or if \
         the `Value` at `field` is not `Tup`.",
        get_tup,
        Tup
    );

    get_fn!(
        "Returns the `Obj` found at `field`. Returns an error if the field was not found or if \
         the `Value` at `field` is not `Obj`.",
        get_obj,
        Obj
    );

    /// Creates an empty `Obj`.
    pub fn empty() -> Self {
        Self::from_pairs_unchecked(vec![], None)
    }

    /// Returns a new `Obj` created from the given `Vec` of `Pair`s with optional `parent`.
    ///
    /// Returns an error if a pair contains an invalid field name.
    ///
    /// A valid field name must start with an alphabetic character or '_' and subsequent characters
    /// must be alphabetic, numeric, or '_'.
    pub fn from_pairs(pairs: Vec<Pair>, parent: Option<Self>) -> OverResult<Self> {
        for Pair(ref field, _) in &pairs {
            if !Self::is_valid_field(field) {
                return Err(OverError::InvalidFieldName(field.clone()));
            }
        }

        Ok(Self::from_pairs_unchecked(pairs, parent))
    }

    /// Returns a new `Obj` created from the given `Vec` of `Pair`s with optional `parent`.
    ///
    /// It is faster than the safe version, `from_pairs`, if you know every field has a valid name.
    /// You can check ahead of time whether a field is valid with `is_valid_field`.
    ///
    /// See `from_pairs` for more details.
    pub fn from_pairs_unchecked(pairs: Vec<Pair>, parent: Option<Self>) -> Self {
        let id = crate::gen_id();

        Self {
            inner: Arc::new(ObjInner { pairs, parent, id }),
        }
    }

    /// Returns a reference to the inner vec of this `Obj`.
    pub fn pairs_ref(&self) -> &Vec<Pair> {
        &self.inner.pairs
    }

    /// Returns a new `Obj` loaded from a file.
    pub fn from_file(path: &str) -> OverResult<Self> {
        Ok(parse::load_from_file(path)?)
    }

    /// Writes this `Obj` to given file in `.over` representation.
    ///
    /// # Notes
    ///
    /// The fields of the `Obj` will be output in the same order they were read.
    ///
    /// Also note some shorthand from the original file, including mathematical operations and file
    /// includes, may not be preserved when creating the `Obj` representation, and may not appear
    /// when writing to another file.
    pub fn write_to_file(&self, path: &str) -> OverResult<()> {
        util::write_file_str(path, &self.write_to_string())?;
        Ok(())
    }

    /// Writes this `Obj` to a `String`.
    ///
    /// # Notes
    ///
    /// See `write_to_file`.
    pub fn write_to_string(&self) -> String {
        self.format(false, 0)
    }

    /// Iterates over each `(String, Value)` pair in `self`, applying `f`.
    ///
    /// Parent fields are not included.
    pub fn with_each<F>(&self, mut f: F)
    where
        F: FnMut(&String, &Value),
    {
        for Pair(field, value) in &self.inner.pairs {
            f(field, value)
        }
    }

    /// Returns the number of fields for this `Obj`.
    ///
    /// Parent fields are not included.
    pub fn len(&self) -> usize {
        self.inner.pairs.len()
    }

    /// Returns whether this `Obj` is empty.
    ///
    /// Parent fields are not included.
    pub fn is_empty(&self) -> bool {
        self.inner.pairs.is_empty()
    }

    /// Returns true if this `Obj` contains `field`.
    ///
    /// Parent fields are not included.
    pub fn contains(&self, field: &str) -> bool {
        self.inner
            .pairs
            .iter()
            .any(|Pair(pair_field, _)| field == pair_field)
    }

    /// Gets the `Value` associated with `field` in this object or the first parent with `field`.
    pub fn get(&self, field: &str) -> Option<Value> {
        match self
            .inner
            .pairs
            .iter()
            .find(|Pair(ref field_name, _)| field_name == field)
        {
            Some(Pair(_, ref value)) => Some(value.clone()),
            None => match self.inner.parent {
                Some(ref parent) => parent.get(field),
                None => None,
            },
        }
    }

    /// Gets the `Value` associated with `field` and the `Obj` where it was found (either `self` or
    /// one of its parents).
    pub fn get_with_source(&self, field: &str) -> Option<(Value, Self)> {
        match self
            .inner
            .pairs
            .iter()
            .find(|Pair(ref field_name, _)| field_name == field)
        {
            Some(Pair(_, ref value)) => Some((value.clone(), self.clone())),
            None => match self.inner.parent {
                Some(ref parent) => parent.get_with_source(field),
                None => None,
            },
        }
    }

    /// Returns whether this `Obj` has a parent.
    pub fn has_parent(&self) -> bool {
        self.inner.parent.is_some()
    }

    /// Returns the parent for this `Obj`.
    pub fn get_parent(&self) -> Option<Self> {
        self.inner.parent.as_ref().cloned()
    }

    /// An iterator visiting all field-value pairs in order.
    pub fn iter(&self) -> Iter<Pair> {
        self.pairs_ref().iter()
    }

    /// Returns true if `field` is a valid field name for an `Obj`.
    ///
    /// The first character must be alphabetic or '_'. Subsequent characters are allowed to be
    /// alphabetic, digits, or '_'.
    pub fn is_valid_field(field: &str) -> bool {
        let mut first = true;

        for ch in field.chars() {
            if first {
                if !Self::is_valid_field_char(ch, true) {
                    return false;
                }
                first = false;
            } else if !Self::is_valid_field_char(ch, false) {
                return false;
            }
        }

        true
    }

    /// Returns true if the given char is valid for a field, depending on whether it is the first
    /// char or not.
    ///
    /// See `is_valid_field` for more details.
    pub fn is_valid_field_char(ch: char, first: bool) -> bool {
        match ch {
            ch if ch.is_alphabetic() => true,
            ch if parse::util::is_digit(ch) => !first,
            '_' => true,
            '^' => first,
            _ => false,
        }
    }
}

impl ReferenceType for Obj {
    fn id(&self) -> usize {
        self.inner.id
    }

    fn num_references(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Default for Obj {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format(true, INDENT_STEP))
    }
}

impl FromStr for Obj {
    type Err = OverError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(parse::load_from_str(s)?)
    }
}

/// For two Objs to be equal, the following two checks must pass:
/// 1. If either Obj has a parent, then both must have parents and the parents must be equal.
/// 2. The two Objs must have all the same fields pointing to the same values.
impl PartialEq for Obj {
    fn eq(&self, other: &Self) -> bool {
        let inner = &self.inner;
        let other_inner = &other.inner;

        // Check parent equality.
        if inner.parent.is_some() && other_inner.parent.is_some() {
            let parent = self.get_parent().unwrap();
            let other_parent = other.get_parent().unwrap();
            if parent != other_parent {
                return false;
            }
        } else if !(inner.parent.is_none() && other_inner.parent.is_none()) {
            return false;
        }

        // Check pairs equality.
        inner.pairs == other_inner.pairs
    }
}

impl Eq for Obj {}
