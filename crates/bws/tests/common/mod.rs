use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

/// Fake access token accepted by `crates/fake-server`.
pub const TEST_TOKEN: &str = "0.ec2c1d46-6a4b-4751-a310-af9601317f2d.C2IgxjjLF7qSshsbwe8JGcbM075YXw:X8vbvA0bduihIDe/qrzIQQ==";

/// Runs the `bws` binary in an isolated environment and returns `(exit code, stdout, stderr)`.
pub fn bws(args: &[&str], env: &[(&str, &str)]) -> (i32, String, String) {
    let home = tempfile::tempdir().expect("temp home dir to be created");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bws"));
    for (key, _) in std::env::vars_os() {
        let key = key.to_string_lossy();
        if key.starts_with("BWS_") || key == "RUST_BACKTRACE" || key == "SHELL" {
            cmd.env_remove(key.as_ref());
        }
    }
    let output = cmd
        .args(args)
        .env("HOME", home.path())
        .env("NO_COLOR", "1")
        .envs(env.iter().copied())
        .stdin(Stdio::null())
        .output()
        .expect("bws binary to run");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Asserts that stderr is exactly `lines`, and that `lines` itself leaks no internals — so a
/// future author cannot satisfy the equality above by pasting leaky output into the expectation.
pub fn assert_error(stderr: &str, lines: &[&str]) {
    let actual: Vec<&str> = stderr.lines().collect();
    assert_eq!(actual, lines, "unexpected stderr:\n{stderr}");

    // The secret half of the token; it implies the whole token as a substring.
    let (_, token_secret) = TEST_TOKEN
        .split_once(':')
        .expect("TEST_TOKEN to contain ':'");
    for forbidden in ["Location:", "os error", "{\"", "<html", token_secret] {
        assert!(
            !lines.iter().any(|l| l.contains(forbidden)),
            "expected output contains {forbidden:?}:\n{stderr}"
        );
    }
}

pub struct Route {
    pub method: &'static str,
    pub path_prefix: &'static str,
    pub status: u16,
    pub content_type: &'static str,
    pub body: &'static str,
}

/// Minimal HTTP/1.1 server that answers each request with the first matching canned route.
pub struct MockServer {
    port: u16,
    stop: Arc<AtomicBool>,
}

impl MockServer {
    pub fn start(routes: Vec<Route>) -> MockServer {
        let listener = TcpListener::bind("127.0.0.1:0").expect("mock server to bind");
        let port = listener.local_addr().expect("mock server address").port();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();

        thread::spawn(move || {
            for stream in listener.incoming() {
                if stop_flag.load(Ordering::SeqCst) {
                    break;
                }
                if let Ok(stream) = stream {
                    handle(stream, &routes);
                }
            }
        });

        MockServer { port, stop }
    }

    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // Wakes the accept loop so the thread sees the stop flag.
        let _ = TcpStream::connect(("127.0.0.1", self.port));
    }
}

fn handle(mut stream: TcpStream, routes: &[Route]) {
    let Ok(read_half) = stream.try_clone() else {
        return;
    };
    let mut reader = BufReader::new(read_half);

    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();

    let mut content_length = 0;
    for line in reader.by_ref().lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0; content_length];
    let _ = reader.read_exact(&mut body);

    let response = match routes
        .iter()
        .find(|r| r.method.eq_ignore_ascii_case(&method) && path.starts_with(r.path_prefix))
    {
        Some(route) => format!(
            "HTTP/1.1 {} Mock\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            route.status,
            route.content_type,
            route.body.len(),
            route.body
        ),
        None => {
            "HTTP/1.1 599 No Route\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string()
        }
    };
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}
