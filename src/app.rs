use crate::{
    todo::{Todo, TodoKind, TODAY},
    SyncKind, SyncState,
};
use chrono::Datelike;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border::{self, PLAIN},
    text::Line,
    widgets::{
        canvas::{Canvas, Map, MapResolution},
        Block, List, ListState, Paragraph, Row, Table, TableState,
    },
    DefaultTerminal, Frame,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, io, path::PathBuf, sync::{mpsc, Arc, LazyLock, Mutex, RwLock}};
use tui_input::{backend::crossterm::EventHandler, Input as InputBuffer};

pub static CURRENT_PATH: LazyLock<PathBuf> = LazyLock::new(|| std::env::current_dir().unwrap());

pub static TODO_LIST_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| CURRENT_PATH.join("todo_list.json"));

pub static SYNC_STATE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| std::env::current_dir().unwrap().join("sync_state.json"));

#[derive(Debug, Default, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
}

#[derive(Debug, Default)]
pub struct App {
    pub todo_list: Arc<RwLock<Vec<Todo>>>,
    pub exit: bool,
    pub tab: AppTab,
    pub tab_state: ListState,
    pub table_state: TableState,
    pub input_buffer: InputBuffer,
    pub input_mode: InputMode,
    pub sync_state: Arc<RwLock<SyncState>>,
    pub update_cache: Option<String>,
}

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub enum AppTab {
    #[default]
    Home,
    Todo,
    Sync,
    About,
}

impl AppTab {
    fn get_tab_list() -> Vec<String> {
        vec![
            String::from("Home(H)"),
            String::from("Todo(T)"),
            String::from("Sync(S)"),
            String::from("About(A)"),
        ]
    }
}

