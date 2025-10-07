// enum ProcessName reprezentuje nazwy danych zbieranych o procesie
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessName {
    PID,
    NAME,
    CPU,
    MEM,
    READ,
    WRITE,
    TOTAL_READ,
    TOTAL_WRITTEN,
    USER,
}

impl ProcessName {
    // tablica wszystkich nazw
    pub const ALL: [ProcessName; 9] = [
        ProcessName::PID,
        ProcessName::NAME,
        ProcessName::CPU,
        ProcessName::MEM,
        ProcessName::READ,
        ProcessName::WRITE,
        ProcessName::TOTAL_READ,
        ProcessName::TOTAL_WRITTEN,
        ProcessName::USER,
    ];
    // poruszanie się do przodu po tablicy
    pub fn next(&self) -> ProcessName {
        let i = Self::ALL.iter().position(|x| x == self).unwrap();
        if i != (Self::ALL.len() - 2) {
            Self::ALL[i + 1]
        } else {
            Self::ALL[i]
        }
    }
    // poruszanie się do tyłu
    pub fn prev(&self) -> ProcessName {
        let i = Self::ALL.iter().position(|x| x == self).unwrap();
        if i != 0 {
            Self::ALL[i - 1]
        } else {
            Self::ALL[i]
        }
    }
    // podbieranie indeksu (przydatne przy określaniu, która to kolumna)
    pub fn get_index(&self) -> usize {
        Self::ALL.iter().position(|x| x == self).unwrap()
    }
    // pobieranie nazwy z indeksu (przydatne przy posiadaniu numeru kolumny)
    pub fn get_name(i: usize) -> ProcessName {
        Self::ALL[i]
    }
}

// przydatne stałe
pub const MAX_LEN: usize = 60;
pub const CHART_RANGE: [f64; 2] = [0.0, 60.0];
pub const INTERVAL: u64 = 500;
pub const ITEM_HEIGHT: u16 = 1;

// oznacznie kolumn
pub const COLUMN_LABEL: [&str; 8] = [
    "PID",
    "Name",
    "CPU %",
    "Mem",
    "R",
    "W",
    "T.Read",
    "T.Write",
];

// instrukcja obsługi
pub const INFO_TEXT: [&str; 2] = [
    "(Esc) quit | (↑) move up | (↓) move down | (←) move left | (→) move right | (Space) stop | (Tab) Cpu usage graph",
    "(Shift + →) left col sort | (Shift + ←) right col sort | (Shift + ↓) dec sort | (Shift + ↑) inc sort",
]; 