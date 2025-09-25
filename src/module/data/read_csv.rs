use csv::{ReaderBuilder, StringRecord};
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, serde::Deserialize)]
struct CsvRow3 {
    timestamp: i64,
    volume: Option<u64>,
    close: f64,
}

pub fn read_close_series(csv_path: &PathBuf) -> Result<Vec<(i64, f64)>, Box<dyn Error>> {
    // Try headered 3-column file first: timestamp,volume,close
    let mut rdr = ReaderBuilder::new().trim(csv::Trim::All).from_path(csv_path)?;
    let mut rows: Vec<(i64, f64)> = Vec::new();
    let mut tried_headered = false;

    if let Ok(h) = rdr.headers() {
        let has_timestamp = h.iter().any(|x| x.eq_ignore_ascii_case("timestamp"));
        let has_close = h.iter().any(|x| x.eq_ignore_ascii_case("close"));
        if has_timestamp && has_close {
            tried_headered = true;
            for (i, r) in rdr.deserialize::<CsvRow3>().enumerate() {
                match r {
                    Ok(row) => rows.push((row.timestamp, row.close)),
                    Err(e) => {
                        if i < 5 {
                            eprintln!("CSV warn: skip row {}: {}", i + 1, e);
                        }
                    }
                }
            }
        }
    }

    if rows.is_empty() {
        // Fallback: no headers or different headers — assume 3 columns (ts, vol, close)
        if !tried_headered {
            // rebuild reader if consumed
            rdr = ReaderBuilder::new()
                .has_headers(false)
                .trim(csv::Trim::All)
                .from_path(csv_path)?;
        }

        for (i, rec) in rdr.records().enumerate() {
            let rec: StringRecord = match rec {
                Ok(r) => r,
                Err(e) => {
                    if i < 5 {
                        eprintln!("CSV warn: bad row {}: {}", i + 1, e);
                    }
                    continue;
                }
            };
            if rec.len() < 3 {
                continue;
            }
            // Skip header-like first row
            if i == 0 {
                let f0 = rec.get(0).unwrap_or("");
                if f0.chars().any(|c| c.is_alphabetic()) {
                    continue;
                }
            }
            let ts: i64 = match rec.get(0).and_then(|s| s.parse().ok()) {
                Some(v) => v,
                None => continue,
            };
            let close: f64 = match rec.get(2).and_then(|s| s.parse().ok()) {
                Some(v) => v,
                None => continue,
            };
            rows.push((ts, close));
        }
    }

    // Sort by timestamp just in case
    rows.sort_by_key(|(ts, _)| *ts);
    Ok(rows)
}

