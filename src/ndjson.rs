//! Print logs as ndjson.

use log::{kv, LevelFilter, Log, Metadata, Record};
use serde_json::Value;
use std::collections::HashMap;
use std::time;

#[derive(Debug)]
pub struct Logger {}

#[derive(serde_derive::Serialize)]
struct Msg {
    level: u8,
    time: u128,
    msg: String,
    #[serde(flatten)]
    key_values: Option<HashMap<String, Value>>,
}

impl Logger {
    pub fn new() -> Self {
        Self {}
    }

    /// Start logging.
    pub fn start(self, filter: LevelFilter) -> Result<(), log::SetLoggerError> {
        let res = log::set_boxed_logger(Box::new(self));
        if res.is_ok() {
            log::set_max_level(filter);
        }
        res
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            print_ndjson(record)
        }
    }
    fn flush(&self) {}
}

// TODO: implement key_values mapping
fn print_ndjson(record: &Record<'_>) {
    let msg = Msg {
        level: get_level(record.level()),
        key_values: format_kv_pairs(&record),
        time: time::UNIX_EPOCH.elapsed().unwrap().as_millis(),
        msg: record.args().to_string(),
    };
    println!("{}", serde_json::to_string(&msg).unwrap())
}

fn get_level(level: log::Level) -> u8 {
    use log::Level::*;
    match level {
        Trace => 10,
        Debug => 20,
        Info => 30,
        Warn => 40,
        Error => 50,
    }
}

fn format_kv_pairs(record: &Record) -> Option<HashMap<String, Value>> {
    struct Visitor {
        key_values: Option<HashMap<String, Value>>,
    }

    impl<'kvs> kv::Visitor<'kvs> for Visitor {
        fn visit_pair(
            &mut self,
            key: kv::Key<'kvs>,
            val: kv::Value<'kvs>,
        ) -> Result<(), kv::Error> {
            if let None = self.key_values {
                self.key_values = Some(HashMap::new());
            }
            let kv = self.key_values.as_mut().unwrap();
            kv.insert(key.to_string(), val.to_string().into());
            Ok(())
        }
    }

    let mut visitor = Visitor { key_values: None };
    record.key_values().visit(&mut visitor).unwrap();
    visitor.key_values
}
