#![forbid(unsafe_code)]
#![cfg_attr(all(test, feature = "nightly"), feature(test))]

pub use db_ip_core::*;

#[cfg(feature = "bincode")]
#[doc(hidden)]
pub use bincode;

#[cfg(feature = "include-region-lite")]
#[doc(hidden)]
pub const REGION_LITE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/region_lite.bin"));

#[cfg(feature = "include-country-code-lite")]
#[doc(hidden)]
pub const COUNTRY_CODE_LITE_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/country_code_lite.bin"));

#[macro_export]
#[cfg(feature = "include-region-lite")]
macro_rules! include_region_database {
    () => {{
        let db: $crate::DbIpDatabase<$crate::Region> =
            $crate::bincode::deserialize($crate::REGION_LITE_BYTES).unwrap();
        db
    }};
}

#[macro_export]
#[cfg(feature = "include-country-code-lite")]
macro_rules! include_country_code_database {
    () => {{
        let db: $crate::DbIpDatabase<$crate::CountryCode> =
            $crate::bincode::deserialize($crate::COUNTRY_CODE_LITE_BYTES).unwrap();
        db
    }};
}

#[cfg(test)]
#[cfg(any(feature = "ipv4", feature = "ipv6"))]
mod test {
    #[cfg(feature = "nightly")]
    extern crate test;
    #[allow(unused_imports)]
    use crate::{CountryCode, DbIpDatabase};

    #[test]
    fn country_code() {
        assert_eq!(CountryCode::from_str("US"), CountryCode::from_str("us"));
    }

    #[test]
    #[cfg(all(feature = "ipv4", feature = "include-region-lite"))]
    fn region_v4() {
        use crate::Region;

        let db = include_region_database!();
        assert_eq!(
            db.get_v4(&"94.250.200.0".parse().unwrap()),
            Some(Region::NorthAmerica)
        );
    }

    #[test]
    #[cfg(all(feature = "ipv4", feature = "include-country-code-lite"))]
    fn country_code_v4() {
        let db = include_country_code_database!();
        println!("country code length v4: {}", db.len_v4());
        assert_eq!(
            db.get_v4(&"94.250.200.0".parse().unwrap()),
            Some(CountryCode::from_str("US").unwrap())
        );
    }

    #[test]
    #[cfg(all(feature = "ipv4", feature = "include-country-code-lite"))]
    fn city_country_code_v4() {
        let db = include_country_code_database!();
        println!("city country code length v4: {}", db.len_v4());
        assert_eq!(
            db.get_v4(&"94.250.200.0".parse().unwrap()),
            Some(CountryCode::from_str("US").unwrap())
        );
    }

    #[test]
    #[cfg(all(feature = "ipv6", feature = "include-region-lite"))]
    fn region_v6() {
        use crate::Region;

        let db = crate::include_region_database!();
        assert_eq!(
            db.get_v6(&"2a07:7ec5:77a1::".parse().unwrap()),
            Some(Region::Europe)
        );
    }

    // cargo bench --features nightly  -- bench_region_v4
    #[allow(soft_unstable)]
    #[cfg(all(feature = "nightly", feature = "embed-country-code-lite"))]
    #[bench]
    fn bench_region_v4(b: &mut test::Bencher) {
        use crate::Region;
        use std::net::Ipv4Addr;

        let db = crate::include_region_code_lite!();
        let mut i = 0u32;

        b.iter(|| {
            test::black_box(db.get_v4(&Ipv4Addr::from(i.to_be_bytes())));
            i = i.wrapping_add(1).wrapping_mul(7);
        });
    }

    // cargo bench --features nightly  -- bench_region_v6
    #[allow(soft_unstable)]
    #[cfg(all(
        feature = "nightly",
        feature = "csv",
        feature = "download-country-lite"
    ))]
    #[bench]
    fn bench_region_v6(b: &mut test::Bencher) {
        use std::net::Ipv6Addr;

        let db = crate::include_region_database!();
        let mut i = 0u128;

        b.iter(|| {
            test::black_box(db.get_v6(&Ipv6Addr::from(i.to_be_bytes())));
            i = i.wrapping_add(1).wrapping_mul(7);
        });
    }
}

doc_comment::doctest!("../README.md");
