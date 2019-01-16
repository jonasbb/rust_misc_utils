use num_traits::Bounded;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    iter::FromIterator,
    str::FromStr,
};

/// Helper type to ensure to calculate a minimal value
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Min<T> {
    value: Option<T>,
}

impl<T> Min<T>
where
    T: Copy + Ord,
{
    /// Create a new instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new instance with an initial value to compare to
    pub fn with_initial(initial: T) -> Self {
        Self {
            value: Some(initial),
        }
    }

    /// Return the minimal value found so far
    ///
    /// Returns `None` if neither an initial value exists nor `update` was called.
    /// Returns `Some(T)` if at least one value exists.
    pub fn get_min(&self) -> Option<T> {
        self.value
    }

    /// Return the minimal value found so far
    ///
    /// This method falls back to the maximal value for type `T`, if no other value exists.
    pub fn get_min_extreme(&self) -> T
    where
        T: Bounded,
    {
        self.get_min().unwrap_or_else(T::max_value)
    }

    /// Update the value by replacing it with a lower value
    ///
    /// If `value` is less than `self`, then self is replaced with `value`.
    /// This method can be called with type `T` or type `Min<T>`.
    pub fn update<V: Into<Self>>(&mut self, value: V) {
        match (self.value, value.into().value) {
            (None, None) => self.value = None,
            (Some(v), None) | (None, Some(v)) => self.value = Some(v),
            (Some(v1), Some(v2)) => self.value = Some(v1.min(v2)),
        }
    }
}

impl<T> Default for Min<T> {
    fn default() -> Self {
        Self { value: None }
    }
}

impl<T> From<T> for Min<T>
where
    T: Copy + Ord,
{
    fn from(value: T) -> Self {
        Self::with_initial(value)
    }
}

impl<T> FromIterator<T> for Min<T>
where
    T: Ord,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let m = iter.into_iter().min();
        if m.is_some() {
            Self { value: m }
        } else {
            Self::default()
        }
    }
}

impl<T> Display for Min<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        if let Some(v) = &self.value {
            write!(f, "{}", v)
        } else {
            write!(f, "<uninitialized>")
        }
    }
}

impl<T> FromStr for Min<T>
where
    T: Copy + FromStr + Ord,
{
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::with_initial(T::from_str(s)?))
    }
}

/// Helper type to ensure to calculate a maximal value
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Max<T> {
    value: Option<T>,
}

impl<T> Max<T>
where
    T: Copy + Ord,
{
    /// Create a new instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new instance with an initial value to compare to
    pub fn with_initial(initial: T) -> Self {
        Self {
            value: Some(initial),
        }
    }

    /// Return the maximal value found so far
    ///
    /// Returns `None` if neither an initial value exists nor `update` was called.
    /// Returns `Some(T)` if at least one value exists.
    pub fn get_max(&self) -> Option<T> {
        self.value
    }

    /// Return the maximal value found so far
    ///
    /// This method falls back to the minimal value for type `T`, if no other value exists.
    pub fn get_max_extreme(&self) -> T
    where
        T: Bounded,
    {
        self.get_max().unwrap_or_else(T::min_value)
    }

    /// Update the value by replacing it with a higher value
    ///
    /// If `value` is greater than `self`, then self is replaced with `value`.
    /// This method can be called with type `T` or type `Max<T>`.
    pub fn update<V: Into<Self>>(&mut self, value: V) {
        match (self.value, value.into().value) {
            (None, None) => self.value = None,
            (Some(v), None) | (None, Some(v)) => self.value = Some(v),
            (Some(v1), Some(v2)) => self.value = Some(v1.max(v2)),
        }
    }
}

impl<T> Default for Max<T> {
    fn default() -> Self {
        Self { value: None }
    }
}

impl<T> From<T> for Max<T>
where
    T: Copy + Ord,
{
    fn from(value: T) -> Self {
        Self::with_initial(value)
    }
}

impl<T> FromIterator<T> for Max<T>
where
    T: Ord,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let m = iter.into_iter().max();
        if m.is_some() {
            Self { value: m }
        } else {
            Self::default()
        }
    }
}

impl<T> Display for Max<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        if let Some(v) = &self.value {
            write!(f, "{}", v)
        } else {
            write!(f, "<uninitialized>")
        }
    }
}

impl<T> FromStr for Max<T>
where
    T: Copy + FromStr + Ord,
{
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::with_initial(T::from_str(s)?))
    }
}
