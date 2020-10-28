/*!
`cmos` is a library for interfacing with the [CMOS chip](https://en.wikipedia.org/wiki/Nonvolatile_BIOS_memory) found on most motherboards.
Along with this generic functionality, there is also added functions for reading the [RTC](https://en.wikipedia.org/wiki/Real-time_clock) (Real Time Clock).

The implementation is based off of the implementation found on the [osdev.org page](https://wiki.osdev.org/CMOS), where more information can be found.

### Notes
Getting date information from the RTC is mostly trivial, except for calculating the year.
Some RTC chips support calculating the correct year through a century register, but this is platform specific.
This can be done by querying the Fixed ACPI Description Table. More details can be found [here](https://wiki.osdev.org/CMOS#Century_Register).
This library also supports providing the current year, through the [`CMOSCenturyHandler`] enum as a fallback.

## Examples
To get the current RTC time using the current year:
```rust,no_run
# use cmos::{CMOS, CMOSCenturyHandler};
// Create a CMOS object (unsafe due to the use of port I/O)
let mut cmos = unsafe { CMOS::new() };
// Read the rtc date time using this year
let rtc = cmos.read_rtc(CMOSCenturyHandler::CurrentYear(2019));
```

To get the current RTC tiem by passing in the century register num:
```rust,no_run
# use cmos::{CMOS, CMOSCenturyHandler};
// Create a CMOS object (unsafe due to the use of port I/O)
let mut cmos = unsafe { CMOS::new() };
// Read the rtc date time using this year
let rtc = cmos.read_rtc(CMOSCenturyHandler::CenturyRegister(0xA5));
```

[`CMOSCenturyHandler`]: enum.CMOSCenturyHandler.html
*/

#![no_std]

use cpuio::Port;
use core::{
	cmp::Ordering,
	fmt::{Display, Formatter, Result},
	usize,
};

/// The standard CMOS struct
#[derive(Debug)]
pub struct CMOS {
	address_port: Port<u8>,
	data_port: Port<u8>,
}

/// Implements the CMOS struct
impl CMOS {
	/// Create a new CMOS struct
	///
	/// Note: This function is unsafe due to the creation of port I/O
	/// # Examples
	/// ```rust,no_run
	/// # use cmos::{CMOS, CMOSCenturyHandler};
	/// let mut cmos = unsafe { CMOS::new() };
	/// ```
	pub unsafe fn new() -> CMOS { CMOS { address_port: Port::<u8>::new(0x70), data_port: Port::<u8>::new(0x71) } }

	/// Reads all the registers in CMOS
	/// # Examples
	/// ```rust,no_run
	/// # use cmos::{CMOS, CMOSCenturyHandler};
	/// let mut cmos = unsafe { CMOS::new() };
	/// // Blank array to read into
	/// let mut cmos_values: [u8; 128] = [0; 128];
	/// // Read values into provided array
	/// cmos.read_all(&mut cmos_values);
	/// ```
	pub fn read_all(&mut self, output: &mut [u8; 128]) {
		for i in 0..128 {
			self.address_port.write(i);
			output[i as usize] = self.data_port.read();
		}
	}

	/// Writes to all the registers in CMOS
	/// # Examples
	/// Writes all 0's, probably not a best idea to actually do this
	/// ```rust,no_run
	/// # use cmos::{CMOS, CMOSCenturyHandler};
	/// let mut cmos = unsafe { CMOS::new() };
	/// // Example values to write (don't do this!)
	/// let values: [u8; 128] = [0; 128];
	/// // Writes values to all CMOS registers
	/// cmos.write_all(&values);
	/// ```
	pub fn write_all(&mut self, input: &[u8; 128]) {
		for i in 0..128 {
			self.address_port.write(i);
			self.data_port.write(input[i as usize]);
		}
	}

	/// Reads from a singe register in CMOS
	/// # Examples
	/// ```rust,no_run
	/// # use cmos::{CMOS, CMOSCenturyHandler};
	/// let mut cmos = unsafe { CMOS::new() };
	/// // Read from register 0x04 in the CMOS
	/// let reg_4 = cmos.read(0x04);
	/// ```
	pub fn read(&mut self, reg: u8) -> u8 {
		self.address_port.write(reg);
		self.data_port.read()
	}

