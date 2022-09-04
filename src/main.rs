#![warn(missing_docs)]
use chrono::prelude::*;
use chrono::Duration;
use clap::Parser;


#[derive(Parser)]
struct Args {
    // todo
}

/// User options
#[derive(Debug)]
struct Config {
    session_dur: Duration,
    s_break_dur: Duration,
    l_break_dur: Duration,
    num_blocks: i32,
    auto_start: bool,
    auto_start_session: bool,
    auto_start_break: bool,
}

/// Sane defaults
impl Default for Config {
    fn default() -> Self {
        Config {
            session_dur: Duration::minutes(25),
            s_break_dur: Duration::minutes(5),
            l_break_dur: Duration::minutes(15),
            num_blocks: 4,
            auto_start: true,
            auto_start_session: false,
            auto_start_break: true,
        }
    }
}

/// There are three ways to slice this tomato.
enum Slice {
    Session(DateTime<Utc>),
    ShortBreak(DateTime<Utc>),
    LongBreak(DateTime<Utc>),
}
use Slice::*;

/// Represents the core application state at a given time.
/// `progress` is always relative to the beginning of a cycle.
#[derive(Debug)]
struct Tomate {
    config: Config,
    progress: Duration,
    instant: DateTime<Utc>,
}

impl Tomate {
    fn new(config: Option<Config>) -> Tomate {
        let _config = config.unwrap_or(Config::default());
        Tomate {
            config: _config,
            progress: Duration::zero(),
            instant: Utc::now(),
        }
    }
    
    /// Find the start of the next session or break.
    fn next_slice(&self) -> Option<Slice> {
        None
    }

    /// Look at the clock and figure out the next application state.
    fn next(&self) -> Tomate {
        let instant = Utc::now();

        match self.next_slice() {
            Session(t) => {},
            ShortBreak(t) => {},
            LongBreak(t) => {},
        }

        Tomate {
            ..*self,
            instant,
            progress,
            next_slice,
        }
    }
}

#[derive(Debug)]
struct Stats {
    blocks: i32,
    sessions: i32,
    total_session_time: Duration,
}

fn main() {
    let args = Args::parse();
    let cfg = Config::default();
    let mut tomate = Tomate::new(Some(cfg));

    loop {
        let now = Utc::now();

        tomate = tomate.next();

        std::thread::sleep(std::Duration::seconds(1));
    }
}

