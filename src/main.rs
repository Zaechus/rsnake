use std::{
    collections::VecDeque,
    io::{Write, stdout},
    process::ExitCode,
    sync::mpsc,
    thread,
    time::Duration,
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute, queue,
    style::{Color, Print, SetForegroundColor},
    terminal,
    tty::IsTty,
};
use rand::Rng;

const WIDTH: u16 = 32;
const HEIGHT: u16 = 16;
const INTERVAL: u64 = 200;

enum Direction {
    Left,
    Down,
    Up,
    Right,
}

struct Snek {
    pub x: u16,
    pub y: u16,
}

impl Snek {
    fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

fn main() -> ExitCode {
    let mut stdout = stdout();
    if !stdout.is_tty() {
        return ExitCode::FAILURE;
    }

    terminal::enable_raw_mode().unwrap();
    queue!(
        stdout,
        cursor::Hide,
        terminal::Clear(terminal::ClearType::All)
    )
    .unwrap();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop {
            if event::poll(Duration::from_millis(50)).unwrap()
                && let Event::Key(KeyEvent {
                    code,
                    kind: KeyEventKind::Press,
                    ..
                }) = event::read().unwrap()
            {
                tx.send(code).unwrap();
            }
        }
    });

    let mut rng = rand::rng();

    let mut snake = VecDeque::with_capacity(3);
    snake.push_back(Snek::new(WIDTH / 2 + 1, HEIGHT / 2 + 1));
    snake.push_back(Snek::new(snake[0].x, snake[0].y + 1));
    snake.push_back(Snek::new(snake[0].x, snake[0].y + 2));
    let mut direction = Direction::Up;
    let mut apple = (WIDTH / 4, HEIGHT / 4);

    // print border
    queue!(stdout, cursor::MoveTo(0, 0)).unwrap();
    for _ in 0..=WIDTH {
        write!(stdout, "-").unwrap();
    }
    queue!(stdout, cursor::MoveTo(0, 0)).unwrap();
    for _ in 1..HEIGHT {
        queue!(
            stdout,
            cursor::MoveDown(1),
            Print("|"),
            cursor::MoveRight(WIDTH - 1),
            Print("|"),
            cursor::MoveLeft(WIDTH + 1),
        )
        .unwrap();
    }
    queue!(stdout, cursor::MoveDown(1)).unwrap();
    for _ in 0..=WIDTH {
        write!(stdout, "-").unwrap();
    }

    queue!(stdout, SetForegroundColor(Color::Green)).unwrap();
    for snek in snake.iter_mut().skip(1) {
        queue!(stdout, cursor::MoveTo(snek.x, snek.y), Print('@')).unwrap();
    }

    loop {
        let sleeps = thread::spawn(move || {
            thread::sleep(Duration::from_millis(INTERVAL));
        });

        let snake_x = snake[0].x;
        let snake_y = snake[0].y;
        execute!(
            stdout,
            cursor::MoveTo(apple.0, apple.1),
            SetForegroundColor(Color::Red),
            Print('a'),
            cursor::MoveTo(snake_x, snake_y),
            SetForegroundColor(Color::Green),
            Print('@')
        )
        .unwrap();

        for snek in snake.iter_mut().skip(1) {
            if snake_x == snek.x && snake_y == snek.y {
                return quit();
            }
        }

        if snake_x == 0 || snake_y == 0 || snake_x == WIDTH || snake_y == HEIGHT {
            break;
        } else if snake_x == apple.0 && snake_y == apple.1 {
            snake.push_front(Snek::new(apple.0, apple.1));
            apple = (rng.random_range(1..WIDTH), rng.random_range(1..HEIGHT));
        }

        while !sleeps.is_finished() {
            if let Ok(code) = rx.try_recv() {
                match code {
                    KeyCode::Char('h') | KeyCode::Char('a') | KeyCode::Left => match direction {
                        Direction::Down | Direction::Up => {
                            direction = Direction::Left;
                            break;
                        }
                        _ => (),
                    },
                    KeyCode::Char('j') | KeyCode::Char('s') | KeyCode::Down => match direction {
                        Direction::Left | Direction::Right => {
                            direction = Direction::Down;
                            break;
                        }
                        _ => (),
                    },
                    KeyCode::Char('k') | KeyCode::Char('w') | KeyCode::Up => match direction {
                        Direction::Left | Direction::Right => {
                            direction = Direction::Up;
                            break;
                        }
                        _ => (),
                    },
                    KeyCode::Char('l') | KeyCode::Char('d') | KeyCode::Right => match direction {
                        Direction::Down | Direction::Up => {
                            direction = Direction::Right;
                            break;
                        }
                        _ => (),
                    },
                    KeyCode::Char('p') | KeyCode::Char(' ') | KeyCode::Esc => loop {
                        if let Ok(code) = rx.recv() {
                            match code {
                                KeyCode::Char('p') | KeyCode::Char(' ') | KeyCode::Esc => break,
                                KeyCode::Char('q') => return quit(),
                                _ => (),
                            }
                        }
                    },
                    KeyCode::Char('q') => return quit(),
                    _ => (),
                }
            }
        }

        match direction {
            Direction::Left => snake.push_front(Snek::new(snake_x - 1, snake_y)),
            Direction::Down => snake.push_front(Snek::new(snake_x, snake_y + 1)),
            Direction::Up => snake.push_front(Snek::new(snake_x, snake_y - 1)),
            Direction::Right => snake.push_front(Snek::new(snake_x + 1, snake_y)),
        }
        if let Some(tail) = snake.back() {
            queue!(stdout, cursor::MoveTo(tail.x, tail.y), Print(' ')).unwrap();
        }
        snake.pop_back();
    }

    quit()
}

fn quit() -> ExitCode {
    execute!(
        stdout(),
        SetForegroundColor(Color::Reset),
        cursor::MoveTo(WIDTH, HEIGHT),
        cursor::Show
    )
    .unwrap();
    terminal::disable_raw_mode().unwrap();
    ExitCode::SUCCESS
}
