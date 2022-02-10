#![forbid(unsafe_code)]
#![cfg_attr(all(test, feature = "nightly"), feature(test))]

use std::convert::TryInto;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
#[allow(unused_imports)]
use std::io::Read;
#[allow(unused_imports)]
use std::net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr};
#[allow(unused_imports)]
use std::str::FromStr;

/// A map of ip range to data derived from a country code.
#[derive(Debug)]
#[cfg(any(feature = "ipv4", feature = "ipv6"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DbIpDatabase<V> {
    #[cfg(feature = "ipv4")]
    v4: DbIpDatabaseInner<u32, V>,
    #[cfg(feature = "ipv6")]
    v6: DbIpDatabaseInner<u128, V>,
}

/// Errors that may arise when loading a [`DbIpDatabase`] from CSV.
#[derive(Debug)]
#[cfg(feature = "csv")]
#[non_exhaustive]
pub enum FromCsvError {
    /// An address range must start and end as either Ipv4 or Ipv6.
    AddrMismatch,
    /// Address ranges must be in ascending order. This will be raised if they aren't.
    AddrOutOfOrder,
    /// Failed to parse an address.
    AddrParse(AddrParseError),
    /// CSV error.
    Csv(csv::Error),
    /// CSV record was missing required data.
    InvalidRecord,
}

/// Data associated with an ip address, derived from a [`CountryCode`].
///
/// In general, the fewer the possibilities, the more compressed the data structure will be. For
/// Example, if you mapped a country code to a boolean, the data structure will store very large
/// ranges of true/false, consisting of multiple consecutive ranges in the original dataset.
pub trait IpData: Copy + Clone + PartialEq {
    /// db-ip data consists of csv records, any data must be derived from then.
    /// Should return [`Err(Error::InvalidRecord)`] if the fields are insufficient and the loading should
    /// be aborted, and [`Ok(None)`] if the field is fine, but the data is irrelevant.
    ///
    /// # Notes
    ///
    /// - The first two indices are the begin and end of the ip range, respectively.
    /// - You don't have to implement this if you disable the `csv` feature.
    /// - If you do implement it, you are responsible for knowing which indices correspond to which data.
    #[cfg(feature = "csv")]
    fn from_record(record: &csv::StringRecord) -> Result<Option<Self>, FromCsvError>;
}

/// A two letter, uppercase country code.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct CountryCode([u8; 2]);

impl CountryCode {
    /// Expects a two ASCII character country code. Will automatically upper-case.
    pub fn from_str(country_code_str: &str) -> Option<Self> {
        let bytes: [u8; 2] = country_code_str.as_bytes().try_into().ok()?;
        Self::from_bytes(bytes)
    }

    /// Returns an equivalent string e.g. `"US"` or `"AU"`.
    pub fn as_str(&self) -> &str {
        // We only ever put valid Utf8 bytes in.
        std::str::from_utf8(&self.0).unwrap()
    }

    pub(crate) fn from_bytes(mut bytes: [u8; 2]) -> Option<Self> {
        if std::str::from_utf8(&bytes).is_ok() {
            for byte in &mut bytes {
                *byte = byte.to_ascii_uppercase();
            }
            Some(Self(bytes))
        } else {
            // Invalid Utf8.
            None
        }
    }
}

impl Debug for CountryCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Display for CountryCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl IpData for CountryCode {
    #[cfg(feature = "csv")]
    fn from_record(record: &csv::StringRecord) -> Result<Option<Self>, FromCsvError> {
        let idx = match record.len() {
            // Country data
            3 => 2,
            // City data
            8 => 3,
            // Not present.
            _ => return Err(FromCsvError::InvalidRecord),
        };
        let country_code_str = record.get(idx).ok_or(FromCsvError::InvalidRecord)?;
        let country_code =
            CountryCode::from_str(country_code_str).ok_or(FromCsvError::InvalidRecord)?;
        Ok(Some(country_code))
    }
}

/// A very broad region id, useful for high-level operations. Roughly corresponds to populated
/// continents.
#[cfg(feature = "region")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Region {
    Africa,
    Asia,
    Europe,
    NorthAmerica,
    Oceania,
    SouthAmerica,
}

