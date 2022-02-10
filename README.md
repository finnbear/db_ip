# db_ip

An (unofficial) library for querying [db-ip.com](https://db-ip.com/) CSV data in safe Rust.

This library is not affiliated with or endorsed by [db-ip.com](https://db-ip.com/). It stores the
geolocation data in memory, rather than using any API.

## Examples

You can use `DbIp<CountryCode>` to get the actual two-letter country code (this takes more RAM to store).

```rust
use db_ip::{DbIp, CountryCode};

let db_ip = DbIp::<CountryCode>::from_csv_file("./test_country_data.csv").expect("you must download country_data.csv");

assert_eq!(
    db_ip.get_v4(&"0.1.2.3".parse().unwrap()),
    Some(CountryCode::from_str("US").unwrap())
);
```

You can use `DbIp<Region>`, enabled by the `region` feature, to gain a broad understanding of an IP's location.

```rust
use db_ip::{DbIp, Region};

let db_ip = DbIp::<Region>::from_csv_file("./test_country_data.csv").expect("you must download country_data.csv");

assert_eq!(
    db_ip.get_v4(&"0.1.2.3".parse().unwrap()),
    Some(Region::NorthAmerica)
);
```

Finally, you can implement `IpData` yourself, to store any other type of data that can be derived from Country or
City data records.

## Downloading IP Geolocation Data

You must visit one of the following links to download the actual ip geolocation data (in CSV format).

- [Country data lite](https://db-ip.com/db/download/ip-to-country-lite) (recommended)
- [City data lite](https://db-ip.com/db/download/ip-to-city-lite) (larger file size)
- You may also try the paid versions, but they have not been tested with this crate

## Features

The raw csv data takes a while to parse, even in release mode. You may use
the `serde` feature to create and load a serialized version.

You can selectively disable the `ipv4` and `ipv6` features, depending on your needs. Both are
on by default.

Lookups are relatively speedy, taking less than 100ns in release mode.

If you want to embed the data into your Rust binary, you can do so efficiently with:
```console
cargo run --package db_ip --bin export_region_bincode --release --features serde,bincode -- country_data.csv db_ip_region.bin
```

You can then use the following macro:
```rust
#[cfg(all(feature = "region", feature = "serde", feature = "bincode", feature = "ipv4", feature = "csv"))]
{
    use db_ip::{Region, include_db_ip_region_bincode};

    let db_ip = include_db_ip_region_bincode!("../db_ip_region.bin");

    assert_eq!(
        db_ip.get_v4(&"1.0.0.0".parse().unwrap()),
        Some(Region::Oceania)
    );
}
```

## Limitations

If you want easier access to data other than `CountryCode` and `Region`, create an issue.

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