	/// Writes to a singe register in CMOS
	/// # Examples
	/// Writes `0x08` to register `0x04`
	/// ```rust,no_run
	/// # use cmos::{CMOS, CMOSCenturyHandler};
	/// let mut cmos = unsafe { CMOS::new() };
	/// // Write 0x08 into register 0x04
	/// cmos.write(0x04, 0x08);
	/// ```
	pub fn write(&mut self, reg: u8, val: u8) {
		self.address_port.write(reg);
		self.data_port.write(val);
	}

	/// Reads and checks the status of the update in progress flag.
	/// When reading from the RTC, it's best to read until this flag is 0.
	///
	/// More info found [here](https://wiki.osdev.org/CMOS#RTC_Update_In_Progress)
	///
	/// # Examples
	/// ```rust,no_run
	/// # use cmos::{CMOS, CMOSCenturyHandler};
	/// let mut cmos = unsafe { CMOS::new() };
	/// let mut reg0;
	/// // Read register 0x00 until progress flag not 0
	/// while cmos.get_update_in_progress_flag() != 0 {
	///     reg0 = cmos.read(0x00);
	/// }
	/// ```
	/// [`CMOS`]: struct.CMOS.html
	pub fn get_update_in_progress_flag(&mut self) -> u8 { self.read(0x0A) & 0x80 }

	fn read_into_rtc(&mut self, rtc_time: &mut RTCDateTime) {
		while self.get_update_in_progress_flag() != 0 {
			rtc_time.second = self.read(0x00);
			rtc_time.minute = self.read(0x02);
			rtc_time.hour = self.read(0x04);
			rtc_time.day = self.read(0x07);
			rtc_time.month = self.read(0x08);
			rtc_time.year = self.read(0x09) as usize;
		}
	}

	/// Reads from the RTC part of CMOS
	/// Returns an [`RTCDateTime`] struct, which includes all date time fields.
	/// This method automatically converts BCD to binary values and 12 hours to 24 hour if necessary.
	///
	/// # Examples
	/// ```rust,no_run
	/// # use cmos::{CMOS, CMOSCenturyHandler};
	/// let mut cmos = unsafe { CMOS::new() };
	/// // Get current RTC by current year of 2019
	/// let rtc = cmos.read_rtc(CMOSCenturyHandler::CurrentYear(2019));
	/// ```
	/// [`RTCDateTime`]: struct.RTCDateTime.html
	pub fn read_rtc(&mut self, century_handler: CMOSCenturyHandler) -> RTCDateTime {
		let mut rtc_time = RTCDateTime { second: 0, minute: 0, hour: 0, day: 0, month: 0, year: 0 };

		// Note: This uses the "read registers until you get the same values twice in a row" technique to avoid getting
		// dodgy/inconsistent values due to RTC updates
		self.read_into_rtc(&mut rtc_time);

		let mut century = 0;
		if let CMOSCenturyHandler::CenturyRegister(century_reg) = century_handler {
			century = self.read(century_reg);
		}

		let mut last_second;
		let mut last_minute;
		let mut last_hour;
		let mut last_day;
		let mut last_month;
		let mut last_year;
		let mut last_century;

		loop {
			last_second = rtc_time.second;
			last_minute = rtc_time.minute;
			last_hour = rtc_time.hour;
			last_day = rtc_time.day;
			last_month = rtc_time.month;
			last_year = rtc_time.year;
			last_century = century;

			self.read_into_rtc(&mut rtc_time);

			if last_second != rtc_time.second
				|| last_minute != rtc_time.minute
				|| last_hour != rtc_time.hour
				|| last_day != rtc_time.day
				|| last_month != rtc_time.month
				|| last_year != rtc_time.year
				|| last_century != century
			{
				break;
			}
		}

		let register_b = self.read(0x0B);

		// Convert BCD to binary values if necessary
		if (register_b & 0x04) == 0 {
			rtc_time.second = (rtc_time.second & 0x0F) + ((rtc_time.second / 16) * 10);
			rtc_time.minute = (rtc_time.minute & 0x0F) + ((rtc_time.minute / 16) * 10);
			rtc_time.hour = ((rtc_time.hour & 0x0F) + (((rtc_time.hour & 0x70) / 16) * 10)) | (rtc_time.hour & 0x80);
			rtc_time.day = (rtc_time.day & 0x0F) + ((rtc_time.day / 16) * 10);
			rtc_time.month = (rtc_time.month & 0x0F) + ((rtc_time.month / 16) * 10);
			rtc_time.year = (rtc_time.year & 0x0F) + ((rtc_time.year / 16) * 10);

			if let CMOSCenturyHandler::CenturyRegister(_) = century_handler {
				century = (century & 0x0F) + ((century / 16) * 10);
			}
		}

		// Convert 12 hour clock to 24 hour clock if necessary
		if ((register_b & 0x02) == 0) && ((rtc_time.hour & 0x80) != 0) {
			rtc_time.hour = ((rtc_time.hour & 0x7F) + 12) % 24;
		}

		// Calculate the full (4-digit) year
		match century_handler {
			CMOSCenturyHandler::CenturyRegister(_) => rtc_time.year += (century * 100) as usize,
			CMOSCenturyHandler::CurrentYear(current_year) => {
				rtc_time.year += (current_year / 100) * 100;
				if rtc_time.year < current_year {
					rtc_time.year += 100;
				}
			}
		}

		rtc_time
	}
	