#[cfg(feature = "region")]
impl IpData for Region {
    #[cfg(feature = "csv")]
    fn from_record(record: &csv::StringRecord) -> Result<Option<Self>, FromCsvError> {
        let country_code = CountryCode::from_record(record)?;
        Ok(country_code.and_then(|cc| db_ip_macros::country_code_str_to_region!(cc.as_str())))
    }
}

#[cfg(any(feature = "ipv4", feature = "ipv6"))]
impl<V: IpData> DbIpDatabase<V> {
    /// Gets the value associated with an ip address, if any.
    #[cfg(all(feature = "ipv4", feature = "ipv6"))]
    pub fn get(&self, ip: &IpAddr) -> Option<V> {
        match ip {
            IpAddr::V4(v4) => self.get_v4(v4),
            IpAddr::V6(v6) => self.get_v6(v6),
        }
    }

    /// Gets the value associated with an Ipv4 address, if any.
    #[cfg(feature = "ipv4")]
    pub fn get_v4(&self, v4: &Ipv4Addr) -> Option<V> {
        self.v4.lookup(&ip_v4_to_ne(v4))
    }

    /// Gets the value associated with an Ipv6 address, if any.
    #[cfg(feature = "ipv6")]
    pub fn get_v6(&self, v6: &Ipv6Addr) -> Option<V> {
        self.v6.lookup(&ip_v6_to_ne(v6))
    }

    /// Returns number of ranges/values stored for both Ipv4 and Ipv6 addresses.
    #[cfg(all(feature = "ipv4", feature = "ipv6"))]
    pub fn len(&self) -> usize {
        self.len_v4() + self.len_v6()
    }

    /// Returns number of ranges/values stored for Ipv4 addresses.
    #[cfg(feature = "ipv4")]
    pub fn len_v4(&self) -> usize {
        self.v4.len()
    }

    /// Returns number of ranges/values stored for Ipv6 addresses.
    #[cfg(feature = "ipv6")]
    pub fn len_v6(&self) -> usize {
        self.v6.len()
    }

    /// Load from CSV file contained in string.
    #[cfg(feature = "csv")]
    pub fn from_csv_str(csv: &str) -> Result<Self, FromCsvError> {
        Self::from_csv_reader(csv.as_bytes())
    }

    /// Load from CSV file reader.
    #[cfg(feature = "csv")]
    pub fn from_csv_reader<R: Read>(reader: R) -> Result<Self, FromCsvError> {
        let reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(reader);

        Self::from_csv_reader_inner(reader)
    }

    /// Load from CSV file contained in file.
    #[cfg(feature = "csv")]
    pub fn from_csv_file(path: &str) -> Result<Self, FromCsvError> {
        let reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)
            .map_err(FromCsvError::Csv)?;

        Self::from_csv_reader_inner(reader)
    }

    #[cfg(feature = "csv")]
    fn from_csv_reader_inner<R: Read>(mut reader: csv::Reader<R>) -> Result<Self, FromCsvError> {
        #[cfg(feature = "ipv4")]
        let mut v4 = DbIpDatabaseInnerBuilder::new();
        #[cfg(feature = "ipv6")]
        let mut v6 = DbIpDatabaseInnerBuilder::new();

        for record in reader.records() {
            let record = record.map_err(FromCsvError::Csv)?;

            if let Some(value) = V::from_record(&record)? {
                let begin = IpAddr::from_str(&record[0]).map_err(FromCsvError::AddrParse)?;
                let end = IpAddr::from_str(&record[1]).map_err(FromCsvError::AddrParse)?;

                if begin.is_ipv4() != end.is_ipv4() {
                    return Err(FromCsvError::AddrMismatch);
                }

                match (begin, end) {
                    #[cfg(feature = "ipv4")]
                    (IpAddr::V4(begin), IpAddr::V4(end)) => {
                        let begin_ne = ip_v4_to_ne(&begin);
                        let end_ne = ip_v4_to_ne(&end);

                        v4.push(begin_ne, end_ne, end_ne.checked_add(1), value)?;
                    }
                    #[cfg(feature = "ipv6")]
                    (IpAddr::V6(begin), IpAddr::V6(end)) => {
                        let begin_ne = ip_v6_to_ne(&begin);
                        let end_ne = ip_v6_to_ne(&end);

                        v6.push(begin_ne, end_ne, end_ne.checked_add(1), value)?;
                    }
                    _ => {}
                }
            }
        }

        Ok(Self {
            #[cfg(feature = "ipv4")]
            v4: v4.inner,
            #[cfg(feature = "ipv6")]
            v6: v6.inner,
        })
    }
}

