use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Text,
    widgets::{
        Axis, Block, BorderType, Cell, Chart, Dataset, GraphType, HighlightSpacing, Paragraph, Row,
        Scrollbar, ScrollbarOrientation, Table,
    },
    Frame,
};
use ratatui::prelude::Span;

use crate::models::{CHART_RANGE, COLUMN_LABEL, INFO_TEXT, ITEM_HEIGHT, ProcessName};
use crate::utils::{change_units, format_option, format_option_units};
use super::state::App;

impl App {
    // funkcja renderująca główną tabele
    pub fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let mut header_names: Vec<String> = COLUMN_LABEL.iter().map(|s| s.to_string()).collect();
        // znak "kierunku" sortowania
        if self.reverse_sort {
            header_names[self.sort_tag.get_index()] += " (v) ";
        } else {
            header_names[self.sort_tag.get_index()] += " (^) ";
        }

        // mapowanie na celle i row
        let header_cells: Vec<Cell> = header_names.into_iter().map(Cell::from).collect();
        let header = Row::new(header_cells).style(Style::default()).height(ITEM_HEIGHT);

        // liczenie wierszy (za pomocą map)
        let rows = self.items.iter().enumerate().map(|(i, proc)| {
            let columns = [
                proc.pid.to_string(),
                proc.name.clone(),
                format_option(proc.cpu.map(|v| format!("{:.1}", v))),
                format_option_units(proc.mem_mb),
                format_option_units(proc.read_bytes.map(|v| v as f64)),
                format_option_units(proc.write_bytes.map(|v| v as f64)),
                format_option_units(proc.total_read.map(|v| v as f64)),
                format_option_units(proc.total_written.map(|v| v as f64)),
            ];

            let cells = columns
                .iter()
                .enumerate()
                .map(|(idx, content)| {
                    let mut cell = Cell::from(Text::from(format!("{content}")));
                    // jeżeli to jest ta zaznaczona komórka
                    if i == self.state.selected().unwrap_or(0) && idx == self.selected_column {
                        cell = cell.style(Style::default().add_modifier(Modifier::REVERSED));
                    }
                    cell
                })
                .collect::<Vec<_>>();

            Row::new(cells).height(ITEM_HEIGHT)
        }).collect::<Vec<_>>();