        /// This method is similar to read_rtc however it waits until the RTC updates itself then
        /// returns the current time
        pub fn wait_for_update_then_read_rtc(&mut self , century_handler: CMOSCenturyHandler) -> RTCDateTime{
            // wait until to rtc updates itself
            while self.get_update_in_progress_flag() == 0 {}
            self.read_rtc(century_handler)
        }

}

/// Enum for determining how to calculate the year when reading the RTC
#[derive(Debug, Clone, Copy)]
pub enum CMOSCenturyHandler {
	/// This option is for providing the number of the century register in the RTC
	CenturyRegister(u8),
	/// This option is for providing the current year as a backup
	CurrentYear(usize),
}

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

pub const MAX: RTCDateTime = RTCDateTime { year: usize::MAX, month: 12, day: 31, hour: 23, minute: 59, second: 59 };

pub const MIN: RTCDateTime = RTCDateTime { year: 0, month: 1, day: 1, hour: 0, minute: 0, second: 0 };

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

impl RTCDateTime {
	/// Check if the `RTCDateTime` instance is a valid date.
	/// The function takes into account the number of days in months and leap years.
	#[inline]
	pub fn is_valid(&self) -> bool {
		self.month < 13
			&& self.hour < 24
			&& self.minute < 60
			&& self.second < 60
			&& self.day < RTCDateTime::days_by_month(self.year, self.month)
	}

	/// Attempt to create a valid `RTCDateTime` from a tuple.
	/// Returns `Some(RTCDateTime)` in case of success, or `None` if the operation failed.
	/// The datetime eventually returned is guaranteed to be valid.
	#[inline]
	pub fn from_tuple(tuple: &(usize, u8, u8, u8, u8, u8)) -> Option<Self> {
		let datetime = Self { year: tuple.0, month: tuple.1, day: tuple.2, hour: tuple.3, minute: tuple.4, second: tuple.5 };
		if datetime.is_valid() {
			Some(datetime)
		} else {
			None
		}
	}

	/// Returns a tuple containing the fields of a `RTCDateTime` by descending order.
	#[inline]
	pub fn as_tuple(&self) -> (usize, u8, u8, u8, u8, u8) {
		(self.year, self.month, self.day, self.hour, self.minute, self.second)
	}

	/// Returns the maximal number of days given a month and a year.
	#[doc(hidden)]
	fn days_by_month(year: usize, month: u8) -> u8 {
		match month {
			1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
			4 | 6 | 9 | 11 => 30,
			2 => {
				if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
					29
				} else {
					28
				}
			},
			_ => 0,
		}
	}
}
