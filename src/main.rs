use std::{env::args, fs::{read_to_string, File}, io::{stdout, ErrorKind, Read, Write}, process::{exit, Command, Stdio}, time::{SystemTime, UNIX_EPOCH}};

const CONFIG_PATH: &str = "./exe-proxy-target.txt";
const LOG_PATH_PREFIX: &str = "./exe-proxy";
fn main() -> ! {
    let program_path = match read_to_string(CONFIG_PATH) {
        Ok(it) => it.trim().to_string(),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            if let Err(err) = File::create(CONFIG_PATH) {
                eprintln!("you need to create a file at {CONFIG_PATH}. couldn't create it: {err}");
            } else {
                eprintln!("you need to put a path to the program you want to proxy to {CONFIG_PATH}");
            }
            exit(69002);
        }
        Err(err) => {
            eprintln!("couldn't open the config file: {err}");
            exit(69003);
        },
    };
    let time_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(it) => it.as_millis(),
        Err(_) => 0,
    };
    let log_path = format!("{LOG_PATH_PREFIX}.{time_ms}.log");
    let mut log = match File::create(&log_path) {
        Ok(it) => it,
        Err(err) => {
            eprintln!("couldn't create a log file at {log_path}: {err}");
            exit(69003)
        },
    };
    if let Err(err) = writeln!(log, "{}:", args().collect::<Vec<String>>().join(" ")) {
        eprintln!("couldn't write to log {log_path}: {err}");
        exit(69003);
    }

    if program_path.contains('\n') {
        eprintln!("{CONFIG_PATH} contains more than one line");
        exit(69006);
    }
    if program_path.is_empty() {
        eprintln!("{CONFIG_PATH} is empty. add a target program path");
        exit(69007);
    }

    // let stdout_buff = BufWriter::new(todo!());
    let mut child = match Command::new(&program_path)
    .args(args().skip(1))
    .stdout(Stdio::piped())
    .spawn() {
        Ok(it) => it,
        Err(err) => {
            eprintln!("couldn't spawn {program_path}: {err}");
            exit(69004);
        }
    };

    let mut child_stdout = child.stdout.take()
        .expect("should be there since Stdio::piped() is used");
    let mut stdout_writer = LogWriter {
        file: log,
        stdio: stdout(),
    };
    let mut buf = [0u8; 4];
    while let Ok(n) = child_stdout.read(&mut buf) {
        if n <= 0 {
            break;
        }
        if let Err(err) = stdout_writer.write(&buf[..n]) {
            eprintln!("cannot write stdout: {err}");
            exit(69008)
        }
    }

    // TODO: threaded, stdin, stderr

    match child.wait() {
        Ok(status) => {
            if let Some(code) = status.code() {
                exit(code);
            } else {
                exit(69005);
            }
        }
        Err(_) => todo!(),
    }
}

struct LogWriter<W: Write> {
    pub file: File,
    pub stdio: W,
}

impl<W: Write> Write for LogWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)?;
        self.stdio.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()?;
        self.stdio.flush()
    }
}