pub enum Message {
    Add,
    Delete,
    Save,
    Update,
    Filter(TodoKind),
    FilterReset,
    TabChange(AppTab),
    InputModeChange(InputMode),
    SelectPrevious,
    SelectNext,
    Quit,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        //初始化
        self.init();
        //主循环
        while !self.exit {
            //根据数据渲染页面
            terminal.draw(|frame| self.view(frame))?;
            //根据用户事件生成消息
            let mut current_msg = self.handle_events()?;
            //根据消息更新数据
            while let Some(msg) = current_msg {
                current_msg = self.update(msg);
            }
        }
        Ok(())
    }

    fn init(&mut self) {
        if !TODO_LIST_PATH.exists() {
            std::fs::File::create(TODO_LIST_PATH.as_path()).unwrap();
        }
        if !SYNC_STATE_PATH.exists() {
            std::fs::File::create(SYNC_STATE_PATH.as_path()).unwrap();
        }
        self.load_todo_list();
        self.tab_state.select_first();
        self.todo_list.write().unwrap().iter_mut().for_each(Todo::state_check);
        self.sync_data();
    }
    //view方法只负责渲染，尽量不要在这里修改全局数据，启用可变引用只是为了满足状态渲染函数的参数要求
    fn view(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(15), Constraint::Percentage(95)])
            .split(frame.area());
        self.render_tab_bar(frame, layout[0]);
        match self.tab {
            AppTab::Home => {
                self.render_home_window(frame, layout[1]);
            }
            AppTab::Todo => {
                self.render_todo_window(frame, layout[1]);
            }
            AppTab::Sync => {
                self.render_sync_window(frame, layout[1]);
            }
            AppTab::About => {}
        }
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Add => {
                let input = self.input_buffer.value();
                if !input.is_empty() {
                    let todo = Todo::new(input);
                    self.todo_list.write().unwrap().push(todo);
                    if let Some(ref created_at) = self.update_cache {
                        self.todo_list.write().unwrap().retain(|todo| todo.created_at != *created_at);
                        self.update_cache = None;
                        self.table_state.select_last();
                    }
                    self.input_buffer.reset();
                }
                Some(Message::Save)
            }
            Message::Save => {
                {let mut sync_state = self.sync_state.write().unwrap();
                self.todo_list.write().unwrap().iter_mut().for_each(Todo::reset_hidden_flag);
                sync_state.last_save_at = chrono::Local::now().naive_local();
                sync_state.last_sync_kind = SyncKind::LocalSave;}
                self.save_todo_list();
                None
            }
            Message::Delete => {
                if let Some(index) = self.table_state.selected() {
                    self.todo_list.write().unwrap().remove(index);
                }
                Some(Message::Save)
            }
            Message::Update => {
                if let Some(index) = self.table_state.selected() {
                    let todo: Todo;
                    {let todo_list = self.todo_list.read().unwrap();
                    todo = 
                        todo_list
                        .iter()
                        .filter(|todo| !todo.is_hidden)
                        .cloned()
                        .nth(index)
                        .unwrap();}
                    let value = match todo.kind {
                        TodoKind::General => todo.text.clone(),
                        TodoKind::Week(week) => format!("{} - {}", todo.text, week),
                        TodoKind::Month(day) => format!("{} - {}", todo.text, day),
                        TodoKind::Once(date) => format!("{} - {}", todo.text, date),
                        TodoKind::Progress(ref progress) => format!("{} @ {}", todo.text, progress),
                    };
                    self.input_buffer = self.input_buffer.clone().with_value(value);
                    self.input_mode = InputMode::Insert;
                    self.update_cache = Some(todo.created_at.clone());
                }
                None
            }
            Message::TabChange(tab) => {
                self.tab = tab;
                let index = self.tab as usize;
                self.tab_state.select(Some(index));
                None
            }
            Message::InputModeChange(input_mode) => {
                if input_mode == InputMode::Normal {
                    self.input_buffer.reset();
                    self.update_cache = None;
                }
                if self.tab == AppTab::Todo {
                    self.input_mode = input_mode;
                }
                None
            }
            Message::Quit => {
                //TODO询问是否同步
                self.save_todo_list();
                self.exit = true;
                None
            }
            Message::SelectPrevious => {
                if let Some(index) = self.table_state.selected() {
                    if index == 0 {
                        self.table_state.select_last();
                    } else {
                        self.table_state.select_previous();
                    }
                } else {
                    self.table_state.select_first();
                }
                None
            }
            Message::SelectNext => {
                if let Some(index) = self.table_state.selected() {
                    if index + 1 == self.todo_list.read().unwrap().len() {
                        self.table_state.select_first();
                    } else {
                        self.table_state.select_next();
                    }
                } else {
                    self.table_state.select_last();
                }
                None
            }
            Message::Filter(todo_kind) => {
                self.todo_list.write().unwrap().iter_mut().for_each(|todo| {
                    todo.is_hidden =
                        !(std::mem::discriminant(&todo_kind) == std::mem::discriminant(&todo.kind));
                });
                None
            }
            Message::FilterReset => {
                self.todo_list.write().unwrap().iter_mut().for_each(Todo::reset_hidden_flag);
                None
            }
        }
    }
    fn handle_events(&mut self) -> io::Result<Option<Message>> {
        if let InputMode::Insert = self.input_mode {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Esc => {
                            return Ok(Some(Message::InputModeChange(InputMode::Normal)))
                        }
                        KeyCode::Enter => return Ok(Some(Message::Add)),
                        _ => {
                            self.input_buffer.handle_event(&Event::Key(key_event));
                        }
                    }
                }
                _ => {}
            }
            return Ok(None);
        }
        if event::poll(std::time::Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    let msg = match key_event.code {
                        KeyCode::Char('q') => Some(Message::Quit), //TODO 大写也要考虑
                        KeyCode::Char('t') if self.tab != AppTab::Todo => {
                            Some(Message::TabChange(AppTab::Todo))
                        }
                        KeyCode::Char('h') if self.tab != AppTab::Home => {
                            Some(Message::TabChange(AppTab::Home))
                        }
                        KeyCode::Char('s') if self.tab != AppTab::Sync => {
                            Some(Message::TabChange(AppTab::Sync))
                        }
                        KeyCode::Char('d') if self.tab == AppTab::Todo => Some(Message::Delete),
                        KeyCode::Char('g') if self.tab == AppTab::Todo => {
                            Some(Message::Filter(TodoKind::General))
                        }
                        KeyCode::Char('w') if self.tab == AppTab::Todo => {
                            Some(Message::Filter(TodoKind::Week(TODAY.weekday())))
                        }
                        KeyCode::Char('m') if self.tab == AppTab::Todo => {
                            Some(Message::Filter(TodoKind::Month(TODAY.day())))
                        }
                        KeyCode::Char('o') if self.tab == AppTab::Todo => {
                            Some(Message::Filter(TodoKind::Once(TODAY.date())))
                        }
                        KeyCode::Char('p') if self.tab == AppTab::Todo => {
                            Some(Message::Filter(TodoKind::Progress(String::default())))
                        }
                        KeyCode::Char('r') if self.tab == AppTab::Todo => {
                            Some(Message::FilterReset)
                        }
                        KeyCode::Char('u') if self.tab == AppTab::Todo => Some(Message::Update),
                        KeyCode::Char('i') if self.tab == AppTab::Todo => {
                            Some(Message::InputModeChange(InputMode::Insert))
                        }
                        KeyCode::Up if self.tab == AppTab::Todo => Some(Message::SelectPrevious),
                        KeyCode::Down if self.tab == AppTab::Todo => Some(Message::SelectNext),
                        _ => None,
                    };
                    return Ok(msg);
                }
                _ => {}
            }
        }
        Ok(None)
    }
    fn render_tab_bar(&mut self, frame: &mut Frame, rect: Rect) {
        let title = Line::from(" Tab ").bold();
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(Line::from(" Quit <q> ").centered())
            .border_set(border::PLAIN);
        let list = AppTab::get_tab_list()
            .into_iter()
            .map(|tab| Line::from(tab))
            .collect::<List>()
            .highlight_style(Style::new().italic().cyan())
            .highlight_symbol(" >> ")
            .block(block);
        frame.render_stateful_widget(list, rect, &mut self.tab_state);
    }
    fn render_home_window(&self, frame: &mut Frame, rect: Rect) {
        let title = Line::from(" Home ").bold();
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::PLAIN);
        let canvas = Canvas::default()
            .block(block)
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
            .paint(|ctx| {
                ctx.draw(&Map {
                    resolution: MapResolution::High,
                    color: Color::default(),
                });
                //ctx.layer(); TODO 继续画记得先保存当前状态
            });
        frame.render_widget(canvas, rect);
    }
    fn render_todo_window(&mut self, frame: &mut Frame, rect: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Min(1)])
            .split(rect);
        let block = Block::bordered()
            .title(Line::from(" InputEdit ").bold().centered())
            .title_bottom(
                Line::from(vec![
                    " Insert <I>".into(),
                    " Normal <Esc>".into(),
                    " Add <Enter> ".into(),
                ])
                .centered(),
            )
            .border_set(border::PLAIN);
        let width = rect.width.max(3) - 3;
        let scroll = self.input_buffer.visual_scroll(width.into());
        let input = Paragraph::new(self.input_buffer.value())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Insert => Style::default().fg(Color::Cyan),
            })
            .scroll((0, scroll as u16))
            .block(block);
        frame.render_widget(input, layout[0]);
        if let InputMode::Insert = self.input_mode {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            frame.set_cursor_position((
                // Put cursor past the end of the input text
                layout[0].x + ((self.input_buffer.visual_cursor()).max(scroll) - scroll) as u16 + 1,
                // Move one line down, from the border to the input line
                layout[0].y + 1,
            ))
        };
        let table_block = Block::bordered()
            .title(Line::from(" TodoList ").bold().centered())
            .title_bottom(
                Line::from(vec![
                    " Next <Down>".into(),
                    " Previous <Up>".into(),
                    " Delete <d>".into(),
                    " Update <u> ".into(),
                    " Filter <g, w, m, o, p, r> ".into(),
                ])
                .centered(),
            )
            .border_set(PLAIN);
        let todo_list = self.todo_list.read().unwrap();
        let table = 
            todo_list
            .iter()
            .enumerate()
            .filter(|(_, todo)| !todo.is_hidden)
            .map(|(index, todo)| -> Row {
                Row::new([
                    (index + 1).to_string(),
                    todo.text.clone(),
                    todo.kind.print_info(),
                    todo.state.print_info(),
                    todo.created_at.clone(),
                ])
            })
            .collect::<Table>()
            .header(
                Row::new(["Index", "Content", "Kind", "State", "CreatedAt"])
                    .style(Style::new().bold().underlined())
                    .top_margin(1)
                    .bottom_margin(1),
            )
            .footer(Row::new([
                format!("Total: {}", todo_list.len()),
                format!(
                    "Filtered: {}",
                    todo_list.iter().filter(|todo| !todo.is_hidden).count()
                ),
                format!("State: unsynced"), //TODO 到时候换个地方
            ]))
            .row_highlight_style(Style::new().reversed())
            .widths([
                Constraint::Percentage(10),
                Constraint::Percentage(35),
                Constraint::Percentage(20),
                Constraint::Percentage(15),
                Constraint::Percentage(20),
            ])
            .block(table_block); //TODO 文本多行显示
        frame.render_stateful_widget(table, layout[1], &mut self.table_state);
    }

    fn render_sync_window(&mut self, frame: &mut Frame, rect: Rect) {
        let p = Paragraph::new(format!("{}", self.sync_state.read().unwrap().last_save_at))
            .block(Block::bordered().border_set(border::PLAIN));
        frame.render_widget(p, rect);
    }
    fn save_todo_list(&mut self) {
        let todo_list_file = std::fs::File::create(TODO_LIST_PATH.as_path()).unwrap();
        let sync_log_file = std::fs::File::create(SYNC_STATE_PATH.as_path()).unwrap();
        serde_json::to_writer(todo_list_file, &self.todo_list.read().unwrap().clone()).unwrap();
        serde_json::to_writer(sync_log_file, &self.sync_state.read().unwrap().clone()).unwrap();
    }
    fn load_todo_list(&mut self) {
        let todo_list_file = std::fs::read(TODO_LIST_PATH.as_path()).unwrap();
        if !todo_list_file.is_empty() {
            *self.todo_list.write().unwrap() = serde_json::from_slice(&todo_list_file).unwrap();
        }
        self.todo_list.write().unwrap().iter_mut().for_each(Todo::state_check);
        let sync_log_file = std::fs::read(SYNC_STATE_PATH.as_path()).unwrap();
        if !sync_log_file.is_empty() {
            *self.sync_state.write().unwrap() = serde_json::from_slice(&sync_log_file).unwrap();
        }
    }

    fn sync_data(&mut self) {
        let sync_state = Arc::clone(&self.sync_state);
        let todo_list = Arc::clone(&self.todo_list);
        let local_sync_state = sync_state.read().unwrap().clone();
        let local_todo_list = todo_list.read().unwrap().clone();
        std::thread::spawn(move || match crate::sync_app_data(local_sync_state, local_todo_list) {
            Ok(Some((server_sync_state, server_todo_list))) => {
                *sync_state.write().unwrap() = server_sync_state; 
                *todo_list.write().unwrap() = server_todo_list; 
                //self.save_todo_list();  TODO
            }
            _ => {}
        }); 

        
    }
}
