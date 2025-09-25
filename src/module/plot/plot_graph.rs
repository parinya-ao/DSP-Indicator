use chrono::{DateTime, TimeZone, Utc};
use csv::{ReaderBuilder, StringRecord};
use plotters::prelude::*;
use serde::Deserialize;
use std::{error::Error, path::PathBuf};

#[derive(Debug, Deserialize)]
struct Quote {
    timestamp: i64,
    volume: u64,
    close: f64,
}

fn ts_to_datetime(ts: i64) -> Option<DateTime<Utc>> {
    // รองรับทั้งวินาทีและมิลลิวินาที
    let (secs, nanos) = if ts.abs() >= 1_000_000_000_000 {
        let secs = ts.div_euclid(1000);
        let rem_ms = ts.rem_euclid(1000) as u32;
        (secs, rem_ms * 1_000_000)
    } else {
        (ts, 0u32)
    };
    Utc.timestamp_opt(secs, nanos).single()
}

fn parse_points_headered(
    csv_path: &PathBuf,
) -> Result<(Vec<(DateTime<Utc>, f64)>, usize), Box<dyn Error>> {
    println!("Opening CSV (headered attempt)...");
    let mut rdr = ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(csv_path)?;

    // ถ้า header ไม่ตรงกับฟอร์แมต Yahoo ปกติ ให้ข้ามไป
    if let Ok(hdrs) = rdr.headers() {
        let has_timestamp = hdrs.iter().any(|h| h.eq_ignore_ascii_case("timestamp"));
        let has_open = hdrs.iter().any(|h| h.eq_ignore_ascii_case("open"));
        let has_close = hdrs.iter().any(|h| h.eq_ignore_ascii_case("close"));
        if !(has_timestamp && has_open && has_close) {
            println!(
                "Header not matching Yahoo format, skipping headered path: {:?}",
                hdrs
            );
            return Ok((Vec::new(), 0));
        }
    }

    let mut points: Vec<(DateTime<Utc>, f64)> = Vec::new();
    let (mut total, mut parsed) = (0usize, 0usize);

    for rec in rdr.deserialize::<Quote>() {
        total += 1;
        if total <= 5 {
            println!("Processing row {}: {:?}", total, rec);
        } else if total == 6 {
            println!("... (continuing with more rows)");
        }

        match rec {
            Ok(q) => {
                if total <= 3 {
                    println!(
                        "  Parsed quote: timestamp={}, close={}",
                        q.timestamp, q.close
                    );
                }
                if let Some(dt) = ts_to_datetime(q.timestamp) {
                    if q.close.is_finite() {
                        points.push((dt, q.close));
                        parsed += 1;
                        if parsed <= 3 {
                            println!("  ✓ Added point: {} -> {}", dt, q.close);
                        }
                    } else if total <= 3 {
                        println!("  ✗ Skipped: close price not finite");
                    }
                } else if total <= 3 {
                    println!("  ✗ Skipped: invalid timestamp {}", q.timestamp);
                }
            }
            Err(e) => {
                eprintln!("Warning: skip bad row {}: {}", total, e);
            }
        }
    }

    Ok((points, total))
}

