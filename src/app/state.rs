use std::collections::{HashMap, VecDeque};
use sysinfo::{Pid, System};
use unicode_width::UnicodeWidthStr;
use ratatui::widgets::TableState;
use std::cmp::max;
use ratatui::widgets::ScrollbarState;

use crate::models::{ProcessInfo, ProcessInfoHistory, GeneralInfo, ProcessName, COLUMN_LABEL};
use crate::utils::push_bounded;

// struktura stanu aplikacji
pub struct App {
    pub state: TableState, // stan tabeli
    pub items: Vec<ProcessInfo>, // tablica aktualnie zczytanych danych o procesach
    pub longest_item_lens: (u16, u16, u16, u16, u16, u16, u16, u16), // szerokość kolumn
    pub scroll_state: ScrollbarState, // stan scrollbar'a
    pub selected_column: usize, // zaznaczona kolumna przez kursor
    pub sort_tag: ProcessName, // nazwa względem której sortujemy
    pub reverse_sort: bool, // czy sortujemy rosnąco, czy malejąco
    pub process_stats_history: HashMap<Pid, ProcessInfoHistory>, // mapa Pid -> ProcessInfoHistory (do rysowania wykresów)
    pub chart_pid: Pid, // Pid procesu, którego rysujemy wykres
    pub chart_col: usize, // columna, której rysujemy wykres
    pub stop: bool, // zatrzymanie
    pub cpu_usage_history: VecDeque<f64>, // tablica "najświerzszych" danych o zużyciu procesora
    pub general_info: GeneralInfo, // aktualne dane o całym systemie
    pub plot_cpu: bool, // czy rysować dane zaznaczonej komórki tabeli, czy zużycie procesora
}

