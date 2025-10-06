use chrono::{TimeZone, Utc};
use yahoo::Quote;
use yahoo_finance_api as yahoo;
use yahoo_finance_api::time::OffsetDateTime;

pub async fn fetch_data(symbol: &str) -> Result<Vec<(u64, u64, f64)>, Box<dyn std::error::Error>> {
    let provider = yahoo::YahooConnector::new();

    // setup time
    let start_cron = match Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0) {
        chrono::LocalResult::Single(t) => t,
        _ => return Err("invalid start date".into()),
    };
    // Extend the range to cover the desired latest trading day (2025-10-05)
    let now_cron = match Utc.with_ymd_and_hms(2025, 10, 5, 0, 0, 0) {
        chrono::LocalResult::Single(t) => t,
        _ => return Err("invalid end date".into()),
    };
    // offset date time
    let start = match OffsetDateTime::from_unix_timestamp(start_cron.timestamp()) {
        Ok(v) => v,
        _ => return Err("invalid start time".into()),
    };
    let end = match OffsetDateTime::from_unix_timestamp(now_cron.timestamp()) {
        Ok(v) => v,
        _ => return Err("invalid end time".into()),
    };

    // time response
    let response = match provider?
        .get_quote_history_interval(symbol, start, end, "1d")
        .await
    {
        Ok(resp) => resp,
        Err(e) => return Err(Box::new(e)),
    };

    let quotes = match response.quotes() {
        Ok(quotes) => quotes,
        Err(e) => return Err(Box::new(e)),
    };

    // filter ให้เหลือแค่ timestamp volume close
    let filtered: Vec<(u64, u64, f64)> = quotes
        .into_iter()
        .map(|q: Quote| (q.timestamp as u64, q.volume as u64, q.close as f64))
        .collect();
    Ok(filtered)
}

pub async fn fetch_log_data(
    symbol: &str,
) -> Result<Vec<(u64, u64, f64)>, Box<dyn std::error::Error>> {
    let provider = yahoo::YahooConnector::new();

    // setup time
    let start_cron = match Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0) {
        chrono::LocalResult::Single(t) => t,
        _ => return Err("invalid start date".into()),
    };
    // Extend the range to cover the desired latest trading day (2025-10-05)
    let now_cron = match Utc.with_ymd_and_hms(2025, 10, 5, 0, 0, 0) {
        chrono::LocalResult::Single(t) => t,
        _ => return Err("invalid end date".into()),
    };
    // offset date time
    let start = match OffsetDateTime::from_unix_timestamp(start_cron.timestamp()) {
        Ok(v) => v,
        _ => return Err("invalid start time".into()),
    };
    let end = match OffsetDateTime::from_unix_timestamp(now_cron.timestamp()) {
        Ok(v) => v,
        _ => return Err("invalid end time".into()),
    };

    // time response
    let response = match provider?
        .get_quote_history_interval(symbol, start, end, "1d")
        .await
    {
        Ok(resp) => resp,
        Err(e) => return Err(Box::new(e)),
    };

    let quotes = match response.quotes() {
        Ok(quotes) => quotes,
        Err(e) => return Err(Box::new(e)),
    };

    // filter ให้เหลือแค่ timestamp volume close
    let filtered: Vec<(u64, u64, f64)> = quotes
        .into_iter()
        .map(|q: Quote| (q.timestamp as u64, q.volume as u64, (q.close).ln()))
        .collect();
    Ok(filtered)
}
