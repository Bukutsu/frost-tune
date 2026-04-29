use crate::error::{AppError, ErrorKind, Result};
use crate::hardware::helper_ipc::{HelperRequest, HelperResponse};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct ElevatedTransport {
    child: Child,
    child_stdin: ChildStdin,
    child_stdout: Option<BufReader<ChildStdout>>,
}

impl ElevatedTransport {
    pub fn spawn() -> Result<Self> {
        let mut transport = if let Some(installed_helper) = discover_installed_helper_path() {
            spawn_via_pkexec(CommandSpec {
                program: installed_helper,
                args: vec![],
            })?
        } else {
            let current_exe = std::env::current_exe()
                .map_err(|e| AppError::general(format!("Failed to resolve current executable path: {}", e)))?;

            spawn_via_pkexec(CommandSpec {
                program: current_exe,
                args: vec!["--hid-helper".to_string()],
            })?
        };

        // Version check
        use crate::hardware::helper_ipc::IPC_VERSION;
        match transport.round_trip(&HelperRequest::Version)? {
            HelperResponse::Version { version } => {
                if version != IPC_VERSION {
                    return Err(AppError::new(
                        ErrorKind::HardwareError,
                        format!("IPC Version mismatch: UI={} helper={}. Re-install the application.", IPC_VERSION, version),
                    ));
                }
            }
            _ => {
                return Err(AppError::new(
                    ErrorKind::HardwareError,
                    "Elevated helper failed version handshake",
                ));
            }
        }

        Ok(transport)
    }

    pub fn round_trip(&mut self, request: &HelperRequest) -> Result<HelperResponse> {
        let payload = serde_json::to_string(request)
            .map_err(|e| AppError::new(ErrorKind::ParseError, format!("Failed to serialize helper request: {}", e)))?;

        self.child_stdin
            .write_all(payload.as_bytes())
            .map_err(|e| AppError::new(ErrorKind::IpcError, format!("Failed to write request to helper: {}", e)))?;
        self.child_stdin
            .write_all(b"\n")
            .map_err(|e| AppError::new(ErrorKind::IpcError, format!("Failed to write request delimiter to helper: {}", e)))?;
        self.child_stdin
            .flush()
            .map_err(|e| AppError::new(ErrorKind::IpcError, format!("Failed to flush helper stdin: {}", e)))?;

        let mut stdout = self.child_stdout.take().ok_or_else(|| {
            AppError::new(ErrorKind::DeviceLost, "Helper stdout is missing (previous timeout?)")
        })?;

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let res = stdout.read_line(&mut line);
            let _ = tx.send((res, line, stdout));
        });

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        let (bytes, line) = loop {
            if let Some(status) = self
                .child
                .try_wait()
                .map_err(|e| AppError::general(format!("Failed checking helper status: {}", e)))?
            {
                return Err(AppError::new(ErrorKind::DeviceLost, format!("Helper exited unexpectedly: {}", status)));
            }

            if std::time::Instant::now() > deadline {
                return Err(AppError::new(ErrorKind::ReadTimeout, "Elevated helper response timed out (15s)"));
            }

            match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok((res, line, returned_stdout)) => {
                    self.child_stdout = Some(returned_stdout);
                    let bytes = res.map_err(|e| AppError::new(ErrorKind::ReadTimeout, format!("Failed to read helper response: {}", e)))?;
                    break (bytes, line);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(AppError::new(ErrorKind::DeviceLost, "Helper read thread disconnected"));
                }
            }
        };

        if bytes == 0 {
            return Err(AppError::new(ErrorKind::DeviceLost, "Helper closed stdout unexpectedly"));
        }

        serde_json::from_str::<HelperResponse>(line.trim())
            .map_err(|e| AppError::new(ErrorKind::ParseError, format!("Failed to parse helper response: {}", e)))
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

fn spawn_via_pkexec(spec: CommandSpec) -> Result<ElevatedTransport> {
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
                AppError::new(ErrorKind::PolkitAuthRequired, "pkexec not found. Install polkit (policykit-1).")
            } else {
                AppError::general(format!("Failed to launch helper via pkexec: {}", e))
            }
        })?;

    let child_stdin = child
        .stdin
        .take()
        .ok_or_else(|| AppError::general("Failed to open helper stdin"))?;
    let child_stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::general("Failed to open helper stdout"))?;

    Ok(ElevatedTransport {
        child,
        child_stdin,
        child_stdout: Some(BufReader::new(child_stdout)),
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