/// Stores either Ipv4 or Ipv6 addresses/values.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct DbIpDatabaseInner<IP, V> {
    /// Sorted address range starts, in native endian.
    starts: Vec<IP>,
    /// Value associated with each address range.
    values: Vec<Option<V>>,
}

impl<IP: Ord + Copy, V: IpData> DbIpDatabaseInner<IP, V> {
    #[cfg(feature = "csv")]
    fn new() -> Self {
        Self {
            starts: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Lookup value associated with native endian IP address, based on range.
    fn lookup(&self, ip: &IP) -> Option<V> {
        debug_assert_eq!(self.starts.len(), self.values.len());

        match self.starts.binary_search(ip) {
            Ok(idx) => self.values[idx],
            Err(idx) => {
                if self.starts.get(idx).map(|end| ip < end).unwrap_or(true) {
                    self.values.get(idx - 1).copied().unwrap_or(None)
                } else {
                    None
                }
            }
        }
    }

    /// How many IP ranges.
    fn len(&self) -> usize {
        self.values.len()
    }
}

/// Helps build [`DbIpDatabaseInner`] from sorted CSV data.
#[cfg(feature = "csv")]
struct DbIpDatabaseInnerBuilder<IP, V> {
    inner: DbIpDatabaseInner<IP, V>,
    next: IP,
    done: bool,
}

#[cfg(feature = "csv")]
impl<IP: Ord + Copy + Default, V: IpData> DbIpDatabaseInnerBuilder<IP, V> {
    pub fn new() -> Self {
        Self {
            inner: DbIpDatabaseInner::new(),
            next: IP::default(),
            done: false,
        }
    }

    /// Adds one IP range.
    pub fn push(
        &mut self,
        start: IP,
        end: IP,
        end_plus_one: Option<IP>,
        value: V,
    ) -> Result<(), FromCsvError> {
        if self.done {
            return Err(FromCsvError::AddrOutOfOrder);
        }

        if start < self.next || start > end {
            return Err(FromCsvError::AddrOutOfOrder);
        }
        if self
            .inner
            .values
            .last()
            .map(|last| last != &Some(value))
            .unwrap_or(true)
        {
            if start > self.next {
                // Gap of unknown values.
                self.inner.starts.push(self.next);
                self.inner.values.push(None);
                self.next = start;
            }
            self.inner.starts.push(start);
            self.inner.values.push(Some(value));
        }
        if let Some(nxt) = end_plus_one {
            self.next = nxt;
        } else {
            self.done = true;
        }
        Ok(())
    }
}

#[cfg(feature = "ipv4")]
pub(crate) fn ip_v4_to_ne(v4: &Ipv4Addr) -> u32 {
    u32::from_be_bytes(v4.octets())
}

#[cfg(feature = "ipv6")]
pub(crate) fn ip_v6_to_ne(v6: &Ipv6Addr) -> u128 {
    u128::from_be_bytes(v6.octets())
}

#[cfg(feature = "serde")]
impl serde::Serialize for CountryCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(self.as_str())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for CountryCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(CountryCodeStrVisitor)
        } else {
            deserializer.deserialize_bytes(CountryCodeBytesVisitor)
        }
    }
}

#[cfg(feature = "serde")]
pub struct CountryCodeStrVisitor;

#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for CountryCodeStrVisitor {
    type Value = CountryCode;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a two letter country code")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        CountryCode::from_str(value).ok_or(serde::de::Error::custom("invalid country code"))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        CountryCode::from_str(&value).ok_or(serde::de::Error::custom("invalid country code"))
    }
}

#[cfg(feature = "serde")]
pub struct CountryCodeBytesVisitor;

#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for CountryCodeBytesVisitor {
    type Value = CountryCode;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a two letter country code as bytes")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let bytes: [u8; 2] = value
            .try_into()
            .map_err(|_| serde::de::Error::custom("expected 2 country code bytes"))?;
        CountryCode::from_bytes(bytes).ok_or(serde::de::Error::custom("invalid country code bytes"))
    }
}

