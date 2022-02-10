use db_ip::{DbIp, Region};
use std::convert::TryInto;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Instant;

pub fn main() -> Result<(), String> {
    let start = Instant::now();
    let res: Result<[String; 2], _> = env::args().skip(1).collect::<Vec<_>>().try_into();
    match res {
        Err(_) => Err(format!(
            "expected two arguments, input path followed by output path"
        )),
        Ok([input, output]) => match DbIp::<Region>::from_csv_file(&input) {
            Err(e) => Err(format!("error: {:?}", e)),
            Ok(db_ip) => {
                let ser = bincode::serialize(&db_ip).unwrap();

                match OpenOptions::new().create(true).write(true).open(output) {
                    Err(e) => Err(format!("could not open output file for writing: {:?}", e)),
                    Ok(mut f) => {
                        if let Err(e) = f.write_all(&ser) {
                            Err(format!("error writing to output file: {:?}", e))
                        } else {
                            println!(
                                "created output file of size {:?} in {:?}",
                                ser.len(),
                                start.elapsed()
                            );
                            Ok(())
                        }
                    }
                }
            }
        },
    }
}
