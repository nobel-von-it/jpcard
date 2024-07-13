use core::fmt;
use std::io::stdout;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};

const GAME_NAME: &str = "JP WORDS GAME";

#[derive(Debug, Clone, PartialEq, Eq)]
enum States {
    Menu,
    Game,
    Exiting,
}
#[derive(Debug, Clone)]
enum Choice {
    // options
    Exit,
    Start,
    Yes,
    No,
    // dynamic variants
    Vars(String),
}
impl fmt::Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Choice::Start => write!(f, "Start"),
            Choice::Exit => write!(f, "Exit"),
            Choice::Yes => write!(f, "Yes"),
            Choice::No => write!(f, "No"),
            Choice::Vars(str) => write!(f, "{str}"),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
struct Dictionary {
    words: Vec<Word>,
}
#[derive(Debug, Serialize, Deserialize)]
struct Word {
    word: String,
    vars: Vec<String>,
    correct: usize,
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

struct Choices {
    vars: Vec<Choice>,
    select: usize,
    correct: usize,
}
impl Choices {
    fn new(vars: Vec<Choice>) -> Self {
        Self {
            vars,
            select: 0,
            correct: 0,
        }
    }
    fn up(&mut self) {
        if self.select > 0 {
            self.select -= 1;
        }
    }
    fn down(&mut self) {
        if self.select < self.vars.len() - 1 {
            self.select += 1;
        }
    }
    fn get(&self) -> Choice {
        self.vars[self.select].clone()
    }
}

struct Screen {
    question: String,
    choice: Choices,
    state: States,
    score: usize,
}
impl Screen {
    fn new(vars: Vec<Choice>, state: States) -> Self {
        Self {
            question: "Hello".to_string(),
            choice: Choices::new(vars),
            state,
            score: 0,
        }
    }
    fn start(&mut self) {
        self.state = States::Game;
        self.update();
    }
    fn compare(&mut self) {
        if self.choice.select == self.choice.correct {
            self.score += 1;
        }
    }
    // read a word and variants from file
    fn update(&mut self) {
        if self.state != States::Game {
            return;
        }

        // file:
        // jp_word1:1var,2var,3var...:correct_ansver uint
        // jp_word2:1var,2var,3var...:correct_ansver uint
        // ...: ...: ...
        //
        // json:
        // words: [word1, word2]
        // this is the simplest way to parse data

        let dictionary: Dictionary =
            serde_json::from_reader(std::fs::File::open("ex.json").unwrap()).unwrap();
        let rl = rand::thread_rng().gen_range(0..dictionary.words.len());

        // add shuffle variants
        if let Some(word) = dictionary.words.get(rl) {
            self.question.clone_from(&word.word);
            self.choice.vars = word
                .vars
                .iter()
                .map(|var| Choice::Vars(var.to_string()))
                .collect();
            self.choice.select = 0;
            self.choice.correct = word.correct;
        }
    }
    fn draw(&self, f: &mut Frame) {
        match self.state {
            States::Exiting => {
                let center_exit = centered_rect(30, 30, f.size());
                let exit_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(100),
                        Constraint::Length(3),
                        Constraint::Length(3),
                    ])
                    .split(center_exit);
                let exit_block = Block::default().title("Exit").borders(Borders::ALL);
                let exit_text = Text::raw("Are you sure?").centered();
                let yes_button = Text::raw(Choice::Yes.to_string()).centered();
                let no_button = Text::raw(Choice::No.to_string()).centered();

                f.render_widget(exit_block, center_exit);
                f.render_widget(exit_text, exit_layout[0]);
                f.render_widget(yes_button, exit_layout[1]);
                f.render_widget(no_button, exit_layout[2]);
            }
            _ => {
                let full_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Percentage(30),
                        Constraint::Percentage(70),
                    ])
                    .split(f.size());
                let score_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(100),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ])
                    .split(
                        Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([
                                Constraint::Length(1),
                                Constraint::Length(3),
                                Constraint::Percentage(100),
                            ])
                            .split(full_layout[2])[1],
                    );
                let mut constraints = vec![];
                for _ in self.choice.vars.iter() {
                    constraints.push(Constraint::Length(2));
                }
                let choice_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(full_layout[2]);

                let score_widget = Text::raw(self.score.to_string()).centered();
                let question_widget = Text::raw(&self.question).centered();
                let full_widget =
                    Paragraph::new("").block(Block::new().title(GAME_NAME).borders(Borders::ALL));

                f.render_widget(score_widget, score_layout[1]);
                f.render_widget(full_widget, f.size());
                f.render_widget(question_widget, full_layout[1]);
                for (i, choice) in self.choice.vars.iter().enumerate() {
                    let tmp = if i == self.choice.select {
                        format!("{}. [{}]", i + 1, choice)
                    } else {
                        format!("{}. {}", i + 1, choice)
                    };
                    f.render_widget(Text::raw(tmp).centered(), choice_layout[i]);
                }
            }
        }
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
                KeyCode::Esc => screen.state = States::Exiting,
                KeyCode::Up => screen.choice.up(),
                KeyCode::Down => screen.choice.down(),
                KeyCode::Enter => match screen.choice.get() {
                    Choice::Exit | Choice::Yes => break,
                    Choice::Start | Choice::No => screen.start(),
                    Choice::Vars(_) => {
                        screen.compare();
                        screen.update();
                    }
                },
                _ => {}
            }
        }
    }
    Ok(())
}
