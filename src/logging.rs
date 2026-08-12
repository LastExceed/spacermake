use std::env::args;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use chrono::{Local, NaiveDate};
use log::kv::VisitSource;
use log::*;

use crate::APP_NAME;

use self::ansi_escape_codes::SelectGraphicRendition::{self,*};

mod ansi_escape_codes;

pub fn rainbow() {
	println!("{ForegroundBlack}Black{Reset}");
	println!("{ForegroundRed}Red{Reset}");
	println!("{ForegroundGreen}Green{Reset}");
	println!("{ForegroundYellow}Yellow{Reset}");
	println!("{ForegroundBlue}Blue{Reset}");
	println!("{ForegroundMagenta}Magenta{Reset}");
	println!("{ForegroundCyan}Cyan{Reset}");
	println!("{ForegroundWhite}White{Reset}");
	println!("{ForegroundBrightBlack}BrightBlack{Reset}");
	println!("{ForegroundBrightRed}BrightRed{Reset}");
	println!("{ForegroundBrightGreen}BrightGreen{Reset}");
	println!("{ForegroundBrightYellow}BrightYellow{Reset}");
	println!("{ForegroundBrightBlue}BrightBlue{Reset}");
	println!("{ForegroundBrightMagenta}BrightMagenta{Reset}");
	println!("{ForegroundBrightCyan}BrightCyan{Reset}");
	println!("{ForegroundBrightWhite}BrightWhite{Reset}");
}

static INSTANCE: Logger = Logger { cache: LazyLock::new(Mutex::default) }; // const default when

struct Cache {
	date: NaiveDate,
	file: File
}

impl Cache {
	fn refresh(&mut self, today: NaiveDate) {
		if self.date == today {
			return;
		}
		
		self.file.flush().expect("flush log file");
		*self = Self::default();
	}
}

impl Default for Cache {
	fn default() -> Self {
		let date = Local::now().date_naive();
		let file = new_file(date);

		Self { date, file }
	}
}

fn new_file(date: NaiveDate) -> File {	
	let date_fmt = date.format("%Y-%m-%d");

	let mut n = 0;
	let mut path;
	
	loop {
		path = dir().join(format!("logs_{date_fmt}_#{n}.tsv"));
		if !path.exists() { break; }
		n += 1;
	}
	
	File
	::options()
	.create_new(true)
	.append(true)
	.open(path)
	.expect("create log file")
}

pub struct Logger {
	cache: LazyLock<Mutex<Cache>>
}

impl Logger {
	pub fn init() {
		let dir = dir();
		if !dir.exists() {
			fs::create_dir_all(dir).expect("create logs dir");
		}
		
		let max_level =
			if args().any(|arg| arg == "--trace") {
				LevelFilter::Trace
			} else {
				LevelFilter::Debug
			};
		
		set_logger(&INSTANCE).unwrap();
		set_max_level(max_level);
	}
	
	fn is_enabled_for(metadata: &Metadata<'_>) -> bool {
		metadata.target().starts_with(APP_NAME)
	}
}

impl Log for Logger {
	fn enabled(&self, metadata: &Metadata<'_>) -> bool {
		Self::is_enabled_for(metadata)
	}

	fn log(&self, record: &Record<'_>) {		
		if !Self::is_enabled_for(record.metadata()) {
			return;
		}

		let now = Local::now();

		let mut inner = self.cache.lock().expect("logger mutex posioned");
		inner.refresh(now.date_naive());
		
		let level = record.level();
		let args = record.args();
		let time_fmt = now.format("%H:%M'%S%.3f");
		let [color1, color2] = get_colors(level);
		
		println!("{time_fmt} {color2}{level}: {color1}{args}{Reset}");
		record.key_values().visit(&mut Visitor).unwrap();
		writeln!(&mut inner.file, "{time_fmt}\t{level}\t{args}").expect("write log file");
	}

	fn flush(&self) {
		// nothing to do here
	}
}

struct Visitor;

impl VisitSource<'_> for Visitor {
    fn visit_pair<'kvs>(&mut self, key: log::kv::Key<'kvs>, value: log::kv::Value<'kvs>) -> Result<(), log::kv::Error> {
        println!("\t{ForegroundMagenta}{key}\t{value}{Reset}");
        Ok(())
    }
}

fn dir() -> PathBuf {
	crate::dir().join("logs")
}

const fn get_colors(level: Level) -> [SelectGraphicRendition; 2] {
	match level {
		Level::Error => [ForegroundRed   , ForegroundBrightRed   ],
		Level::Warn  => [ForegroundYellow, ForegroundBrightYellow],
		Level::Info  => [ForegroundGreen , ForegroundBrightGreen ],
		Level::Debug => [ForegroundBlue  , ForegroundBrightBlue  ],
		Level::Trace => [ForegroundCyan  , ForegroundBrightCyan  ]
	}
}