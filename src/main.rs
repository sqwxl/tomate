#![allow(dead_code)]
use chrono::prelude::*;
use chrono::Duration;
use clap::Parser;
use core::fmt;
use std::io::Write;
use directories::ProjectDirs;
use notify_rust::Notification;
use std::fmt::Display;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process;

fn main() {
    // let args = Args::parse();
    let cfg = Config::default();

    let mut tomate = Tomate::new(Utc::now(), &cfg);

    let _record = match Record::read(&cfg.record_path) {
        Ok(r) => Some(r),
        Err(_) => None,
    };
    // todo use values from record

    loop {
        tomate = tomate.next();
        print!("\r{}", tomate.describe());
        std::io::stdout().flush().unwrap();
        std::thread::sleep(std::time::Duration::new(1, 0));
    }
}

#[derive(Parser)]
struct Args {
    // todo
}

/// User options
#[derive(Debug, Clone)]
struct Config {
    work_duration: Duration,
    short_break_duration: Duration,
    long_break_duration: Duration,
    blocks_n: u32,
    auto_start: bool,
    auto_start_session: bool,
    auto_start_break: bool,
    record_path: PathBuf,
}
impl Default for Config {
    fn default() -> Self {
        let mut record_path = Path::new("").to_path_buf();
        if let Some(proj_dirs) = ProjectDirs::from("", "", "tomate") {
            record_path = proj_dirs.data_dir().to_path_buf();
        }
        Config {
            work_duration: Duration::minutes(25),
            short_break_duration: Duration::minutes(5),
            long_break_duration: Duration::minutes(15),
            blocks_n: 4,
            auto_start: true,
            auto_start_session: true,
            auto_start_break: true,
            record_path,
        }
    }
}

/// Record contains all the information needed to reproduce a Tomate application
/// state, as well as some usage statistics. Used when writing/reading to file.
#[derive(Debug, Clone, Copy)]
struct Record {
    blocks: u32,
    sessions: u32,
    total_session_time: Duration,
}
impl Record {
    fn read(file_path: &Path) -> Result<Record, io::Error> {
        match fs::read_to_string(file_path) {
            Ok(s) => {
                let l = s
                    .lines()
                    .map(|l| l.trim().parse().unwrap())
                    .collect::<Vec<u32>>();
                Ok(Record {
                    blocks: l[0],
                    sessions: l[1],
                    total_session_time: Duration::milliseconds(l[2] as i64),
                })
            }
            Err(e) => Err(e),
        }
    }
    fn write(&self, path: &str) -> std::io::Result<()> {
        fs::write(path, self.to_string())
    }
}
impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n{}\n{}\n",
            self.blocks, self.sessions, self.total_session_time
        )
    }
}

/// There are three ways to slice this tomato.
#[derive(Debug, PartialEq)]
enum TomatePhase {
    Work(DateTime<Utc>),
    ShortBreak(DateTime<Utc>),
    LongBreak(DateTime<Utc>),
}
use TomatePhase::*;

impl Display for TomatePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match &self {
            TomatePhase::Work(_) => "Work",
            TomatePhase::ShortBreak(_) => "Short break",
            TomatePhase::LongBreak(_) => "Long break",
        };

        write!(f, "{}", description)
    }
}

/// Represents the core application state at a given time.
#[derive(Debug)]
struct Tomate {
    config: Config,
    block: u32,
    phase: TomatePhase,
    running: bool,
}
impl Tomate {
    fn new(instant: DateTime<Utc>, config: &Config) -> Tomate {
        Tomate {
            config: config.to_owned(),
            block: 0,
            phase: Work(instant),
            running: config.auto_start && config.auto_start_session,
        }
    }

    /// Look at the clock and figure out the next application state.
    fn next(&self) -> Tomate {
        let (phase, block, running) = self.next_phase();

        Tomate {
            config: self.config.clone(),
            block,
            phase,
            running,
        }
    }

    /// Return a string describing the current state
    fn describe(&self) -> String {
        let time_remaining = self.time_remaining(Utc::now());
        let hours = time_remaining.num_hours();
        let minutes = (time_remaining - Duration::hours(hours)).num_minutes();
        let seconds =
            (time_remaining - Duration::hours(hours) - Duration::minutes(minutes)).num_seconds();
        let paused = if self.running {""} else {" - paused"};
        return format!(
            "Phase: {} - Time remaining: {:0>2}:{:0>2}:{:0>2}{paused}",
            self.phase, hours, minutes, seconds
        );
    }

    /// Return the time left in the current session
    fn time_remaining(&self, now: DateTime<Utc>) -> Duration {
        match self.phase {
            Work(t) => self.config.work_duration - (now - t),
            ShortBreak(t) => self.config.short_break_duration - (now - t),
            LongBreak(t) => self.config.long_break_duration - (now - t),
        }
    }

    /// Given a point in time, returns next state (TomatePhase, block #, running or not)
    fn next_phase(&self) -> (TomatePhase, u32, bool) {
        let now = Utc::now();
        if !self.running {
            // only update phase start time
            return (
                match self.phase {
                    Work(_) => Work(now),
                    ShortBreak(_) => ShortBreak(now),
                    LongBreak(_) => LongBreak(now),
                },
                self.block,
                self.running,
            );
        }
        let phase_completed = self.time_remaining(now) < Duration::zero();
        match self.phase {
            Work(start_time) => {
                if phase_completed {
                    notify("Time to take a break!");
                    if self.block < self.config.blocks_n - 1 {
                        (ShortBreak(now), self.block, self.config.auto_start_break)
                    } else {
                        (LongBreak(now), self.block, self.config.auto_start_break)
                    }
                } else {
                    (Work(start_time), self.block, self.running)
                }
            }
            ShortBreak(start_time) => {
                if phase_completed {
                    (Work(now), self.block + 1, self.config.auto_start_session)
                } else {
                    (ShortBreak(start_time), self.block, self.running)
                }
            }
            LongBreak(start_time) => {
                if phase_completed {
                    (
                        Work(now),
                        0,
                        self.config.auto_start && self.config.auto_start_session,
                    )
                } else {
                    (LongBreak(start_time), self.block, self.running)
                }
            }
        }
    }
}

fn notify(message: &str) {
    match Notification::new()
        .summary(&message)
        .show() {
            Ok(_) => {},
            Err(e) => {
                println!("Error sending notification: {e}");
                process::exit(1);
            }
        }
}
