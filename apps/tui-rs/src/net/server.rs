use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::config::SERVER_PORT;

/// Wrapper around the child server process.
///
/// On drop the process group is sent SIGTERM, then SIGKILL after 5 s.
pub struct ServerProcess {
    child: Option<Child>,
}

impl ServerProcess {
    /// Launch the Bun dev server inside `server_dir`.
    ///
    /// Tries `mise exec -- bun run dev` first (respects pinned toolchain),
    /// falls back to bare `bun run dev`.
    pub fn start(server_dir: &Path) -> Self {
        if !server_dir.is_dir() {
            return Self { child: None };
        }

        let mise = which("mise");
        let (program, args): (&str, Vec<&str>) = if mise.is_some() {
            ("mise", vec!["exec", "--", "bun", "run", "dev"])
        } else if which("bun").is_some() {
            ("bun", vec!["run", "dev"])
        } else {
            return Self { child: None };
        };

        let child = unsafe {
            Command::new(program)
                .args(&args)
                .current_dir(server_dir)
                .env("SERVER_PORT", SERVER_PORT)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .pre_exec(|| {
                    // Put the child in its own process group so we can kill the
                    // whole tree on shutdown.
                    libc::setpgid(0, 0);
                    Ok(())
                })
                .spawn()
                .ok()
        };

        Self { child }
    }

    /// Gracefully terminate the server (SIGTERM → wait 5 s → SIGKILL).
    pub fn shutdown(&mut self) {
        let child = match self.child.take() {
            Some(c) => c,
            None => return,
        };
        let pid = child.id() as i32;

        // SIGTERM the whole process group
        unsafe { libc::kill(-pid, libc::SIGTERM) };

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut child = child;
        loop {
            match child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(100));
                }
                _ => break,
            }
        }

        // Force-kill if still alive
        unsafe { libc::kill(-pid, libc::SIGKILL) };
        let _ = child.wait();
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn which(name: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let full = dir.join(name);
            if full.is_file() {
                Some(full)
            } else {
                None
            }
        })
    })
}
