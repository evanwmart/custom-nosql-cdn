// src/logging.rs

use crossbeam::channel::{unbounded, Sender, Receiver};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use log::{Record, Level, Metadata};
use serde::Serialize;
use chrono::{Local, Datelike}; 
use std::fs::File;
use std::io::Write;
use std::time::Duration; 

#[derive(Serialize, Clone)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}

pub struct CircularBuffer<T> {
    buffer: Mutex<VecDeque<T>>,
    capacity: usize,
}

impl<T> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    pub fn push(&self, item: T) {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.len() == self.capacity {
            buffer.pop_front(); // Remove the oldest item
        }
        buffer.push_back(item);
    }

    pub fn get_logs(&self) -> Vec<T>
    where
        T: Clone,
    {
        let buffer = self.buffer.lock().unwrap();
        buffer.iter().cloned().collect()
    }
}

pub struct SharedLogger {
    buffer: Arc<CircularBuffer<LogEntry>>,
    sender: Sender<LogEntry>,
}

impl SharedLogger {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = unbounded();
        let logger = Self {
            buffer: Arc::new(CircularBuffer::new(capacity)),
            sender,
        };

        // Start the background logging thread
        std::thread::spawn(move || {
            logging_thread(receiver);
        });

        logger
    }

    pub fn get_logs(&self) -> Vec<LogEntry> {
        self.buffer.get_logs()
    }
}

impl log::Log for SharedLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_entry = LogEntry {
                level: record.level().to_string(),
                message: record.args().to_string(),
                timestamp: Local::now().to_rfc3339(),
            };

            // Add to in-memory circular buffer
            self.buffer.push(log_entry.clone());

            // Send to the background logging thread
            let _ = self.sender.send(log_entry);
        }
    }

    fn flush(&self) {}
}

fn logging_thread(receiver: Receiver<LogEntry>) {
    let mut buffer = Vec::new();
    let mut current_date = Local::now().format("%Y-%m-%d").to_string();
    let mut file = open_log_file();

    loop {
        match receiver.recv_timeout(Duration::from_millis(500)) {
            Ok(log_entry) => {
                buffer.push(log_entry);
                // Flush if buffer reaches a certain size
                if buffer.len() >= 500 {
                    flush_buffer(&mut file, &mut buffer);
                }
            }
            Err(crossbeam::channel::RecvTimeoutError::Timeout) => {
                // Flush any remaining logs
                if !buffer.is_empty() {
                    flush_buffer(&mut file, &mut buffer);
                }
            }
            Err(crossbeam::channel::RecvTimeoutError::Disconnected) => {
                // Flush and exit the thread
                if !buffer.is_empty() {
                    flush_buffer(&mut file, &mut buffer);
                }
                break;
            }
        }

        // Check if date has changed
        let today = Local::now().format("%Y-%m-%d").to_string();
        if current_date != today {
            file = open_log_file();
            current_date = today;
        }
    }
}

fn open_log_file() -> File {
    let date = Local::now();
    let dir_path = format!("logs/{}/{}", date.year(), date.format("%m"));
    let log_file_path = format!("{}/{}.log", dir_path, date.format("%d"));

    std::fs::create_dir_all(&dir_path).unwrap();

    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .unwrap()
}

fn flush_buffer(file: &mut File, buffer: &mut Vec<LogEntry>) {
    for log_entry in buffer.drain(..) {
        let log_line = format!(
            "{} [{}] {}\n",
            log_entry.timestamp, log_entry.level, log_entry.message
        );
        if let Err(e) = file.write_all(log_line.as_bytes()) {
            eprintln!("Failed to write log entry: {}", e);
        }
    }
    if let Err(e) = file.flush() {
        eprintln!("Failed to flush log file: {}", e);
    }
}

pub fn init_logger() -> Arc<SharedLogger> {
    let logger = Arc::new(SharedLogger::new(100));
    log::set_max_level(log::LevelFilter::Info);
    log::set_boxed_logger(Box::new(logger.clone())).unwrap();
    logger
}
