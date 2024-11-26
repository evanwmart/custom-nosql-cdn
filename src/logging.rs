use std::sync::{Arc, Mutex};
use log::{Record, Level, Metadata};
use serde::Serialize;
use chrono::Datelike;
use std::io::Write;


#[derive(Serialize, Clone)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}

pub struct SharedLogger {
    buffer: Arc<Mutex<Vec<LogEntry>>>,
    capacity: usize,
}

impl SharedLogger {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            capacity,
        }
    }

    pub fn get_logs(&self, _as_json: bool) -> Vec<LogEntry> {
        self.buffer.lock().unwrap().clone()
    }

    fn add_to_buffer(&self, log_entry: LogEntry) {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.len() >= self.capacity {
            buffer.remove(0);
        }
        buffer.push(log_entry);
    }
}

impl log::Log for SharedLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_entry = LogEntry {
                level: format!("{}", record.level()),
                message: format!("{}", record.args()),
                timestamp: chrono::Local::now().to_rfc3339(),
            };

            // Write to in-memory buffer
            self.add_to_buffer(log_entry.clone());

            // Define the log file path based on the current date
            let date = chrono::Local::now();
            let dir_path = format!("logs/{}/{}/{}", date.year(), date.format("%m"), date.format("%d"));
            let log_file_path = format!("{}.log", dir_path);

            // Ensure the directory exists
            std::fs::create_dir_all(&dir_path).unwrap();

            // Append to the log file
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file_path)
            {
                let log_line = format!(
                    "{} [{}] {}\n",
                    log_entry.timestamp, log_entry.level, log_entry.message
                );
                let _ = file.write_all(log_line.as_bytes());
            }
        }
    }

    fn flush(&self) {}
}

pub fn init_logger() -> Arc<SharedLogger> {
    let logger = Arc::new(SharedLogger::new(100));
    log::set_max_level(log::LevelFilter::Info);
    log::set_boxed_logger(Box::new(logger.clone())).unwrap();
    logger
}
