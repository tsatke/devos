use crate::mcore::context::ExecutionContext;
use crate::serial_println;
use log::{info, Level, Metadata, Record};

pub(crate) fn init() {
    log::set_logger(&SerialLogger).unwrap();
    log::set_max_level(::log::LevelFilter::Trace);

    info!("logging initialized");
}

pub struct SerialLogger;

impl SerialLogger {}

impl log::Log for SerialLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() < Level::Trace || metadata.target().starts_with("kernel")
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let color = match record.level() {
                Level::Error => "\x1b[1;31m",
                Level::Warn => "\x1b[1;33m",
                Level::Info => "\x1b[1;94m",
                Level::Debug => "\x1b[1;30m",
                Level::Trace => "\x1b[1;90m",
            };

            if let Some(cpu) = ExecutionContext::try_load().map(ExecutionContext::cpu_id) {
                serial_println!(
                    "{}{:5}\x1b[0m cpu{} [{}] {}",
                    color,
                    record.level(),
                    cpu,
                    record.target(),
                    record.args()
                );
            } else {
                serial_println!(
                    "{}{:5}\x1b[0m boot [{}] {}",
                    color,
                    record.level(),
                    record.target(),
                    record.args()
                );
            }
        }
    }

    fn flush(&self) {
        // no-op
    }
}
