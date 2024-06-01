
use std::io::stdout;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend}, layout::{Constraint, Direction, Layout}, text::Text, Frame, Terminal
};
#[derive(Debug, Clone)]
enum States {
    Menu,
    Game,
}
#[derive(Debug, Clone)]
enum Choice {
    Exit,
    Start,
    Another(String),
}
impl Choice {
    fn new(str: &str) -> Self {
        Choice::Another(str.to_string())
    }
    fn to_string(&self) -> String {
        match self.clone() {
            Choice::Start => String::from("Start"),
            Choice::Exit => String::from("Exit"),
            Choice::Another(str) => str,
        }
    }
}

struct Choices {
    vars: Vec<Choice>,
    select: usize,
}
impl Choices {
    fn new(vars: Vec<Choice>) -> Self {
        Self {
            vars,
            select: 0,
        }
    }
    fn up(&mut self) {
        if self.select > 0 {
            self.select -= 1;
        }
    }
    fn down(&mut self) {
        if self.select < self.vars.len()-1 {
            self.select += 1;
        }
    }
    fn get(&self) -> Choice{
        self.vars[self.select].clone()
    }
}

struct Screen {
    question: String,
    choice: Choices,
    state: States,
}
impl Screen {
    fn new(vars: Vec<Choice>, state: States) -> Self {
        Self {
            question: "JPGAME".to_string(),
            choice: Choices::new(vars),
            state,
        }
    }
    fn show(&self) {
        for (i, choice) in self.choice.vars.iter().enumerate() {
            if i == self.choice.select {
                println!("{}. [{:?}]", i+1, choice.to_string());
            } else {
                println!("{}. {:?}", i+1, choice.to_string());
            }
        }
    }
    fn start(&mut self) {
        self.state = States::Game;
        self.choice.vars = vec![Choice::new("Hello"), Choice::new("World")];
    }
    fn update(&mut self) {

    }
    fn draw(&self, f: &mut Frame) {
        let full_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(f.size());
        let question_widget = Text::raw(&self.question).centered();

        f.render_widget(question_widget, full_layout[0]);
    }
}

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let mut t = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut screen = Screen::new(vec![Choice::Start, Choice::Exit], States::Menu);

    let res = run(&mut t, &mut screen);

    disable_raw_mode()?;
    execute!(t.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    t.show_cursor()?;

    res?;
    Ok(())
}
fn run<B: Backend>(t: &mut Terminal<B>, screen: &mut Screen) -> anyhow::Result<()> {
    loop {
        t.draw(|f| screen.draw(f))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }
    Ok(())
}

