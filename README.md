# db_ip

An (unofficial) library for querying [db-ip.com](https://db-ip.com/) CSV data in safe Rust.

This library is not affiliated with or endorsed by [db-ip.com](https://db-ip.com/).

## Examples

You can use `DbIp<CountryCode>` to get the actual two-letter country code (this takes more RAM to store).

```rust
use db_ip::{DbIp, CountryCode};

let db_ip = DbIp::<CountryCode>::from_csv_file("./test_data.csv").expect("you must download data.csv");

assert_eq!(
    db_ip.get_v4(&"0.1.2.3".parse().unwrap()),
    Some(CountryCode::from_str("US").unwrap())
);
```

You can use `DbIp<Region>`, enabled by the `region` feature, to gain a broad understanding of an IP's location.

```rust
use db_ip::{DbIp, Region};

let db_ip = DbIp::<Region>::from_csv_file("./test_data.csv").expect("you must download data.csv");

assert_eq!(
    db_ip.get_v4(&"0.1.2.3".parse().unwrap()),
    Some(Region::America)
);
```

Finally, you can implement `IpData` yourself, to store any other type of data that can be derived from Country or
City data records.

## Downloading IP Geolocation Data

You must visit [db-ip](https://db-ip.com/db/download/ip-to-country-lite) to download the actual ip geolocation data.

Make sure you select the Country (recommended) or City data in CSV format. The "lite" data is currently free, but subject to terms.

## Features

The raw csv data takes a while to parse, even in release mode. You may use
the `serde` feature to create and load a serialized version.

You can selectively disable the `ipv4` and `ipv6` features, depending on your needs. Both are
on by default.

Lookups are relatively speedy, taking less than 100ns in release mode.

## Limitations

If you want easier access to data other than `CountryCode` and `Region`, create an issue.

For now, the `region` feature will result in `serde` and `serde_json` being imported, due to [locale-codes](https://github.com/johnstonskj/locale-codes).

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.