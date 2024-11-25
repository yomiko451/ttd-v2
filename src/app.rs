use crate::todo::Todo;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border::{self, PLAIN},
    text::{self, Line, Text},
    widgets::{canvas::{Canvas, Line as CanvasLine, Map, MapResolution, Rectangle}, Block, List, ListState, Paragraph, Row, Table, TableState, Tabs, Widget},
    DefaultTerminal, Frame,
};
use std::{default, io};
use tui_input::{backend::crossterm::EventHandler, Input as InputBuffer};

#[derive(Debug, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

#[derive(Debug, Default)]
pub struct Input {
    buffer: InputBuffer,
    mode: InputMode,
}

#[derive(Debug, Default)]
pub struct App {
    pub todo_list: Vec<Todo>,
    pub exit: bool,
    pub tab: AppTab,
    pub tab_state: ListState,
    pub table_state: TableState,
    pub input: Input,
}

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub enum AppTab {
    #[default]
    Home,
    Todo,
    Info,
    Help
}

impl AppTab {
    fn get_tab_list() -> Vec<String> {
        vec![
            String::from("Home"),
            String::from("Todo"),
            String::from("Info"),
            String::from("Help"),
        ]
    }
}

pub enum Message {
    AddTodo,
    TabChange(AppTab),
    InputStart,
    InputEnd,
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
        self.tab_state.select_first();
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
                self.render_main_window(frame, layout[1]);
            }
            AppTab::Todo => {
                self.render_todo_list_window(frame, layout[1]);
            }
            AppTab::Info => {}
            AppTab::Help => {}
        }
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::AddTodo => {
                let input = self.input.buffer.value();
                if !input.is_empty() {
                    let todo = Todo::new(input);
                    self.todo_list.push(todo);
                }
                self.input.buffer.reset();
            }
            Message::TabChange(tab) => {
                self.tab = tab;
                let index = self.tab as usize;
                self.tab_state.select(Some(index));
            }
            Message::InputStart => {
                if self.tab == AppTab::Todo {
                    self.input.mode = InputMode::Editing;
                }
            }
            Message::InputEnd => self.input.mode = InputMode::Normal,
            Message::Quit => self.exit = true,
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
            }
            Message::SelectNext => {
                if let Some(index) = self.table_state.selected() {
                    if index + 1 == self.todo_list.len() {
                        self.table_state.select_first();
                    } else {
                        self.table_state.select_next();
                    }
                } else {
                    self.table_state.select_last();
                }
            }
        }
        None //TODO 状态机看看需不需要，不需要就删了精简代码
    }
    fn handle_events(&mut self) -> io::Result<Option<Message>> {
        if let InputMode::Editing = self.input.mode {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Esc => return Ok(Some(Message::InputEnd)),
                        KeyCode::Enter => return Ok(Some(Message::AddTodo)),
                        _ => {
                            self.input.buffer.handle_event(&Event::Key(key_event));
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
                        KeyCode::Esc => Some(Message::Quit),
                        KeyCode::Char('t') if self.tab != AppTab::Todo => Some(Message::TabChange(AppTab::Todo)),
                        KeyCode::Char('h') if self.tab != AppTab::Home => Some(Message::TabChange(AppTab::Home)),
                        KeyCode::Char('i') if self.tab == AppTab::Todo => Some(Message::InputStart),
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
    fn render_main_window(&self, frame: &mut Frame, rect: Rect) {
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
    fn render_todo_list_window(&mut self, frame: &mut Frame, rect: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Min(1)])
            .split(rect);
        let title = Line::from(" InputEdit ").bold();
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::PLAIN);
        let width = rect.width.max(3) - 3;
        let scroll = self.input.buffer.visual_scroll(width.into());
        let input = Paragraph::new(self.input.buffer.value())
            .style(match self.input.mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Cyan),
            })
            .scroll((0, scroll as u16))
            .block(block);
        frame.render_widget(input, layout[0]);
        if let InputMode::Editing = self.input.mode {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            frame.set_cursor_position((
                // Put cursor past the end of the input text
                layout[0].x
                    + ((self.input.buffer.visual_cursor()).max(scroll) - scroll) as u16
                    + 1,
                // Move one line down, from the border to the input line
                layout[0].y + 1,
            ))
        };
        let table_title = Line::from(" TodoList ").bold();
        let table_block = Block::bordered()
            .title(table_title.centered())
            .border_set(PLAIN);
        let table = self
            .todo_list
            .iter()
            .enumerate()
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
                Row::new(vec!["Index", "Content", "Kind", "State", "CreatedAt"])
                    .style(Style::new().bold())
                    .bottom_margin(1),
            )
            .footer(Row::new([
                format!("Total: {}", self.todo_list.len()),
                format!("Filtered: 0"), //TODO 筛选
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
}
