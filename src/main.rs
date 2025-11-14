use std::{io::stdout, process::ExitCode, sync::mpsc, thread, time::Duration};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute, queue,
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode},
    tty::IsTty,
};
use rand::Rng;

const WIDTH: u16 = 32;
const HEIGHT: u16 = 16;
const INTERVAL: u64 = 200;

#[derive(Clone, Copy)]
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

    enable_raw_mode().unwrap();
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

    let mut snake = vec![
        Snek::new(WIDTH / 2 + 1, HEIGHT / 2 + 1),
        Snek::new(WIDTH / 2 + 1, HEIGHT / 2 + 2),
        Snek::new(WIDTH / 2 + 1, HEIGHT / 2 + 3),
    ];
    let mut direction = Direction::Up;
    let mut apple = (WIDTH / 4, HEIGHT / 4);

    loop {
        let sleeps = thread::spawn(move || {
            thread::sleep(Duration::from_millis(INTERVAL));
        });

        queue!(stdout, cursor::MoveTo(0, 0)).unwrap();
        for _ in 0..=WIDTH {
            print!("-");
        }
        queue!(stdout, cursor::MoveTo(0, 0)).unwrap();
        for _ in 1..HEIGHT {
            queue!(
                stdout,
                cursor::MoveDown(1),
                terminal::Clear(terminal::ClearType::CurrentLine),
                Print("|"),
                cursor::MoveRight(WIDTH - 1),
                Print("|"),
                cursor::MoveLeft(WIDTH + 1),
            )
            .unwrap();
        }
        queue!(stdout, cursor::MoveDown(1)).unwrap();
        for _ in 0..=WIDTH {
            print!("-");
        }
        let snake_x = snake[0].x;
        let snake_y = snake[0].y;
        queue!(
            stdout,
            cursor::MoveTo(apple.0, apple.1),
            Print('a'),
            cursor::MoveTo(snake_x, snake_y),
            Print('@')
        )
        .unwrap();
        for snek in snake.iter_mut().skip(1) {
            queue!(stdout, cursor::MoveTo(snek.x, snek.y), Print('@')).unwrap();
            if snake_x == snek.x && snake_y == snek.y {
                return quit();
            }
        }
        execute!(stdout).unwrap();

        if snake[0].x == 0 || snake[0].y == 0 || snake[0].x == WIDTH || snake[0].y == HEIGHT {
            break;
        } else if snake[0].x == apple.0 && snake[0].y == apple.1 {
            snake.insert(0, Snek::new(apple.0, apple.1));
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
            Direction::Left => snake.insert(0, Snek::new(snake[0].x - 1, snake[0].y)),
            Direction::Down => snake.insert(0, Snek::new(snake[0].x, snake[0].y + 1)),
            Direction::Up => snake.insert(0, Snek::new(snake[0].x, snake[0].y - 1)),
            Direction::Right => snake.insert(0, Snek::new(snake[0].x + 1, snake[0].y)),
        }
        snake.pop();
    }

    quit()
}

fn quit() -> ExitCode {
    disable_raw_mode().unwrap();
    execute!(stdout(), cursor::MoveTo(WIDTH, HEIGHT), cursor::Show).unwrap();
    ExitCode::SUCCESS
}