fn parse_points_threecol(
    csv_path: &PathBuf,
) -> Result<(Vec<(DateTime<Utc>, f64)>, usize), Box<dyn Error>> {
    println!("Opening CSV (3-column fallback)...");
    let mut best_points: Vec<(DateTime<Utc>, f64)> = Vec::new();
    let mut best_total: usize = 0;

    // ทดลองหลาย delimiter เพื่อรองรับทั้ง comma / space / tab / semicolon
    for delim in [b',', b' ', b'\t', b';'] {
        println!("Trying delimiter: {:?}", delim as char);
        let mut rdr = ReaderBuilder::new()
            .has_headers(false)
            .trim(csv::Trim::All)
            .flexible(true)
            .delimiter(delim)
            .from_path(csv_path)?;

        let mut points: Vec<(DateTime<Utc>, f64)> = Vec::new();
        let (mut total, mut parsed) = (0usize, 0usize);

        for (idx, rec) in rdr.records().enumerate() {
            let rec: StringRecord = match rec {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Warning: bad row {}: {}", idx + 1, e);
                    continue;
                }
            };
            total += 1;

            if rec.len() < 3 {
                // อาจเป็นบรรทัดว่าง/คอมเมนต์
                continue;
            }

            // ตรวจจับ header แถวแรกแบบ manual (เช่น "timestamp volume closeprice")
            if idx == 0 {
                let f0 = rec.get(0).unwrap_or("");
                if f0.chars().any(|c| c.is_alphabetic()) {
                    println!("Detected header row, skipping it: {:?}", rec);
                    continue;
                }
            }

            let ts_str = rec.get(0).unwrap_or("");
            let close_str = rec.get(2).unwrap_or("");

            let ts: i64 = match ts_str.parse() {
                Ok(v) => v,
                Err(_) => {
                    eprintln!(
                        "Warning: invalid timestamp at row {}: {:?}",
                        idx + 1,
                        ts_str
                    );
                    continue;
                }
            };

            let close: f64 = match close_str.parse() {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("Warning: invalid close at row {}: {:?}", idx + 1, close_str);
                    continue;
                }
            };

            if let Some(dt) = ts_to_datetime(ts) {
                if close.is_finite() {
                    points.push((dt, close));
                    parsed += 1;
                    if parsed <= 3 {
                        println!("  ✓ Added point: {} -> {}", dt, close);
                    }
                }
            }

            if total > 200 && parsed == 0 {
                // เปลี่ยน delimiter
                break;
            }
        }

        if !points.is_empty() {
            best_points = points;
            best_total = total;
            break;
        } else if total > best_total {
            best_total = total;
        }
    }

    Ok((best_points, best_total))
}

