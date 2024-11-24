use std::io;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = ttd_v2::App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}
