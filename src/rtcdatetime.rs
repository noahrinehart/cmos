use core::{
	cmp::Ordering,
	fmt::{Display, Formatter, Result},
	ops::{Add, AddAssign, Sub, SubAssign},
	u8::MAX,
	usize::MAX as usize_MAX,
};

/// Results struct from reading RTC with self-explanatory fields
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RTCDateTime {
	pub year: usize,
	pub month: u8,
	pub day: u8,
	pub hour: u8,
	pub minute: u8,
	pub second: u8,
}

impl Ord for RTCDateTime {
	/// Compare the fields one by one in descending order
	fn cmp(&self, other: &Self) -> Ordering {
		(self.year, self.month, self.day, self.hour, self.minute, self.second).cmp(&(
			other.year,
			other.month,
			other.day,
			other.hour,
			other.minute,
			other.second,
		))
	}
}

impl PartialOrd for RTCDateTime {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Display for RTCDateTime {
	/// Prints a `RTCDateTime` formatted according to the [ISO 8601](https://en.wikipedia.org/wiki/ISO_8601) standard.
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{}-{}-{}T{}:{}:{}Z", self.year, self.month, self.day, self.hour, self.minute, self.second)
	}
}

impl Add for RTCDateTime {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		unimplemented!();
		RTCDateTime::min()
		/*
		Self {
			x: self.x + other.x,
			y: self.y + other.y,
		}
		*/
	}
}

impl AddAssign for RTCDateTime {
	fn add_assign(&mut self, other: Self) {
		*self = RTCDateTime::add(*self, other);
	}
}

impl Sub for RTCDateTime {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		unimplemented!();
		RTCDateTime::min()
		/*
		Self {
			x: self.x - other.x,
			y: self.y - other.y,
		}
		*/
	}
}

impl SubAssign for RTCDateTime {
	fn sub_assign(&mut self, other: Self) {
		*self = RTCDateTime::sub(*self, other);
	}
}

impl RTCDateTime {
	/// Returns the maximal `RTCDateTime` possible.
	pub fn max() -> Self { Self { year: usize_MAX, month: MAX, day: MAX, hour: MAX, minute: MAX, second: MAX } }

	/// Returns the minimal `RTCDateTime` possible.
	pub fn min() -> Self { Self { year: 0, month: 0, day: 0, hour: 0, minute: 0, second: 0 } }

	/// Check if the `RTCDateTime` instance is a valid date.
	/// The function takes into account the number of days in months and leap years.
	pub fn is_valid(&self) -> bool {
		self.month < 13 && self.hour < 25 && self.minute < 60 && self.second < 60 && self.day == RTCDateTime::days_by_month(self.year, self.month)
	}

	/// Transforms the caller into a valid `RTCDateTime`.
	pub fn into_valid(mut self) -> Self {
		if self.is_valid() {
			self
		} else {
			loop {
				if self.second < 60 {
					break;
				}
				self.second -= 60;
				self.minute += 1;
			}
			loop {
				if self.minute < 60 {
					break;
				}
				self.minute -= 60;
				self.hour += 1;
			}
			loop {
				if self.hour < 24 {
					break;
				}
				self.hour -= 24;
				self.day += 1;
			}
			loop {
				if self.day < RTCDateTime::days_by_month(self.year, self.month) && self.month < 13 {
					break;
				}
				if RTCDateTime::days_by_month(self.year, self.month) < self.day {
					self.day -= RTCDateTime::days_by_month(self.year, self.month);
					self.month += 1;
				}
				if 12 < self.month {
					self.month -= 12;
					if self.year < usize_MAX {
						self.year += 1;
					} else {
						return RTCDateTime::max();
					}
				}
			}
			self
		}
	}

	/// Attempt to create a valid `RTCDateTime` from a tuple.
	/// Returns `Some(RTCDateTime)` in case of success, or `None` if the operation failed.
	pub fn from_tuple(tuple: &(usize, u8, u8, u8, u8, u8)) -> Option<Self> {
		let new = Self { year: tuple.0, month: tuple.1, day: tuple.2, hour: tuple.3, minute: tuple.4, second: tuple.5 };
		new.into_valid()
	}

	// Tranforms the calling instance into a tuple containing all its fields by descending order.
	pub fn into_tuple(self) -> (usize, u8, u8, u8, u8, u8) {
		(self.year, self.month, self.day, self.hour, self.minute, self.second)
	}

	/// Returns a tuple containing the fields of a `RTCDateTime` by descending order.
	pub fn as_tuple(&self) -> (usize, u8, u8, u8, u8, u8) {
		(self.year, self.month, self.day, self.hour, self.minute, self.second)
	}

	/// returns the maximal number of days given a month and a year.
	fn days_by_month(year: usize, month: u8) -> u8 {
		match month {
			1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
			4 | 6 | 9 | 11 => 30,
			_ => {
				if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
					29
				} else {
					28
				}
			}
		}
	}
}
