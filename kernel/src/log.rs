use crate::serial_println;
use log::{info, Level, Metadata, Record};

pub fn init() {
    ::log::set_logger(&SerialLogger).unwrap();
    ::log::set_max_level(::log::LevelFilter::Debug);

    info!("logging initialized");
}

pub struct SerialLogger;

impl SerialLogger {}

impl log::Log for SerialLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let color = match record.level() {
                Level::Error => "\x1b[1;31m",
                Level::Warn => "\x1b[1;33m",
                Level::Info => "\x1b[1;30m",
                Level::Debug => "\x1b[1;94m",
                Level::Trace => "\x1b[1;90m",
            };

            let target = record.target();
            let module_path = if let Some(last_colon) = target.rfind(':') {
                &target[last_colon + 1..]
            } else {
                target
            };

            serial_println!(
                "{}{:5}\x1b[0m [{}] {}",
                color,
                record.level(),
                module_path,
                record.args()
            );
        }
    }

    fn flush(&self) {
        // no-op
    }
}
