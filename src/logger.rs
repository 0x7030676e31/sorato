use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::{env, fmt, fs};
use std::io::Write;

struct Padded<T> {
  value: T,
  width: usize,
}

impl<T: fmt::Display> fmt::Display for Padded<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{: <width$}", self.value, width = self.width)
  }
}

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);
static LINE_COUNT: AtomicUsize = AtomicUsize::new(0);
const MAX_LINES: usize = 1024;
const FILE: &str = "logs.txt";

fn max_target_width(target: &str) -> usize {
  let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
  if max_width < target.len() {
    MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
    target.len()
  } else {
    max_width
  }
}

pub fn init() {
  let mut builder = pretty_env_logger::formatted_builder();
  let lock = Mutex::new(());

  if let Ok(count) = fs::read_to_string(FILE).map(|s| s.lines().count()) {
    LINE_COUNT.store(count + 1, Ordering::Relaxed);
  }

  let file = fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(FILE);

  if let Err(e) = file {
    eprintln!("Failed to open file: {}", e);
    return;
  }

  let date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
  let _ = writeln!(file.unwrap(), "======================================[ {} ]======================================", date);

  let _ = builder
    .parse_filters(&env::var("RUST_LOG").unwrap())
    .format(move |f, record| {
      let target = record.target();
      let max_width = max_target_width(target);

      let style = f.default_level_style(record.level());
      let level = style.value(Padded {
        value: record.level(),
        width: 5,
      });

      let mut style = f.style();
      let target = style.set_bold(true).value(Padded {
        value: target,
        width: max_width,
      });

      let res = writeln!(f, " {} {} > {}", level, target, record.args());
      
      let _lock = lock.lock().unwrap();
      let file = fs::OpenOptions::new()
        .create(true)
        // .write(true)
        // .read(true)
        .append(true)
        .open(FILE);

      let mut file = match file {
        Ok(file) => file,
        Err(e) => {
          eprintln!("Failed to open file: {}", e);
          return res;
        }
      };

      let date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
      writeln!(file, "[{} {}] {} > {}", level, date, target, record.args())
        .expect("Failed to write to file");

      let count = LINE_COUNT.fetch_add(1, Ordering::Relaxed);
      if count as f32 >= MAX_LINES as f32 * 1.5 {
        let lines = fs::read_to_string(FILE).expect("Failed to read file");
        let lines = lines.lines().skip(count - MAX_LINES + 1).chain(Some("")).collect::<Vec<_>>();
        fs::write(FILE, lines.join("\n")).expect("Failed to write to file");
        LINE_COUNT.store(MAX_LINES, Ordering::Relaxed);
      }
      
      res

      // if let Ok(mut file) = file {

      //   let date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
      //   writeln!(file, "[{} {}] {} > {}", level, date, target, record.args())
      //     .expect("Failed to write to file");

      //   let count = LINE_COUNT.fetch_add(1, Ordering::Relaxed) as f32;
      //   if count >= MAX_LINES as f32 * 1.5 {
      //     let lines = fs::read_to_string("output.txt").expect("Failed to read file");
      //     let lines = lines.lines().skip(count as usize - MAX_LINES).collect::<Vec<_>>();
      //     fs::write("output.txt", lines.join("\n")).expect("Failed to write to file");
      //     LINE_COUNT.store(MAX_LINES, Ordering::Relaxed);
      //   }
      // }

      // drop(lock);
    })
    .try_init();
}