/*
impl Region {
    fn to_u8(self) -> u8 {
        match self {
            Region::Africa => 0,
            Region::NorthAmerica => 1,
            Region::Asia => 2,
            Region::Europe => 3,
            Region::Oceania => 4,
        }
    }

    fn from_u8(num: u8) -> Self {
        match num {
            0 => Region::Africa,
            1 => Region::NorthAmerica,
            2 => Region::Asia,
            3 => Region::Europe,
            4 => Region::Oceania,
            _ => panic!("invalid Region u8"),
        }
    }
}

#[cfg(any(feature = "ipv4", feature = "ipv6"))]
fn ip_to_bytes(ip: IpAddr) -> Vec<u8> {
    match ip {
        IpAddr::V4(v4) => v4.octets().to_vec(),
        IpAddr::V6(v6) => v6.octets().to_vec(),
    }
}
 */

#[cfg(all(feature = "serde", feature = "bincode", feature = "region"))]
#[doc(hidden)]
pub use bincode;

#[macro_export]
#[cfg(all(feature = "serde", feature = "bincode", feature = "region"))]
macro_rules! include_db_ip_region_bincode {
    ($path: expr) => {{
        let db_ip: ::db_ip::DbIpDatabase<::db_ip::Region> =
            ::db_ip::bincode::deserialize(include_bytes!($path)).unwrap();
        db_ip
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
    #[cfg(all(feature = "region", feature = "ipv4", feature = "csv"))]
    fn region_v4() {
        use crate::Region;

        if let Ok(db_ip) = DbIpDatabase::<Region>::from_csv_file("./country_data.csv") {
            /*
            let ser = bincode::serialize(&db_ip).unwrap();
            println!("{} {} {}", db_ip.len_v4(), db_ip.len_v6(), ser.len());
             */

            assert_eq!(
                db_ip.get_v4(&"94.250.200.0".parse().unwrap()),
                Some(Region::NorthAmerica)
            );
        } else {
            println!("Warning: create country_data.csv to run all test.");
        }
    }

    #[test]
    #[cfg(all(feature = "ipv4", feature = "csv"))]
    fn country_code_v4() {
        if let Ok(db_ip) = DbIpDatabase::<CountryCode>::from_csv_file("./country_data.csv") {
            println!("country code length v4: {}", db_ip.len_v4());
            assert_eq!(
                db_ip.get_v4(&"94.250.200.0".parse().unwrap()),
                Some(CountryCode::from_str("US").unwrap())
            );
        } else {
            println!("Warning: create country_data.csv to run all test.");
        }
    }

    #[test]
    #[cfg(all(feature = "ipv4", feature = "csv"))]
    fn city_country_code_v4() {
        if let Ok(db_ip) = DbIpDatabase::<CountryCode>::from_csv_file("./city_data.csv") {
            println!("city country code length v4: {}", db_ip.len_v4());
            assert_eq!(
                db_ip.get_v4(&"94.250.200.0".parse().unwrap()),
                Some(CountryCode::from_str("US").unwrap())
            );
        } else {
            println!("Warning: create country_data.csv to run all test.");
        }
    }

    #[test]
    #[cfg(all(feature = "region", feature = "ipv6", feature = "csv"))]
    fn region_v6() {
        use crate::Region;

        if let Ok(db_ip) = DbIpDatabase::<Region>::from_csv_file("./country_data.csv") {
            assert_eq!(
                db_ip.get_v6(&"2a07:7ec5:77a1::".parse().unwrap()),
                Some(Region::Europe)
            );
        } else {
            println!("Warning: create country_data.csv to run all test.");
        }
    }

    #[test]
    #[cfg(feature = "ipv4")]
    fn compare_v4() {
        use crate::ip_v4_to_ne;
        assert!(
            ip_v4_to_ne(&"127.0.0.1".parse().unwrap()) < ip_v4_to_ne(&"127.0.0.2".parse().unwrap())
        );
        assert!(
            ip_v4_to_ne(&"128.0.0.1".parse().unwrap()) > ip_v4_to_ne(&"127.0.0.2".parse().unwrap())
        );
    }

    #[test]
    #[cfg(feature = "ipv4")]
    fn add_v4() {
        use crate::ip_v4_to_ne;
        assert_eq!(
            ip_v4_to_ne(&"127.0.0.1".parse().unwrap()) + 1,
            ip_v4_to_ne(&"127.0.0.2".parse().unwrap())
        );
        assert_eq!(
            ip_v4_to_ne(&"127.0.0.1".parse().unwrap()) + 256,
            ip_v4_to_ne(&"127.0.1.1".parse().unwrap())
        );
    }

    #[test]
    #[cfg(all(
        feature = "region",
        feature = "serde",
        feature = "bincode",
        feature = "ipv4",
        feature = "csv"
    ))]
    fn region_serde_bincode() {
        use crate::Region;

        let db_ip = DbIpDatabase::<Region>::from_csv_file("./test_country_data.csv").unwrap();

        let ser = bincode::serialize(&db_ip).unwrap();
        println!("region serde bincode size {}: {:?}", ser.len(), ser);
        let de: DbIpDatabase<Region> = bincode::deserialize(&ser).unwrap();

        assert_eq!(
            de.get_v4(&"1.0.0.0".parse().unwrap()),
            Some(Region::Oceania)
        );
    }

    #[test]
    #[cfg(all(
        feature = "serde",
        feature = "bincode",
        feature = "ipv4",
        feature = "csv"
    ))]
    fn country_code_serde_bincode() {
        let db_ip = DbIpDatabase::<CountryCode>::from_csv_file("./test_country_data.csv").unwrap();

        let ser = bincode::serialize(&db_ip).unwrap();
        println!("country code serde bincode size {}: {:?}", ser.len(), ser);
        let de: DbIpDatabase<CountryCode> = bincode::deserialize(&ser).unwrap();

        assert_eq!(
            de.get_v4(&"1.0.0.0".parse().unwrap()),
            Some(CountryCode::from_str("AU").unwrap())
        );
    }

    #[test]
    #[cfg(all(
        feature = "region",
        feature = "serde",
        feature = "ipv4",
        feature = "csv"
    ))]
    fn region_serde_json_v4() {
        use crate::Region;

        let db_ip = DbIpDatabase::<Region>::from_csv_file("./test_country_data.csv").unwrap();

        let ser = serde_json::to_string(&db_ip).unwrap();
        println!("region serde json size {}: {}", ser.len(), ser);
        let de: DbIpDatabase<Region> = serde_json::from_str(&ser).unwrap();

        assert_eq!(
            de.get_v4(&"1.0.0.0".parse().unwrap()),
            Some(Region::Oceania)
        );
    }

    #[test]
    #[cfg(all(feature = "serde", feature = "ipv4", feature = "csv"))]
    fn country_code_serde_json_v4() {
        let db_ip = DbIpDatabase::<CountryCode>::from_csv_file("./test_country_data.csv").unwrap();

        let ser = serde_json::to_string(&db_ip).unwrap();
        println!("country code serde json size {}: {}", ser.len(), ser);
        let de: DbIpDatabase<CountryCode> = serde_json::from_str(&ser).unwrap();

        assert_eq!(
            de.get_v4(&"1.0.0.0".parse().unwrap()),
            Some(CountryCode::from_str("AU").unwrap())
        );
    }

    // cargo bench --features nightly  -- bench_region_v4
    #[allow(soft_unstable)]
    #[cfg(all(feature = "nightly", feature = "csv"))]
    #[bench]
    fn bench_region_v4(b: &mut test::Bencher) {
        use crate::Region;
        use std::net::Ipv4Addr;

        if let Ok(db_ip) = DbIpDatabase::<Region>::from_csv_file("./country_data.csv") {
            let mut i = 0u32;

            b.iter(|| {
                test::black_box(db_ip.get_v4(&Ipv4Addr::from(i.to_be_bytes())));
                i = i.wrapping_add(1).wrapping_mul(7);
            });
        } else {
            println!("Warning: create country_data.csv to run all benchmarks.");
        }
    }

    // cargo bench --features nightly  -- bench_region_v6
    #[allow(soft_unstable)]
    #[cfg(all(feature = "nightly", feature = "csv"))]
    #[bench]
    fn bench_region_v6(b: &mut test::Bencher) {
        use crate::Region;
        use std::net::Ipv6Addr;

        if let Ok(db_ip) = DbIpDatabase::<Region>::from_csv_file("./country_data.csv") {
            let mut i = 0u128;

            b.iter(|| {
                test::black_box(db_ip.get_v6(&Ipv6Addr::from(i.to_be_bytes())));
                i = i.wrapping_add(1).wrapping_mul(7);
            });
        } else {
            println!("Warning: create country_data.csv to run all benchmarks.");
        }
    }
}

doc_comment::doctest!("../README.md");
