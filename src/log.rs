use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use std::io::{Write, Read};
use crate::{peer, proto, config};
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


#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    entries: Vec<proto::LogEntry>,
    start_index: u64,
    metadata_dir: String,
    #[serde(skip)]
    append_mutex: Mutex<String>,
}

impl Log {

    pub fn new(start_index: u64, metadata_dir: String) -> Self {
        Log {
            entries: Vec::new(),
            start_index,
            metadata_dir,
            append_mutex: Mutex::new("".to_string()),
        }
    }

    pub fn append_data(&mut self, term: u64, entry_data: Vec<LogEntryData>) {
        // 防止并发插入相同index的log entry
        if let Ok(_) = self.append_mutex.lock() {
            for entry in entry_data {
                let log_entry = proto::LogEntry {
                    index: self.last_index(0) + 1,
                    term,
                    r#type: entry.0.into(),
                    data: entry.1,
                };
                self.entries.push(log_entry);
            }
            self.dump();
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
            self.dump();
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

    pub fn last_index(&self, last_included_index: u64) -> u64 {
        // 日志因snapshot被清理
        if self.entries.is_empty() && last_included_index > 0 {
            return last_included_index;
        }
        self.entries.last().map(|entry| entry.index).unwrap_or(self.start_index - 1)
    }

    pub fn last_term(&self, last_included_term: u64) -> u64 {
        // 日志因snapshot被清理
        if self.entries.is_empty() && last_included_term > 0 {
            return last_included_term;
        }
        self.entries.last().map(|entry| entry.term).unwrap_or(0)
    }

    pub fn prev_log_term(&self, prev_log_index: u64, last_included_index: u64, last_included_term: u64) -> u64 {
        // prev_log 因snapshot被清理
        if prev_log_index == last_included_index {
            return last_included_term;
        }
        return self.entry(prev_log_index).unwrap().term;
    }

    // 截掉后续不匹配日志
    pub fn truncate_suffix(&mut self, last_index_kept: u64) {
        if last_index_kept < self.start_index {
            return;
        }
        self.entries.truncate((last_index_kept - self.start_index + 1) as usize);
        self.dump();
    }

    // 截掉前面已快照日志
    pub fn truncate_prefix(&mut self, last_included_index: u64) {
        if last_included_index < self.start_index {
            return;
        }
        self.entries.drain(0..(last_included_index - self.start_index + 1) as usize);
        self.start_index = last_included_index + 1;
        self.dump();
    }

    pub fn committed_entries_len(&self, commit_index: u64) -> usize {
        if commit_index < self.start_index {
            return 0;
        }
        return (commit_index - self.start_index + 1) as usize;
    }

    pub fn last_configuration(&self) -> Option<config::Configuration> {
        for entry in self.entries().iter().rev() {
            if entry.r#type() == proto::EntryType::Configuration {
                return Some(config::Configuration::from_data(&entry.data.as_ref()));
            }
        }
        return None;
    }

    pub fn gen_log_filepath(metadata_dir: &String) -> String {
        format!("{}/raft.log", metadata_dir)
    }

    pub fn reload(&mut self) {
        let filepath = Log::gen_log_filepath(&self.metadata_dir);
        if std::path::Path::new(&filepath).exists() {
            let mut log_file = std::fs::File::open(filepath).unwrap();
            let mut log_json = String::new();
            log_file.read_to_string(&mut log_json).expect("failed to read raft log");
            let log: Log = serde_json::from_str(log_json.as_str()).unwrap();
            self.entries = log.entries;
            self.start_index = log.start_index;
        }
    }

    // 保存日志到硬盘
    pub fn dump(&self) {
        let log_filepath = Log::gen_log_filepath(&self.metadata_dir);
        let mut log_file = std::fs::File::create(log_filepath).unwrap();
        let log_json = serde_json::to_string(self).unwrap();
        if let Err(e) = log_file.write(log_json.as_bytes()) {
            panic!("failed to write raft log file, error: {}", e)
        }
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn test_log() {
        let mut log = super::Log::new(1, "./test".to_string());

        log.append_data(1, vec![(super::proto::EntryType::Data, "test1".as_bytes().to_vec())]);

        log.append_data(1, vec![(super::proto::EntryType::Data, "test2".as_bytes().to_vec())]);

        println!("{:?}", log);
        assert_eq!(log.entries().len(), 2);
        assert_eq!(log.start_index(), 1);
        assert_eq!(log.entry(1).unwrap().data, "test1".as_bytes());
        assert_eq!(log.entry(2).unwrap().data, "test2".as_bytes());
        assert_eq!(log.last_index(0), 2);
        assert_eq!(log.pack_entries(1).len(), 2);
        assert_eq!(log.pack_entries(2).len(), 1);
        assert_eq!(log.pack_entries(3).len(), 0);
    }

    #[test]
    fn test_truncate_suffix() {
        let mut log = super::Log::new(2, "./test".to_string());
        log.append_data(1, vec![(super::proto::EntryType::Data, "test1".as_bytes().to_vec())]);
        log.append_data(1, vec![(super::proto::EntryType::Data, "test2".as_bytes().to_vec())]);
        log.append_data(1, vec![(super::proto::EntryType::Data, "test3".as_bytes().to_vec())]);

        log.truncate_suffix(3);
        assert_eq!(log.entries().len(), 2);
    }

    #[test]
    fn test_truncate_prefix() {
        let mut log = super::Log::new(1, "./test".to_string());
        log.append_data(1, vec![(super::proto::EntryType::Data, "test1".as_bytes().to_vec())]);
        log.append_data(1, vec![(super::proto::EntryType::Data, "test2".as_bytes().to_vec())]);
        log.append_data(1, vec![(super::proto::EntryType::Data, "test3".as_bytes().to_vec())]);
        log.append_data(1, vec![(super::proto::EntryType::Data, "test4".as_bytes().to_vec())]);
        log.append_data(1, vec![(super::proto::EntryType::Data, "test5".as_bytes().to_vec())]);

        log.truncate_prefix(3);
        assert_eq!(log.entries().len(), 2);
        assert_eq!(log.start_index(), 4);

        log.append_data(1, vec![(super::proto::EntryType::Data, "test6".as_bytes().to_vec())]);
        log.append_data(1, vec![(super::proto::EntryType::Data, "test7".as_bytes().to_vec())]);
        log.append_data(1, vec![(super::proto::EntryType::Data, "test8".as_bytes().to_vec())]);

        log.truncate_prefix(5);
        assert_eq!(log.entries().len(), 3);
        assert_eq!(log.start_index(), 6);
    }
}
