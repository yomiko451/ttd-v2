use std::io::Write;

use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Serialize, Deserialize};


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Todo {
    pub text: String,
    pub created_at: String,
    pub kind: TodoKind,
    pub state: TodoState,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum TodoState {
    #[default]
    Indefinite,
    OnGoing,
    UpComing,
    Expired,
}

impl TodoState {
    pub fn print_info(&self) -> String {
        match self {
            TodoState::Indefinite => "Indefinite".to_string(),
            TodoState::OnGoing => "OnGoing".to_string(),
            TodoState::UpComing => "UpComing".to_string(),
            TodoState::Expired => "Expired".to_string(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum TodoKind {
    #[default]
    General,
    Progress(String),//TODO 怎么解析？用@符号？
    Week(Weekday),
    Month(u32),
    Once(NaiveDate),
}

impl TodoKind {
    pub fn print_info(&self) -> String {
        match self {
            TodoKind::General => "General".to_string(),
            TodoKind::Progress(s) => format!("Progress: {}", s),
            TodoKind::Week(w) => format!("Week: {}", w),
            TodoKind::Month(m) => format!("Month: {}", m),
            TodoKind::Once(d) => format!("Once: {}", d),
        }
    }
}

impl Todo {
    pub fn new(input: &str) -> Self {
        let (todo_text, todo_kind) = Self::input_parse(&input);
        let mut todo = Todo {
            text: todo_text.to_string(),
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            kind: todo_kind,
            state: TodoState::default(),//TODO 和当前比较一下
        };
        todo.state_check();
        todo
    }

    fn input_parse(input: &str) -> (&str, TodoKind) {
        match input.split_once('-') {
            Some((text, suffix)) => {
                if !text.is_empty() && !suffix.is_empty() {
                    let todo_text = text.trim();
                    let suffix = suffix.trim();
                    let mut todo_kind = TodoKind::default();
                    if let Ok(weekday) = suffix.parse::<chrono::Weekday>() {
                        todo_kind = TodoKind::Week(weekday);
                        return (todo_text, todo_kind);
                    }
                    if let Ok(day) = suffix.parse::<u32>() {
                        if day > 0 && day < 32 {
                            todo_kind = TodoKind::Month(day);
                            return (todo_text, todo_kind);
                        }
                    }
                    if let Ok(date) = suffix.parse::<NaiveDate>() {
                        todo_kind = TodoKind::Once(date);
                        return (todo_text, todo_kind);
                    }
                    (input.trim(), todo_kind)
    
                } else {
                    (input.trim(), TodoKind::default())
                }
            }
            None => {
                match input.split_once('@') {
                    Some((text, suffix)) if !text.is_empty() && !suffix.is_empty() => {
                        let todo_text = text.trim();
                        let suffix = suffix.trim();
                        let todo_kind = TodoKind::Progress(suffix.to_string());
                        (todo_text, todo_kind)
                    }
                    _ => (input.trim(), TodoKind::default())
                }
            }
        }
    }

    pub fn state_check(&mut self) {
        let now = chrono::Local::now().naive_local();
        match self.kind {
            TodoKind::Once(date) => {
                if date == now.date() {
                    self.state = TodoState::OnGoing;
                } else if date < now.date() {
                    self.state = TodoState::Expired;
                } else {
                    self.state = TodoState::UpComing;
                }
            }
            TodoKind::Week(weekday) => {
                if weekday == now.weekday() {
                    self.state = TodoState::OnGoing;
                } else {
                    self.state = TodoState::UpComing;
                }
            }
            TodoKind::Month(day) => {
                if day == now.day() {
                    self.state = TodoState::OnGoing;
                } else {
                    self.state = TodoState::UpComing;
                }
            }
            _ => {}
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_kind_parse_test() {
        let input = [
            "do something awesome! -2024-12-22",
            "do something awesome! -Mon",
            "do something awesome! -24",
            "do something awesome! -SaT",
            "do something awesome! -243",
            "do something awesome! -abcdefg",
            ];
        assert_eq!(Todo::input_parse(input[0]).1, TodoKind::Once(NaiveDate::from_ymd_opt(2024, 12, 22).unwrap()));
        assert_eq!(Todo::input_parse(input[1]).1, TodoKind::Week(chrono::Weekday::Mon));
        assert_eq!(Todo::input_parse(input[2]).1, TodoKind::Month(24));
        assert_eq!(Todo::input_parse(input[3]).1, TodoKind::Week(chrono::Weekday::Sat));
        assert_eq!(Todo::input_parse(input[4]).1, TodoKind::default());
        assert_eq!(Todo::input_parse(input[5]).1, TodoKind::default());
    }
}