        // szerokości
        let widths = [
            Constraint::Length(self.longest_item_lens.0 + 1),
            Constraint::Length(self.longest_item_lens.1 + 1),
            Constraint::Length(self.longest_item_lens.2 + 1),
            Constraint::Length(self.longest_item_lens.3 + 1),
            Constraint::Length(self.longest_item_lens.4 + 1),
            Constraint::Length(self.longest_item_lens.5 + 1),
            Constraint::Length(self.longest_item_lens.6 + 1),
            Constraint::Length(self.longest_item_lens.7 + 1),
        ];
        // i tworzenie tabeli
        let table = Table::new(rows, &widths)
            .header(header)
            .block(Block::default().borders(ratatui::widgets::Borders::ALL).border_type(BorderType::Rounded))
            .widths(&widths)
            .highlight_symbol(">> ")
            .highlight_spacing(HighlightSpacing::Always);
        // zwracanie w frame
        frame.render_stateful_widget(table, area, &mut self.state);
    }
    // renderowanie scrollbar'a
    pub fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }
    // generowanie instrukcji
    pub fn render_footer(&self, frame: &mut Frame, area: Rect) {
        // wycentrowany paragraf
        let info_footer = Paragraph::new(Text::from_iter(INFO_TEXT))
            .style(Style::default())
            .centered()
            .block(Block::default().border_type(BorderType::Double).borders(ratatui::widgets::Borders::ALL));
        frame.render_widget(info_footer, area);
    }
    // generowanie wykresu
    pub fn render_animated_chart(&self, frame: &mut Frame, area: Rect) {
        // pobieranie odpowieniej tablicy do wykresu
        let values = if !self.plot_cpu {
            self.extract_history_data()
        } else {
            self.cpu_usage_history.iter().cloned().collect()
        };
        let len = values.len();
        // preparing danych (żeby wykres był dosunięty do prawej)
        let data: Vec<(f64, f64)> = (0..60)
            .map(|i| {
                let index = i as isize - (60 - len) as isize;
                let value = if index >= 0 { values[index as usize] } else { 0.0 };
                ((i + 1) as f64, value)
            })
            .collect();

        let x_labels = vec![
            Span::styled(format!("{}s", CHART_RANGE[1]), Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}s", (CHART_RANGE[0] + CHART_RANGE[1]) / 2.0)),
            Span::styled(format!("{}s", CHART_RANGE[0]), Style::default().add_modifier(Modifier::BOLD)),
        ];

        let describe_name = if self.plot_cpu {
            "Cpu avg usage".to_string()
        } else {
            format!("{} \n {:?}", self.chart_pid, ProcessName::get_name(self.chart_col))
        };

        let datasets = vec![Dataset::default()
            .name(describe_name)
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .graph_type(GraphType::Line)
            .data(&data)];

        let max_val = values.iter().copied().fold(f64::NEG_INFINITY, |a, b| a.max(b));

        let y_max_label = if self.chart_col < 3 || self.plot_cpu {
            format!("{}%", max_val.round() as u32)
        } else {
            change_units(max_val)
        };
        let y_mid_label = if self.chart_col < 3 || self.plot_cpu {
            format!("{}%", (max_val / 2.0).round() as u32)
        } else {
            change_units(max_val / 2.0)
        };

        let chart = Chart::new(datasets)
            .block(Block::bordered())
            .x_axis(
                Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    .labels(x_labels)
                    .bounds(CHART_RANGE),
            )
            .y_axis(
                Axis::default()
                    .title("Value")
                    .style(Style::default().fg(Color::Gray))
                    .labels(vec!["0".into(), y_mid_label.into(), y_max_label])
                    .bounds([0.0, max_val]),
            );

        frame.render_widget(chart, area);
    }
    // renderuje tablice zużycia wątków procesora
    // analogiczna co render_table
    pub fn render_cpu_usage(&mut self, frame: &mut Frame, area: Rect) {
        let header_names = vec!["Cpu name".to_string(), "Usage %".to_string()];
        let header_cells: Vec<Cell> = header_names.into_iter().map(Cell::from).collect();
        let header = Row::new(header_cells).style(Style::default()).height(ITEM_HEIGHT);

        let rows = self.general_info.cpu_usage_tab.iter().map(|cpu_info| {
            let columns = [
                cpu_info.name.to_string(),
                format_option(cpu_info.usage.map(|v| format!("{:.1}", v))),
            ];
            let cells = columns.iter().map(|content| Cell::from(Text::from(content.clone()))).collect::<Vec<_>>();
            Row::new(cells).height(ITEM_HEIGHT)
        }).collect::<Vec<_>>();

        let widths = [Constraint::Percentage(50), Constraint::Percentage(50)];

        let table = Table::new(rows, &widths)
            .header(header)
            .block(Block::default().borders(ratatui::widgets::Borders::ALL).border_type(BorderType::Rounded))
            .widths(&widths);

        frame.render_widget(table, area);
    }
    // renderuje tablice zużycia ramu
    // analogiczna co render_table
    pub fn render_ram_usage(&mut self, frame: &mut Frame, area: Rect) {
        let rows = vec![
            Row::new(vec!["Ram total memory".to_string(), format_option_units(self.general_info.ram_total_memory.map(|v| v as f64))]).height(ITEM_HEIGHT),
            Row::new(vec!["Ram available memory".to_string(), format_option_units(self.general_info.ram_available_memory.map(|v| v as f64))]).height(ITEM_HEIGHT),
            Row::new(vec!["Ram used memory".to_string(), format_option_units(self.general_info.ram_used_memor.map(|v| v as f64))]).height(ITEM_HEIGHT),
        ];

        let widths = [Constraint::Percentage(50), Constraint::Percentage(50)];

        let table = Table::new(rows, &widths)
            .block(Block::default().borders(ratatui::widgets::Borders::ALL).border_type(BorderType::Rounded))
            .widths(&widths);

        frame.render_widget(table, area);
    }
    // renderuje tablice zużycia dysków
    // analogiczna co render_table
    pub fn render_disk_usage(&mut self, frame: &mut Frame, area: Rect) {
        let header_names = vec![
            "Disk name".to_string(),
            "Mount point".to_string(),
            "Total space".to_string(),
            "Available space".to_string(),
        ];

        let header_cells: Vec<Cell> = header_names.into_iter().map(Cell::from).collect();
        let header = Row::new(header_cells).style(Style::default()).height(ITEM_HEIGHT);

        let rows = self.general_info.disk_tab.iter().map(|disk_info| {
            let columns = [
                format_option(disk_info.name.clone()),
                format_option(disk_info.mount_point.clone()),
                format_option_units(disk_info.total_space.map(|v| v as f64)),
                format_option_units(disk_info.available_space.map(|v| v as f64)),
            ];
            let cells = columns.iter().map(|content| Cell::from(Text::from(content.clone()))).collect::<Vec<_>>();
            Row::new(cells).height(ITEM_HEIGHT)
        }).collect::<Vec<_>>();

        let widths = [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ];

        let table = Table::new(rows, &widths)
            .header(header)
            .block(Block::default().borders(ratatui::widgets::Borders::ALL).border_type(BorderType::Rounded))
            .widths(&widths);

        frame.render_widget(table, area);
    }

    // centralna funkcja do rysowania
    pub fn draw(&mut self, frame: &mut Frame) {
        // dzielenie przestrzeni na główną i opis na dole
        let vertical: [Rect; 2] = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(4)])
            .areas(frame.area());
        // dzielenie na główną tabele po lewej i reszte po prawej
        let [left, right]: [Rect; 2] = Layout::horizontal([Constraint::Fill(1); 2]).areas(vertical[0]);
        // dzielenie na wykres i reszte danych
        let [line_chart, performance]: [Rect; 2] = Layout::vertical([Constraint::Fill(1); 2]).areas(right);
        // dzielenie na tabele cpu i reszte
        let [cpu_rect, mem_rect]: [Rect; 2] = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(3)]).areas(performance);
        // ostatnie dzielenie na tabele dal ramu i tabele dla dysku
        let [ram_rect, disk_rect]: [Rect; 2] = Layout::vertical([Constraint::Fill(1); 2]).areas(mem_rect);

        // renderowanie widgetow
        self.render_table(frame, left);
        self.render_scrollbar(frame, left);
        self.render_footer(frame, vertical[1]);
        self.render_animated_chart(frame, line_chart);
        self.render_cpu_usage(frame, cpu_rect);
        self.render_ram_usage(frame, ram_rect);
        self.render_disk_usage(frame, disk_rect);
    }
} 