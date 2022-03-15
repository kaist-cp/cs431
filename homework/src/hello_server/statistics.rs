//! Server statisics

use std::collections::HashMap;

/// Report for each operation
#[derive(Debug)]
pub struct Report {
    _id: usize,
    key: Option<String>, // None represents invalid request
}

impl Report {
    /// Creates a new report with the given id and key.
    pub fn new(id: usize, key: Option<String>) -> Self {
        Report { _id: id, key }
    }
}

/// Operation statisics
#[derive(Debug, Default)]
pub struct Statistics {
    hits: HashMap<Option<String>, usize>,
}

impl Statistics {
    /// Add a report to the statisics.
    pub fn add_report(&mut self, report: Report) {
        let hits = self.hits.entry(report.key).or_default();
        *hits += 1;
    }
}
