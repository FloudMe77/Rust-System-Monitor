use std::collections::VecDeque;
use sysinfo::{Pid, Process, System};

// struktura przechowująca informacje o danym procesie
#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub cpu: Option<f32>,
    pub mem_mb: Option<f64>,
    pub read_bytes: Option<u64>,
    pub write_bytes: Option<u64>,
    pub total_read: Option<u64>,
    pub total_written: Option<u64>,
    pub user: Option<String>,
}

impl ProcessInfo {
    // pobieranie danych o DANYM procesie
    pub fn get_data_from_process(process: &Process, sys: &System) -> Self {
        let disk = process.disk_usage();
        let mem_mb = Some(process.memory() as f64);
        let usage = Some(process.cpu_usage());
        let core_count = sys.physical_core_count().unwrap_or(1) as f32;
        let percent_of_total = usage.map(|u| (u / (core_count)) );

        Self {
            pid: process.pid(),
            name: process.name().to_string(),
            cpu: percent_of_total,
            mem_mb,
            read_bytes: Some(disk.read_bytes),
            write_bytes: Some(disk.written_bytes),
            total_read: Some(disk.total_read_bytes),
            total_written: Some(disk.total_written_bytes),
            user: None,
        }
    }
}

// struktura przechowująca listy do generowania wykresów
// wykorzystywane VecDeque, żeby trzymać tylko 60 ostatnich danych
#[derive(Default)]
pub struct ProcessInfoHistory {
    pub cpu: VecDeque<f32>,
    pub mem_mb: VecDeque<f64>,
    pub read_bytes: VecDeque<u64>,
    pub write_bytes: VecDeque<u64>,
    pub total_read: VecDeque<u64>,
    pub total_written: VecDeque<u64>,
} 