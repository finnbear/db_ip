fn main() -> Result<(), &'static str> {
    #[cfg(feature = "download-country-lite")]
    {
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

            if download_lite(date.year(), date.month()).is_ok() {
                return Ok(());
            }
        }

        if download_lite(2022, 2).is_ok() {
            return Ok(());
        }

        return Err("db_ip could not download country lite database");

        fn download_lite(year: i32, month: u32) -> Result<bool, &'static str> {
            let url = format!(
                "https://download.db-ip.com/free/dbip-country-lite-{}-{:02}.csv.gz",
                year, month
            );
            let expiry = Utc.ymd(year, month, 1).and_hms(0, 0, 0);
            let res = download_file(&url, "country_lite.csv", Some(SystemTime::from(expiry)));
            match res {
                Ok(downloaded) => {
                    if downloaded {
                        println!("cargo:warning=db_ip downloaded {}", url);
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

    #[allow(unreachable_code)]
    Ok(())
}
