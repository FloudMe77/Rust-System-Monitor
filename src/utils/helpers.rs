use std::collections::VecDeque;
use crate::models::MAX_LEN;
use std::fmt::Display;

pub fn push_bounded<T>(queue: &mut VecDeque<T>, item: T) {
    // implementacja kolejki pomiarów do wykresów
    if queue.len() == MAX_LEN {
        queue.pop_front();
    }
    queue.push_back(item);
}

pub fn change_units(n: f64) -> String {
    // zamienia jednostki z bajtów na kilobajty itd.
    let mut num = n;
    let mut i: i16 = 0;
    while num > 1024.0 {
        num /= 1024.0;
        i += 1;
    }
    match i {
        0 => format!("{:.1}B", num),
        1 => format!("{:.1}KB", num),
        2 => format!("{:.1}MB", num),
        3 => format!("{:.1}GB", num),
        _ => format!("{:.1}", num),
    }
}

// funkcja do estetycznego pokazywanie danych
pub fn format_option<T: Display>(val: Option<T>) -> String {
    match val {
        Some(v) => v.to_string(),
        None => "--".to_string(),
    }
}

// funkcja do estetycznego pokazywanie danych
pub fn format_option_units(val: Option<f64>) -> String {
    match val {
        Some(v) => change_units(v),
        None => "--".to_string(),
    }
} 