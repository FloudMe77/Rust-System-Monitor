use sysinfo::{Disks, System};

// struktura przechowująca zużycie wątku procesora
#[derive(Debug)]
pub struct CpuInfo {
    pub name: String,
    pub usage: Option<f32>,
}

// struktura reprezentująca inforamcje o danym dysku
#[derive(Debug)]
pub struct DiskInfo {
    pub name: Option<String>,
    pub mount_point: Option<String>,
    pub total_space: Option<u64>,
    pub available_space: Option<u64>,
}

// struktura reprezentująca globalne inforamcje systemowe
#[derive(Debug)]
pub struct GeneralInfo {
    pub ram_total_memory: Option<u64>,
    pub ram_available_memory: Option<u64>,
    pub ram_used_memor: Option<u64>,
    pub cpu_usage_tab: Vec<CpuInfo>,
    pub disk_tab: Vec<DiskInfo>,
}

impl GeneralInfo {
    // funkcja do pobierania danych o zasobach sprzętu
    pub fn get_general_data(sys: System) -> GeneralInfo {
        // CPU usage
        let mut cpu_usage_tab = Vec::new();
        for cpu in sys.cpus().iter() {
            cpu_usage_tab.push(CpuInfo {
                name: cpu.name().to_string(),
                usage: Some(cpu.cpu_usage()),
            });
        }

        // Dyski
        let disks = Disks::new_with_refreshed_list();
        let mut disk_tab = Vec::new();

        for disk in &disks {
            let name = disk.name().to_str().map(|s| s.to_string());
            let mount_point = disk.mount_point().to_str().map(|s| s.to_string());
            let disk_info = DiskInfo {
                name,
                mount_point,
                total_space: Some(disk.total_space()),
                available_space: Some(disk.available_space()),
            };
            disk_tab.push(disk_info);
        }
        // zwrócenie stuktury (siebie)
        Self {
            ram_total_memory: Some(sys.total_memory()),
            ram_available_memory: Some(sys.available_memory()),
            ram_used_memor: Some(sys.used_memory()),
            cpu_usage_tab,
            disk_tab,
        }
    }

    pub fn get_avg_cpu_usage(&self) -> f64 {
        // sumowanie zużycia każdego wątku i dzielenie go przez liczbę wszystkich wątków
        let (sum, count) = self.cpu_usage_tab.iter().filter_map(|cpu| cpu.usage).fold((0.0, 0), |(s, c), u| (s + u, c + 1));
        if count > 0 { (sum / count as f32) as f64 } else { 0.0 }
    }
} 