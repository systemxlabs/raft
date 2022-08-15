use std::sync::Mutex;

use crate::proto;
use crate::logging::*;

lazy_static::lazy_static! {
    static ref VIRTUAL_LOG_ENTRY: proto::LogEntry = proto::LogEntry {
        index: 0,
        term: 0,
        r#type: proto::EntryType::Noop.into(),
        data: "".as_bytes().to_vec(),
    };
}

pub type LogEntryData = (proto::EntryType, Vec<u8>);

#[derive(Debug)]
pub struct Log {
    entries: Vec<proto::LogEntry>,
    start_index: u64,
    append_mutex: Mutex<String>,
}

impl Log {

    pub fn new(start_index: u64) -> Self {
        Log {
            entries: Vec::new(),
            start_index,
            append_mutex: Mutex::new("".to_string()),
        }
    }

    pub fn append_data(&mut self, term: u64, entry_data: Vec<LogEntryData>) {
        // 防止并发插入相同index的log entry
        if let Ok(_) = self.append_mutex.lock() {
            for entry in entry_data {
                let log_entry = proto::LogEntry {
                    index: self.last_index() + 1,
                    term,
                    r#type: entry.0.into(),
                    data: entry.1,
                };
                self.entries.push(log_entry);
            }
        } else {
            error!("append log entry failed due to lock failure");
            return;
        }
    }

    pub fn append_entries(&mut self, entries: Vec<proto::LogEntry>) {
        // 防止并发插入相同index的log entry
        if let Ok(_) = self.append_mutex.lock() {
            for entry in entries {
                self.entries.push(entry);
            }
        } else {
            error!("append log entry failed due to lock failure");
            return;
        }
    }

    pub fn entries(&self) -> &Vec<proto::LogEntry> {
        &self.entries
    }

    pub fn start_index(&self) -> u64 {
        self.start_index
    }

    pub fn entry(&self, index: u64) -> Option<&proto::LogEntry> {
        if index < self.start_index {
            return Some(&VIRTUAL_LOG_ENTRY);
        }
        self.entries.get((index - self.start_index) as usize)
    }

    pub fn pack_entries(&self, next_index: u64) -> Vec<proto::LogEntry> {
        let mut res: Vec<proto::LogEntry> = Vec::new();
        if next_index < self.start_index {
            return res;
        }
        for entry in self.entries.iter().skip((next_index - self.start_index) as usize) {
            res.push(entry.clone());
        }
        res
    }

    pub fn last_index(&self) -> u64 {
        self.entries.last().map(|entry| entry.index).unwrap_or(self.start_index - 1)
    }

    pub fn last_term(&self) -> u64 {
        self.entries.last().map(|entry| entry.term).unwrap_or(0)
    }

    pub fn truncate_suffix(&mut self, last_index_kept: u64) {
        if last_index_kept < self.start_index {
            return;
        }
        self.entries.truncate((last_index_kept - self.start_index + 1) as usize);
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn test_log() {
        let mut log = super::Log::new(1);

        log.append(1, vec![(super::proto::EntryType::Data, "test1".as_bytes().to_vec())]);

        log.append(1, vec![(super::proto::EntryType::Data, "test2".as_bytes().to_vec())]);

        println!("{:?}", log);
        assert_eq!(log.entries().len(), 2);
        assert_eq!(log.start_index(), 1);
        assert_eq!(log.entry(1).unwrap().data, "test1".as_bytes());
        assert_eq!(log.entry(2).unwrap().data, "test2".as_bytes());
        assert_eq!(log.last_index(), 2);
        assert_eq!(log.pack_entries(1).len(), 2);
        assert_eq!(log.pack_entries(2).len(), 1);
        assert_eq!(log.pack_entries(3).len(), 0);
    }

    #[test]
    fn test_truncate_suffix() {
        let mut log = super::Log::new(2);
        log.append_data(1, vec![(super::proto::EntryType::Data, "test1".as_bytes().to_vec())]);
        log.append_data(1, vec![(super::proto::EntryType::Data, "test2".as_bytes().to_vec())]);
        log.append_data(1, vec![(super::proto::EntryType::Data, "test3".as_bytes().to_vec())]);

        log.truncate_suffix(3);
        assert_eq!(log.entries().len(), 2);

    }
}
