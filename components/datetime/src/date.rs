// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/master/LICENSE ).
//! APIs for Date and Time handling
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::ops::{Add, Sub};
use std::str::FromStr;

#[derive(Debug)]
pub enum DateTimeError {
    Parse(std::num::ParseIntError),
    Overflow { field: &'static str, max: usize },
}

impl fmt::Display for DateTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "{}", err),
            Self::Overflow { field, max } => write!(f, "{} must be between 0-{}", field, max),
        }
    }
}

impl From<std::num::ParseIntError> for DateTimeError {
    fn from(input: std::num::ParseIntError) -> Self {
        Self::Parse(input)
    }
}

/// Temporary trait used to represent the input data for [`DateTimeFormat`].
///
/// This type represents all data that the formatted needs in order to produced formatted string.
///
/// *Note*: At the moment we support only `gregorian` calendar, and plan to extend support to
/// other calendars in the upcoming releases. See <https://github.com/unicode-org/icu4x/issues/355>
///
/// [`DateTimeFormat`]: super::DateTimeFormat
pub trait DateTimeType: FromStr {
    fn year(&self) -> usize;
    fn month(&self) -> Month;
    fn day(&self) -> Day;
    fn hour(&self) -> Hour;
    fn minute(&self) -> Minute;
    fn second(&self) -> Second;
}

/// Temporary implementation of [`DateTimeType`],
/// which is used in tests, benchmarks and examples of this component.
///
/// *Notice:* Rust at the moment does not have a canonical way to represent date and time. We are introducing
/// `MockDateTime` as an example of the data necessary for ICU [`DateTimeFormat`] to work, and
/// [we hope to work with the community](https://github.com/unicode-org/icu4x/blob/master/docs/research/date_time.md)
/// to develop core date and time APIs that will work as an input for this component.
///
/// # Examples
///
/// ```
/// use icu_datetime::date::MockDateTime;
///
/// let dt1 = MockDateTime::try_new(2020, 9, 24, 13, 21, 0)
///     .expect("Failed to construct DateTime.");
///
/// let dt2: MockDateTime = "2020-10-14T13:21:00".parse()
///     .expect("Failed to parse a date time.");
/// ```
/// [`DateTimeFormat`]: super::DateTimeFormat
#[derive(Debug, Default)]
pub struct MockDateTime {
    pub year: usize,
    pub month: Month,
    pub day: Day,
    pub hour: Hour,
    pub minute: Minute,
    pub second: Second,
}

impl MockDateTime {
    /// Creates a new `MockDateTime` from a list of already validated date/time parameters.
    pub const fn new(
        year: usize,
        month: Month,
        day: Day,
        hour: Hour,
        minute: Minute,
        second: Second,
    ) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    /// Constructor for the `MockDateTime`.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu_datetime::date::MockDateTime;
    ///
    /// let dt = MockDateTime::try_new(2020, 9, 24, 13, 21, 0)
    ///     .expect("Failed to construct a DateTime");
    /// ```
    pub fn try_new(
        year: usize,
        month: usize,
        day: usize,
        hour: usize,
        minute: usize,
        second: usize,
    ) -> Result<Self, DateTimeError> {
        Ok(Self {
            year,
            month: month.try_into()?,
            day: day.try_into()?,
            hour: hour.try_into()?,
            minute: minute.try_into()?,
            second: second.try_into()?,
        })
    }
}

impl DateTimeType for MockDateTime {
    fn year(&self) -> usize {
        self.year
    }
    fn month(&self) -> Month {
        self.month
    }
    fn day(&self) -> Day {
        self.day
    }
    fn hour(&self) -> Hour {
        self.hour
    }
    fn minute(&self) -> Minute {
        self.minute
    }
    fn second(&self) -> Second {
        self.second
    }
}

impl FromStr for MockDateTime {
    type Err = DateTimeError;

    /// Parse a `MockDateTime` from a string.
    ///
    /// This utility is for easily creating dates, not a complete robust solution. The
    /// string must take a specific form of the ISO 8601 format: `YYYY-MM-DDThh:mm:ss`.
    ///
    /// ```
    /// use icu_datetime::date::MockDateTime;
    ///
    /// let date: MockDateTime = "2020-10-14T13:21:00".parse()
    ///     .expect("Failed to parse a date time.");
    /// ```
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let year: usize = input[0..4].parse()?;
        let month: Month = input[5..7].parse()?;
        let day: Day = input[8..10].parse()?;
        let hour: Hour = input[11..13].parse()?;
        let minute: Minute = input[14..16].parse()?;
        let second: Second = input[17..19].parse()?;
        Ok(Self {
            year,
            month: month - 1,
            day: day - 1,
            hour,
            minute,
            second,
        })
    }
}

/// This macro defines a struct for each type of unit to be used in a DateTime. Each
/// unit is bounded by a range. The traits implemented here will return a Result on
/// whether or not the unit is in range from the given input.
macro_rules! dt_unit {
    ($name:ident, $value:expr) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Hash)]
        pub struct $name(u8);

        impl $name {
            pub const fn new_unchecked(input: u8) -> Self {
                Self(input)
            }
        }

        impl FromStr for $name {
            type Err = DateTimeError;

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                let val: u8 = input.parse()?;
                if val > $value {
                    Err(DateTimeError::Overflow {
                        field: "$name",
                        max: $value,
                    })
                } else {
                    Ok(Self(val))
                }
            }
        }

        impl TryFrom<u8> for $name {
            type Error = DateTimeError;

            fn try_from(input: u8) -> Result<Self, Self::Error> {
                if input > $value {
                    Err(DateTimeError::Overflow {
                        field: "$name",
                        max: $value,
                    })
                } else {
                    Ok(Self(input))
                }
            }
        }

        impl TryFrom<usize> for $name {
            type Error = DateTimeError;

            fn try_from(input: usize) -> Result<Self, Self::Error> {
                if input > $value {
                    Err(DateTimeError::Overflow {
                        field: "$name",
                        max: $value,
                    })
                } else {
                    Ok(Self(input as u8))
                }
            }
        }

        impl From<$name> for u8 {
            fn from(input: $name) -> Self {
                input.0
            }
        }

        impl From<$name> for usize {
            fn from(input: $name) -> Self {
                input.0 as Self
            }
        }

        impl Add<u8> for $name {
            type Output = Self;

            fn add(self, other: u8) -> Self {
                Self(self.0 + other)
            }
        }

        impl Sub<u8> for $name {
            type Output = Self;

            fn sub(self, other: u8) -> Self {
                Self(self.0 - other)
            }
        }
    };
}

dt_unit!(Month, 12);
dt_unit!(WeekDay, 7);
dt_unit!(Day, 32);
dt_unit!(Hour, 24);
dt_unit!(Minute, 60);
dt_unit!(Second, 60);
