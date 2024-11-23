use std::{default, io};
use crate::todo::Todo;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, List, Paragraph, Tabs, Widget},
    DefaultTerminal, Frame,
};
use tui_input::{backend::crossterm::EventHandler, Input as InputBuffer};

#[derive(Debug, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing
}

#[derive(Debug, Default)]
pub struct Input {
    input_buffer: InputBuffer,
    input_mode: InputMode,
    input_text: String
}

#[derive(Debug, Default)]
pub struct App {
    pub counter: i32,
    pub todo_list: Vec<Todo>,
    pub exit: bool,
    pub tab: AppTab,
    pub input: Input,
}


#[derive(Debug, Default)]
pub enum AppTab {
    #[default]
    Home,
    TodoList,
}

pub enum Message {
    Increment,
    Decrement,
    Quit,
    TabChange(AppTab),
    InputStart,
    InputEnd
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
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

    fn view(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(frame.area());
        frame.render_widget(self.render_tab_bar(), layout[0]);
        match self.tab {
            AppTab::Home => {
        
                frame.render_widget(self.render_main_window(), layout[1]);
            }
            AppTab::TodoList => {
                
                self.render_todo_list_window(frame, layout[1]);
            }
        }
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Decrement => self.counter -= 1,
            Message::Increment => self.counter += 1,
            Message::TabChange(tab) => self.tab = tab,
            Message::Quit => self.exit = true,
            Message::InputStart => self.input.input_mode = InputMode::Editing,
            Message::InputEnd => self.input.input_mode = InputMode::Normal,
        }
        None
    }
    fn handle_events(&mut self) -> io::Result<Option<Message>> {
        if let InputMode::Editing = self.input.input_mode {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => return Ok(Some(Message::InputEnd)),
                    //TODO 按下回车添加待办
                    _ => {self.input.input_buffer.handle_event(&Event::Key(key));}
                }
            }
            return Ok(None);
        }
        if event::poll(std::time::Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    let msg = match key_event.code {
                        KeyCode::Left => Some(Message::Increment),
                        KeyCode::Right => Some(Message::Decrement),
                        KeyCode::Esc => Some(Message::Quit),
                        KeyCode::Char('l') => Some(Message::TabChange(AppTab::TodoList)),
                        KeyCode::Char('i') => Some(Message::InputStart),
                        _ => None,
                    };
                    return Ok(msg);
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn render_tab_bar(&self) -> impl Widget {
        let title = Line::from(" *菜单* ").bold();
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::PLAIN);
        let items = vec![
            Text::from("tab1").centered(),
            Text::from("tab2"),
            Text::from("tab3")
        ];
        List::new(items)
            .block(block)
    }
    fn render_main_window(&self) -> impl Widget {
        let title = Line::from(" *待办管理-主页* ").bold();
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::PLAIN);
        let counter_text = Text::from(vec![Line::from(vec![
            "value ".into(),
            self.counter.to_string().yellow(),
        ])]);
        Paragraph::new(counter_text).centered().block(block)
    }
    fn render_todo_list_window(&self, frame: &mut Frame, rect: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(1)
            ])
            .split(rect);
        let title = Line::from(" *待办管理-列表* ").bold();
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::PLAIN)
            ;
        let width = rect.width.max(3) - 3;
        let scroll = self.input.input_buffer.visual_scroll(width.into());
        let input = Paragraph::new(self.input.input_buffer.value())
            .style(match self.input.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .scroll((0, scroll as u16))
            .block(block);
        frame.render_widget(input, layout[0]);
        if let InputMode::Editing = self.input.input_mode {
                // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                frame.set_cursor_position((
                    // Put cursor past the end of the input text
                    layout[0].x
                        + ((self.input.input_buffer.visual_cursor()).max(scroll) - scroll) as u16
                        + 1,
                    // Move one line down, from the border to the input line
                    layout[0].y + 1,
                ))
        };
    }
}