pub fn plot_graph(csv_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    // 1) Working dir & input
    let cwd = std::env::current_dir()?;
    println!("Current dir: {:?}", cwd);

    println!("Reading CSV: {}", csv_path.display());

    // ตรวจสอบว่าไฟล์มีอยู่จริงไหม
    if !csv_path.exists() {
        eprintln!("Error: CSV file does not exist: {}", csv_path.display());
        eprintln!("Current directory contents:");
        if let Ok(entries) = std::fs::read_dir(&cwd) {
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("  {}", entry.file_name().to_string_lossy());
                }
            }
        }
        return Err("CSV file not found".into());
    }

    // ตรวจสอบขนาดไฟล์
    match std::fs::metadata(&csv_path) {
        // --- Font fallback handling -------------------------------------------------
        // plotters panics if the requested font family cannot be resolved on the system.
        // Allow overriding via env var PLOT_FONT_FAMILY and provide several common fallbacks.
        Ok(metadata) => {
            println!("CSV file size: {} bytes", metadata.len());
            if metadata.len() == 0 {
                eprintln!("Error: CSV file is empty");
                return Err("CSV file is empty".into());
            }
        }
        Err(e) => {
            eprintln!("Error reading CSV metadata: {}", e);
            return Err(e.into());
        }
    }

    // 2) Parse points (try headered struct first, then 3-col fallback)
    println!("CSV opened successfully, starting to parse...");
    let (mut points, total1) = match parse_points_headered(&csv_path) {
        Ok((pts, tot)) if !pts.is_empty() => {
            println!("Parsed using named headers (Yahoo style)");
            (pts, tot)
        }
        _ => {
            println!("Fallback to 3-column format (timestamp, volume, close)");
            parse_points_threecol(&csv_path)?
        }
    };

    println!("Rows read: {total1}, points parsed: {}", points.len());
    if points.is_empty() {
        eprintln!(
            "No valid data to plot → เช็ก header/หน่วย timestamp/ตัวคั่น (comma/space) ใน CSV อีกครั้งครับ"
        );
        return Ok(());
    }

    // 3) Sort & ranges
    points.sort_by_key(|(d, _)| *d);

    let mut x0 = points.first().unwrap().0;
    let mut x1 = points.last().unwrap().0;

    // ถ้ามีแค่จุดเดียว ให้ขยาย X ±1 นาที
    if x0 == x1 {
        x0 = x0 - chrono::Duration::minutes(1);
        x1 = x1 + chrono::Duration::minutes(1);
    }

    let (mut y_min, mut y_max) = points
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), &(_, v)| {
            (mn.min(v), mx.max(v))
        });

    if (y_max - y_min).abs() < std::f64::EPSILON {
        y_min -= 1.0;
        y_max += 1.0;
    } else {
        let pad = (y_max - y_min) * 0.05;
        y_min -= pad;
        y_max += pad;
    }

    // 4) Draw & save to absolute path (ใน working dir)
    let out_path = cwd.join("data/plot.png");
    println!("Saving to: {}", out_path.display());

    // ตรวจสอบว่า directory มีสิทธิ์เขียนไหม
    if let Some(parent) = out_path.parent() {
        if !parent.exists() {
            eprintln!(
                "Error: Parent directory does not exist: {}",
                parent.display()
            );
            return Err("Parent directory does not exist".into());
        }

        // ตรวจสอบสิทธิ์เขียน
        match std::fs::metadata(parent) {
            Ok(metadata) => {
                if metadata.permissions().readonly() {
                    eprintln!(
                        "Error: No write permission to directory: {}",
                        parent.display()
                    );
                    return Err("No write permission".into());
                }
            }
            Err(e) => {
                eprintln!("Warning: Cannot check directory permissions: {}", e);
            }
        }
    }

    // ลบไฟล์เก่าถ้ามี
    if out_path.exists() {
        match std::fs::remove_file(&out_path) {
            Ok(_) => println!("Removed existing file"),
            Err(e) => eprintln!("Warning: Cannot remove existing file: {}", e),
        }
    }

    println!("Creating BitMapBackend...");
    let root = BitMapBackend::new(&out_path, (1280, 720)).into_drawing_area();

    println!("Filling background...");
    root.fill(&WHITE)?;

    // --- Font fallback handling -------------------------------------------------
    // plotters panics if the requested font family cannot be resolved on the system.
    // Allow overriding via env var PLOT_FONT_FAMILY and provide several common fallbacks.
    let font_family = std::env::var("PLOT_FONT_FAMILY")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            // Ordered fallback list (first assumed available will be used for all text)
            // You can export PLOT_FONT_FAMILY to force a specific one.
            for cand in [
                "DejaVu Sans",     // very common on most Linux distros
                "Liberation Sans", // fallback family
                "Arial",           // common on many systems
                "Noto Sans",       // Google Noto
                "sans-serif",      // generic family (may still fail on minimal images)
            ] {
                return cand.to_string();
            }
            // Unreachable, but required by type inference
            "sans-serif".to_string()
        });
    println!("Using font family: {font_family}");

    println!("Building chart...");
    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption("Close Price", (font_family.as_str(), 28))
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(x0..x1, y_min..y_max)?;

    println!("Drawing mesh...");
    chart
        .configure_mesh()
        .x_labels(8)
        .x_label_formatter(&|d| d.format("%Y-%m").to_string())
        .label_style((font_family.as_str(), 14))
        .axis_desc_style((font_family.as_str(), 16))
        .y_desc("Price")
        .x_desc("Date (UTC)")
        .draw()?;

    println!("Drawing line series...");
    chart
        .draw_series(LineSeries::new(points.iter().map(|(d, v)| (*d, *v)), &BLUE))?
        .label("Close")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    println!("Drawing legend...");
    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .draw()?;

    // Force flush และ present
    println!("Flushing and presenting...");
    drop(chart); // ปิด chart ก่อน
    root.present()?; // flush
    drop(root); // ปิด backend

    // รอสักนิด
    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("Checking file existence...");
    if out_path.exists() {
        let metadata = std::fs::metadata(&out_path)?;
        println!("✓ File saved successfully!");
        println!("  Size: {} bytes", metadata.len());
        println!("  Path: {}", out_path.display());
    } else {
        eprintln!("✗ Error: File was not created!");
        return Err("File was not saved".into());
    } // Hint การเปิดดูไฟล์ (Linux/macOS)
    std::process::Command::new("xdg-open")
        .arg(&out_path)
        .spawn()
        .ok();
    // std::process::Command::new("open").arg(&out_path).spawn().ok();

    Ok(())
}
