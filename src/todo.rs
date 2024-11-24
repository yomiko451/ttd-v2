use chrono::{NaiveDate, Weekday};

#[derive(Debug, Default)]
pub struct Todo {
    pub text: String,
    pub created_at: String,
    pub kind: TodoKind,
    pub state: TodoState,
}

#[derive(Debug, Default)]
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

#[derive(Debug, Default, PartialEq)]
pub enum TodoKind {
    #[default]
    General,
    Progress(String),
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
        Todo {
            text: todo_text.to_string(),
            created_at: Self::get_time(),
            kind: todo_kind,
            state: TodoState::default(),//TODO 和当前比较一下
        }
    }

    fn get_time() -> String {
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }

    fn input_parse(input: &str) -> (&str, TodoKind) {
        match input.split_once('-') {
            Some((text, suffix)) => {
                if !suffix.is_empty() {
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
                    (todo_text.trim(), todo_kind)
    
                } else {
                    (text.trim(), TodoKind::default())
                }
            }
            None => (input.trim(), TodoKind::default()),
        }
    }
}



mod tests {
    use std::time::Duration;

    use chrono::Datelike;

    use super::*;

    #[test]
    fn test_date() {
        let d1 = chrono::Local::now().naive_local();
        let m1 = d1.month();
        let day1 = d1.day();
        let w1 = d1.weekday();
        let date = d1.date();
        std::thread::sleep(Duration::from_secs(1));
        let d2 = chrono::Local::now().naive_local();
        let m2 = d2.month();
        assert!(d2 > d1);
        assert!(m1 == m2);
        println!("{}", w1);
        println!("{}", day1);
        println!("{}", date);
    }

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
