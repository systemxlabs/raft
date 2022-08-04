use crate::proto;

#[derive(Debug)]
pub struct Log {
    entries: Vec<proto::LogEntry>,
    start_index: u64,
}

impl Log {

    pub fn new(start_index: u64) -> Self {
        Log {
            entries: Vec::new(),
            start_index,
        }
    }

    pub fn append_entries(&mut self, entries: Vec<proto::LogEntry>) {
        self.entries.extend(entries);
    }

    pub fn entries(&self) -> &Vec<proto::LogEntry> {
        &self.entries
    }

    pub fn start_index(&self) -> u64 {
        self.start_index
    }

    pub fn get_entry(&self, index: u64) -> Option<&proto::LogEntry> {
        self.entries.get(index as usize)
    }

    pub fn last_index(&self) -> u64 {
        self.entries.last().map(|entry| entry.index).unwrap_or(0)
    }

    pub fn last_term(&self) -> u64 {
        self.entries.last().map(|entry| entry.term).unwrap_or(0)
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn test_log() {
        let mut log = super::Log::new(1);
        let entry1 = super::proto::LogEntry {
            index: 1,
            term: 1,
            r#type: super::proto::EntryType::Data.into(),
            data: "test".as_bytes().to_vec(),
        };
        log.append_entries(vec![entry1]);

        let entry2 = super::proto::LogEntry {
            index: 2,
            term: 1,
            r#type: super::proto::EntryType::Data.into(),
            data: "test".as_bytes().to_vec(),
        };
        log.append_entries(vec![entry2]);

        println!("{:?}", log);
        assert_eq!(log.entries().len(), 2);
        assert_eq!(log.start_index(), 1);
        assert_eq!(log.get_entry(1).unwrap().index, 2);
    }
}
