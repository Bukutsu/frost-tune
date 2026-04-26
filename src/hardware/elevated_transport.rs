use crate::hardware::helper_ipc::{HelperRequest, HelperResponse};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct ElevatedTransport {
    child: Child,
    child_stdin: ChildStdin,
    child_stdout: BufReader<ChildStdout>,
}

impl ElevatedTransport {
    pub fn spawn() -> Result<Self, String> {
        if let Some(installed_helper) = discover_installed_helper_path() {
            return spawn_via_pkexec(CommandSpec {
                program: installed_helper,
                args: vec![],
            });
        }

        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Failed to resolve current executable path: {}", e))?;

        spawn_via_pkexec(CommandSpec {
            program: current_exe,
            args: vec!["--hid-helper".to_string()],
        })
    }

    pub fn round_trip(&mut self, request: &HelperRequest) -> Result<HelperResponse, String> {
        let payload = serde_json::to_string(request)
            .map_err(|e| format!("Failed to serialize helper request: {}", e))?;

        self.child_stdin
            .write_all(payload.as_bytes())
            .map_err(|e| format!("Failed to write request to helper: {}", e))?;
        self.child_stdin
            .write_all(b"\n")
            .map_err(|e| format!("Failed to write request delimiter to helper: {}", e))?;
        self.child_stdin
            .flush()
            .map_err(|e| format!("Failed to flush helper stdin: {}", e))?;

        let mut line = String::new();
        let bytes = self
            .child_stdout
            .read_line(&mut line)
            .map_err(|e| format!("Failed to read helper response: {}", e))?;

        if bytes == 0 {
            if let Some(status) = self
                .child
                .try_wait()
                .map_err(|e| format!("Failed checking helper status: {}", e))?
            {
                return Err(format!("Helper exited unexpectedly: {}", status));
            }
            return Err("Helper closed stdout unexpectedly".to_string());
        }

        serde_json::from_str::<HelperResponse>(line.trim())
            .map_err(|e| format!("Failed to parse helper response: {}", e))
    }

    pub fn shutdown(&mut self) {
        let _ = self.round_trip(&HelperRequest::Shutdown);
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for ElevatedTransport {
    fn drop(&mut self) {
        self.shutdown();
    }
}

struct CommandSpec {
    program: PathBuf,
    args: Vec<String>,
}

fn spawn_via_pkexec(spec: CommandSpec) -> Result<ElevatedTransport, String> {
    let mut command = Command::new("pkexec");
    command.arg(spec.program.as_os_str());
    for arg in spec.args {
        command.arg(arg);
    }

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "pkexec not found. Install polkit (policykit-1).".to_string()
            } else {
                format!("Failed to launch helper via pkexec: {}", e)
            }
        })?;

    let child_stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Failed to open helper stdin".to_string())?;
    let child_stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to open helper stdout".to_string())?;

    Ok(ElevatedTransport {
        child,
        child_stdin,
        child_stdout: BufReader::new(child_stdout),
    })
}

fn discover_installed_helper_path() -> Option<PathBuf> {
    let helper_path = PathBuf::from("/usr/libexec/frost-tune/frost-tune-hid-helper");
    if Path::new(&helper_path).exists() {
        Some(helper_path)
    } else {
        None
    }
}