impl App {
    pub fn new() -> Self {
        // na starcie
        // pobieram dane
        let (items, general_info) = get_data();
        // wyliczam szerokość kolumn
        let longest_item_lens = Self::constraint_len_calculator(&items);
        // inicjalizuje struktury
        let mut cpu_usage_history = VecDeque::new();
        push_bounded(&mut cpu_usage_history, general_info.get_avg_cpu_usage());

        let items_len = items.len();

        // inicjalizacja stanu
        Self {
            state: TableState::default().with_selected(Some(0)),
            longest_item_lens,
            scroll_state: ScrollbarState::new(items_len.saturating_sub(1)),
            items,
            selected_column: 0,
            sort_tag: ProcessName::CPU,
            reverse_sort: true,
            process_stats_history: HashMap::new(),
            chart_col: 3,
            chart_pid: Pid::from_u32(std::process::id()),
            stop: false,
            cpu_usage_history,
            general_info,
            plot_cpu: true,
        }
    }
    // nastepny rząd dla zaznaczonej komórki
    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) if i >= self.items.len() - 1 => 0,
            Some(i) => i + 1,
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i);
    }
    // poprzedni rząd dla zaznaczonej komórki
    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) if i == 0 => self.items.len() - 1,
            Some(i) => i - 1,
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i);
    }
    // następna kolumna dla zaznaczonej komórki
    pub fn next_column(&mut self) {
        self.selected_column = (self.selected_column + 1) % 8;
    }
    // poprzednia kolumna dla zaznaczonej komórki
    pub fn previous_column(&mut self) {
        if self.selected_column == 0 {
            self.selected_column = 7;
        } else {
            self.selected_column -= 1;
        }
    }
    // sortowanie danych według aktualnego stanu
    pub fn sort_data(&mut self) {
        self.items.sort_by(|a, b| {
            let ordering = match self.sort_tag {
                ProcessName::PID => a.pid.cmp(&b.pid),
                ProcessName::NAME => a.name.cmp(&b.name),
                ProcessName::CPU => a.cpu.partial_cmp(&b.cpu).unwrap_or(std::cmp::Ordering::Equal),
                ProcessName::MEM => a.mem_mb.partial_cmp(&b.mem_mb).unwrap_or(std::cmp::Ordering::Equal),
                ProcessName::READ => a.read_bytes.cmp(&b.read_bytes),
                ProcessName::WRITE => a.write_bytes.cmp(&b.write_bytes),
                ProcessName::TOTAL_READ => a.total_read.cmp(&b.total_read),
                ProcessName::TOTAL_WRITTEN => a.total_written.cmp(&b.total_written),
                ProcessName::USER => a.user.cmp(&b.user),
            };
            if self.reverse_sort {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    // zapisywanie aktualnych danych do mapy
    pub fn save_history_data(&mut self) {
        for proc in self.items.iter() {
            let pid = proc.pid;

            if !self.process_stats_history.contains_key(&pid) {
                let tmp = ProcessInfoHistory::default();
                self.process_stats_history.insert(pid, tmp);
            }

            if let Some(proc_his) = self.process_stats_history.get_mut(&pid) {
                push_bounded(&mut proc_his.cpu, proc.cpu.unwrap_or(0.0));
                push_bounded(&mut proc_his.mem_mb, proc.mem_mb.unwrap_or(0.0));
                push_bounded(&mut proc_his.read_bytes, proc.read_bytes.unwrap_or(0));
                push_bounded(&mut proc_his.write_bytes, proc.write_bytes.unwrap_or(0));
                push_bounded(&mut proc_his.total_read, proc.total_read.unwrap_or(0));
                push_bounded(&mut proc_his.total_written, proc.total_written.unwrap_or(0));
            }
        }
    }
    // funkcja do generowania wykresu
    // przy aktualnym stanie, zwraca odpowiednią tablice do wyrysowania wykresu
    pub fn extract_history_data(&self) -> Vec<f64> {
        let pid = self.chart_pid;
        let proces_history = self.process_stats_history.get(&pid).unwrap();

        match ProcessName::get_name(self.chart_col) {
            ProcessName::CPU => proces_history.cpu.iter().map(|&v| v as f64).collect(),
            ProcessName::MEM => proces_history.mem_mb.iter().copied().collect(),
            ProcessName::READ => proces_history.read_bytes.iter().map(|&v| v as f64).collect(),
            ProcessName::WRITE => proces_history.write_bytes.iter().map(|&v| v as f64).collect(),
            ProcessName::TOTAL_READ => proces_history.total_read.iter().map(|&v| v as f64).collect(),
            ProcessName::TOTAL_WRITTEN => proces_history.total_written.iter().map(|&v| v as f64).collect(),
            _ => proces_history.cpu.iter().map(|&v| v as f64).collect(),
        }
    }

    // zapisywanie pobranych danych o procesach i systemie
    pub fn update_data(&mut self) {
        let (items, general_info) = get_data();
        push_bounded(&mut self.cpu_usage_history, general_info.get_avg_cpu_usage());
        self.items = items;
        self.general_info = general_info;
        self.sort_data();
        self.save_history_data();
    }

    // funkcja licząca szerokośc kolumn
    fn constraint_len_calculator(items: &[ProcessInfo]) -> (u16, u16, u16, u16, u16, u16, u16, u16) {

        // funkcja lokalna do wyznaczania naszerszego elementu w kolumnie
        fn max_width_str<I>(iter: I) -> usize
        where
            I: Iterator<Item = String>,
        {
            iter.map(|s| UnicodeWidthStr::width(s.as_str()))
                .max()
                .unwrap_or(0)
        }
        // dla każdej kolumny wyliczam
        let pid_len = max(COLUMN_LABEL[0].len() + 5, max_width_str(items.iter().map(|p| p.pid.to_string())));
        let name_len = max(COLUMN_LABEL[1].len() + 5, max_width_str(items.iter().map(|p| p.name.clone())));
        let cpu_len = max(COLUMN_LABEL[2].len() + 5, max_width_str(items.iter().map(|p| format!("{:.1}", p.cpu.unwrap_or(0.0)))));
        let mem_len = max(COLUMN_LABEL[3].len() + 5, max_width_str(items.iter().map(|p| format!("{:.1}", p.mem_mb.unwrap_or(0.0)))));
        let read_len = max(COLUMN_LABEL[4].len() + 5, max_width_str(items.iter().map(|p| p.read_bytes.unwrap_or(0).to_string())));
        let write_len = max(COLUMN_LABEL[5].len() + 5, max_width_str(items.iter().map(|p| p.write_bytes.unwrap_or(0).to_string())));
        let total_read_len = max(COLUMN_LABEL[6].len() + 5, max_width_str(items.iter().map(|p| p.total_read.unwrap_or(0).to_string())));
        let total_written_len = max(COLUMN_LABEL[7].len() + 5, max_width_str(items.iter().map(|p| p.total_written.unwrap_or(0).to_string())));
        // zwracanie krotki
        (
            pid_len as u16,
            name_len as u16,
            cpu_len as u16,
            mem_len as u16,
            read_len as u16,
            write_len as u16,
            total_read_len as u16,
            total_written_len as u16,
        )
    }
}
// pobieranie świerzych danych o procesach i systemie
pub fn get_data() -> (Vec<ProcessInfo>, GeneralInfo) {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut process_info_all = Vec::new();
    for (_pid, process) in sys.processes() {
        process_info_all.push(ProcessInfo::get_data_from_process(process, &sys));
    }
    (process_info_all, GeneralInfo::get_general_data(sys))
} 