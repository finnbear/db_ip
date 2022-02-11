fn main() -> Result<(), &'static str> {
    #[cfg(feature = "download-country-lite")]
    {
        fn download_region_lite(output_path: &str) -> Result<(), &'static str> {
            use chrono::{Datelike, Duration, TimeZone, Utc};
            use flate2::bufread::GzDecoder;
            use std::fs;
            use std::fs::File;
            use std::io;
            use std::io::BufReader;
            use std::ops::Sub;
            use std::time::SystemTime;

            for i in 0..12 {
                let date = Utc::now().date().sub(
                    Duration::from_std(std::time::Duration::from_secs(i * 31 * 24 * 3600)).unwrap(),
                );

                if download_lite(date.year(), date.month(), output_path).is_ok() {
                    return Ok(());
                }
            }

            if download_lite(2022, 2, output_path).is_ok() {
                return Ok(());
            }

            return Err("db_ip could not download country lite database");

            fn download_lite(
                year: i32,
                month: u32,
                output_path: &str,
            ) -> Result<bool, &'static str> {
                let url = format!(
                    "https://download.db-ip.com/free/dbip-country-lite-{}-{:02}.csv.gz",
                    year, month
                );
                let expiry = Utc.ymd(year, month, 1).and_hms(0, 0, 0);
                let res = download_file(&url, output_path, Some(SystemTime::from(expiry)));
                match res {
                    Ok(downloaded) => {
                        if downloaded {
                            println!("cargo:warning=db_ip downloaded {} (please read the db-ip.com license terms!)", url);
                        } else {
                            println!(
                                "cargo:warning=db_ip skipped download, already up to date with {}",
                                url
                            );
                        }
                    }
                    Err(e) => println!(
                        "cargo:warning=db_ip error downloading {} database: {:?}",
                        expiry, e
                    ),
                }
                res
            }

            fn download_file(
                url: &str,
                path: &str,
                expiry: Option<SystemTime>,
            ) -> Result<bool, &'static str> {
                if expiry
                    .and_then(|e| fs::metadata(path).ok().map(|md| (e, md)))
                    .and_then(|(e, md)| md.modified().ok().map(|st| (e, st)))
                    .map(|(e, st)| st <= e)
                    .unwrap_or(true)
                {
                    let mut resp = reqwest::blocking::get(url).map_err(|_| "request failed")?;
                    let mut out = File::create(path).map_err(|_| "failed to create file")?;
                    let mut decoded = GzDecoder::new(BufReader::new(&mut resp));
                    io::copy(&mut decoded, &mut out)
                        .map(|_| true)
                        .map_err(|_| "failed to copy content")
                } else {
                    Ok(false)
                }
            }
        }

        use std::env;
        let csv_path = format!("{}/country_lite.csv", env::var("OUT_DIR").unwrap());
        if download_region_lite(&csv_path).is_ok() {
            #[cfg(any(feature = "include-region-lite", feature = "include-country-code-lite"))]
            fn compress_lite<V: db_ip_core::IpData + serde::Serialize>(
                csv_path: &str,
                region_path: &str,
            ) -> Result<(), String> {
                use std::fs::OpenOptions;
                use std::io::Write;

                match db_ip_core::DbIpDatabase::<V>::from_csv_file(csv_path) {
                    Err(e) => Err(format!("error: {:?}", e)),
                    Ok(db_ip) => {
                        let ser = bincode::serialize(&db_ip).unwrap();

                        match OpenOptions::new()
                            .create(true)
                            .write(true)
                            .open(region_path)
                        {
                            Err(e) => {
                                Err(format!("could not open output file for writing: {:?}", e))
                            }
                            Ok(mut f) => {
                                if let Err(e) = f.write_all(&ser) {
                                    Err(format!("error writing to output file: {:?}", e))
                                } else {
                                    Ok(())
                                }
                            }
                        }
                    }
                }
            }

            #[cfg(feature = "include-region-lite")]
            {
                let region_path = format!("{}/region_lite.bin", env::var("OUT_DIR").unwrap());
                if let Err(e) = compress_lite::<db_ip_core::Region>(&csv_path, &region_path) {
                    println!("cargo:warning=db_ip error embedding region: {:?}", e);
                }
            }

            #[cfg(feature = "include-country-code-lite")]
            {
                let country_code_path =
                    format!("{}/country_code_lite.bin", env::var("OUT_DIR").unwrap());
                if let Err(e) =
                    compress_lite::<db_ip_core::CountryCode>(&csv_path, &country_code_path)
                {
                    println!("cargo:warning=db_ip error embedding country code: {:?}", e);
                }
            }
        }
    }

    #[allow(unreachable_code)]
    Ok(())
}
