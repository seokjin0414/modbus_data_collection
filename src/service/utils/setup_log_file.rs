use chrono::{Duration, Local};
use std::fs::{self, OpenOptions};

pub fn setup_log_file() -> std::io::Result<std::fs::File> {
    // logs 디렉토리 없으면 생성
    fs::create_dir_all("logs")?;

    // 날짜 기반 로그 파일 이름
    let today = Local::now().format("%Y-%m-%d").to_string();
    let log_path = format!("logs/error-{}.log", today);

    // 오래된 로그 삭제 (30일 전)
    let retention_days = 30;
    let cutoff = Local::now() - Duration::days(retention_days);

    for entry in fs::read_dir("logs")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("error-") && filename.ends_with(".log") {
                    // 날짜 추출해서 파싱
                    if let Some(date_str) = filename
                        .strip_prefix("error-")
                        .and_then(|s| s.strip_suffix(".log"))
                    {
                        if let Ok(file_date) =
                            chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                        {
                            if file_date < cutoff.date_naive() {
                                let _ = fs::remove_file(path); // 실패 무시
                            }
                        }
                    }
                }
            }
        }
    }

    // 로그 파일 열기 (append 모드)
    OpenOptions::new().create(true).append(true).open(log_path)
}
