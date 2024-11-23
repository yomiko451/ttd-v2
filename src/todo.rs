#[derive(Debug, Default)]
pub struct Todo {
    text: String,
    kind: TodoKind,
    date: TodoDate,
    state: TodoState,
}

#[derive(Debug, Default)]
pub enum TodoState {
    #[default]
    Unspecified,
    OnGoing,
    UpComing,
}

#[derive(Debug, Default)]
pub struct TodoDate {
    year: u16, //试试chrono自带的date
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

#[derive(Debug, Default)]
pub enum TodoKind {
    #[default]
    Unspecified,
    Progress,
    Week,
    Month,
    Year,
    Once,
}