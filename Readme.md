# CMOS
> A utility to read, write CMOS and RTC data. Standard library not required.

[![Linux build status](https://api.travis-ci.org/noahrinehart/cmos.svg)](https://travis-ci.org/noahrinehart/cmos)
[![Coverage Status](https://coveralls.io/repos/github/noahrinehart/cmos/badge.svg?branch=master)](https://coveralls.io/github/noahrinehart/cmos?branch=master)
[![crates.io](https://meritbadge.herokuapp.com/cmos)](https://crates.io/crates/cmos)

## Requirements
* nightly [rust](https://www.rust-lang.org/en-US/)
    * [rustup](https://rustup.rs/)

## Using the library (only tested on x86, nightly compiler required)
Add the crate to your project
```sh
# Cargo.toml
cmos = "0.1.0"
```

## Examples

To read the RTC using the century register.
```rust
use cmos::{CMOS, CMOSCenturyHandler};
// Create a CMOS object (unsafe due to the use of port I/O)
let mut cmos = unsafe { CMOS::new() };
// Read the rtc date time using this year
let rtc = cmos.read_rtc(CMOSCenturyHandler::CenturyRegister(32));
```

To read the RTC using the current year.
```rust
use cmos::{CMOS, CMOSCenturyHandler};
// Create a CMOS object (unsafe due to the use of port I/O)
let mut cmos = unsafe { CMOS::new() };
// Read the rtc date time using this year
let rtc = cmos.read_rtc(CMOSCenturyHandler::CurrentYear(2018));
```

Check the [docs](https://docs.rs/crate/cmos) for more information.

## Contributing
Feel free to contribute what you want. Just send in a pull request!

## License
MIT