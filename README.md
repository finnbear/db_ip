# db_ip

An (unofficial) library for querying [db-ip.com](https://db-ip.com/) CSV databases in safe Rust.

This library is not affiliated with or endorsed by [db-ip.com](https://db-ip.com/).

Be advised that, by using this library with lite databases (such as the one downloaded
automatically in the build step), you are subject [license terms](LICENSE-DBIP)
(requiring attribution).

## Examples

You can use `DbIpDatabase<CountryCode>` to get the actual two-letter country code. The country code database will be
embedded, in a compressed form, in your Rust binary.

```rust
use db_ip::{DbIpDatabase, CountryCode, include_country_code_database};

let db = include_country_code_database!();

assert_eq!(
    db.get(&"192.99.174.0".parse().unwrap()),
    Some(CountryCode::from_str("US").unwrap())
);
```

You can use `DbIpDatabase<Region>`, enabled by the `region` feature, to gain a broad understanding of an IP's location.
Since there are fewer possibilities, this takes less binary size and RAM.

```rust
use db_ip::{DbIpDatabase, Region, include_region_database};

let db = include_region_database!();

assert_eq!(
    db.get(&"192.99.174.0".parse().unwrap()),
    Some(Region::NorthAmerica)
);
```

Finally, you can implement `IpData` yourself, to store any other type of data that can be derived from Country or
City data records.

## Downloading IP Geolocation Data

You can manually download the actual ip geolocation data (in CSV format) in one of the following ways.

- Use the default `download-country-lite` feature, which attempts to download the most recent available Country lite data
- [Country data lite](https://db-ip.com/db/download/ip-to-country-lite) (recommended)
- [City data lite](https://db-ip.com/db/download/ip-to-city-lite) (larger file size)
- You may also try the paid database versions for better accuracy, but they have not been tested with this crate

Once you have downloaded a CSV file, use the `csv` feature to load it.

## Features

The raw csv data takes a while to parse, even in release mode. You may use
the `serde` feature to create and load a serialized version.

You can selectively disable the `ipv4` and `ipv6` features, depending on your needs. Both are
on by default.

Lookups are relatively speedy, taking less than 100ns in release mode.

## Limitations

If you want easier access to data other than `CountryCode` and `Region`, create an issue.

The [db-ip.com](https://db-ip.com/) API is not currently supported, so it is difficult to
keep the database up to date.

## License

Code licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Bundled/downloaded geolocation data licensed under [LICENSE-DBIP](LICENSE-DBIP).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.