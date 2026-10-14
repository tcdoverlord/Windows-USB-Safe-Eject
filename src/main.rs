#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use serde::Deserialize;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone, Debug, Deserialize)]
struct DriveInfo {
    #[serde(rename = "Drive")]
    drive: String,
    #[serde(rename = "Label")]
    label: String,
    #[serde(rename = "SizeGB")]
    size_gb: f64,
    #[serde(rename = "FreeGB")]
    free_gb: f64,
    #[serde(rename = "FileSystem")]
    file_system: String,
}

enum AppEvent {
    Drives(Result<Vec<DriveInfo>, String>),
    EngineLog(String),
    EnginePrompt {
        text: String,
        reply: Sender<String>,
    },
    EngineFinished {
        code: Option<i32>,
        drive: String,
        physical_id: Option<String>,
    },
    PhysicalRemovalDetected,
}

#[derive(Clone, Copy, PartialEq)]
enum PromptKind {
    ConfirmEject,
    ForceClose,
}

struct PendingPrompt {
    text: String,
    reply: Sender<String>,
    kind: PromptKind,
}

struct SafeUsbApp {
    tx: Sender<AppEvent>,
    rx: Receiver<AppEvent>,
    drives: Vec<DriveInfo>,
    selected: Option<String>,
    status: String,
    status_ready: bool,
    logs: Vec<String>,
    scanning: bool,
    ejecting: bool,
    confirm_drive: Option<String>,
    prompt: Option<PendingPrompt>,
    next_scan: Instant,
    eject_started: Option<Instant>,
    close_warning: bool,
    close_when_finished: bool,
    completed_once: bool,
    exit_after_success_at: Option<Instant>,
    awaiting_physical_removal: bool,
}

impl SafeUsbApp {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let mut app = Self {
            tx,
            rx,
            drives: Vec::new(),
            selected: None,
            status: "Starting drive scan…".into(),
            status_ready: false,
            logs: vec![
                "Safe USB Eject · TCDOVERLORD".into(),
                "The original PowerShell safety engine is retained for its lock checks.".into(),
            ],
            scanning: false,
            ejecting: false,
            confirm_drive: None,
            prompt: None,
            next_scan: Instant::now(),
            eject_started: None,
            close_warning: false,
            close_when_finished: false,
            completed_once: false,
            exit_after_success_at: None,
            awaiting_physical_removal: false,
        };
        app.start_scan();
        app
    }

    fn start_scan(&mut self) {
        if self.scanning || self.ejecting {
            return;
        }
        self.scanning = true;
        self.status = "Checking removable drives in the background…".into();
        let tx = self.tx.clone();
        thread::spawn(move || {
            let script = r#"
$ErrorActionPreference = 'Stop'
$rows = @(
  Get-CimInstance Win32_LogicalDisk -Filter "DriveType = 2" -ErrorAction Stop |
  Where-Object { $_.DeviceID -and $null -ne $_.Size -and [double]$_.Size -gt 0 -and $null -ne $_.FreeSpace } |
  Sort-Object DeviceID |
  ForEach-Object {
    [pscustomobject]@{
      Drive = [string]$_.DeviceID
      Label = if ([string]::IsNullOrWhiteSpace([string]$_.VolumeName)) { '(No label)' } else { [string]$_.VolumeName }
      SizeGB = [math]::Round(([double]$_.Size / 1GB), 2)
      FreeGB = [math]::Round(([double]$_.FreeSpace / 1GB), 2)
      FileSystem = if ([string]::IsNullOrWhiteSpace([string]$_.FileSystem)) { '(Not reported)' } else { [string]$_.FileSystem }
    }
  }
)
ConvertTo-Json -InputObject $rows -Compress
"#;
            let mut cmd = Command::new("powershell.exe");
            cmd.args(["-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            #[cfg(windows)]
            cmd.creation_flags(CREATE_NO_WINDOW);

            let result = cmd.output().map_err(|e| format!("Could not query drives: {e}"))
                .and_then(|out| {
                    if !out.status.success() {
                        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
                        return Err(if err.is_empty() { "Windows drive scan failed.".into() } else { err });
                    }
                    let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if raw.is_empty() || raw == "null" {
                        return Ok(Vec::new());
                    }
                    let value: serde_json::Value = serde_json::from_str(&raw)
                        .map_err(|e| format!("Could not parse drive scan: {e}"))?;
                    let rows = if value.is_array() {
                        serde_json::from_value::<Vec<DriveInfo>>(value)
                            .map_err(|e| format!("Invalid drive data: {e}"))?
                    } else {
                        vec![serde_json::from_value::<DriveInfo>(value)
                            .map_err(|e| format!("Invalid drive data: {e}"))?]
                    };
                    Ok(rows)
                });
            let _ = tx.send(AppEvent::Drives(result));
        });
    }

    fn start_eject(&mut self, drive: String) {
        if self.ejecting || self.completed_once { return; }
        self.ejecting = true;
        self.eject_started = Some(Instant::now());
        self.close_warning = false;
        self.close_when_finished = false;
        self.status_ready = false;
        self.status = format!("Running safety checks for {drive}…");
        self.logs.push(format!("Starting safety engine for {drive}."));
        let tx = self.tx.clone();

        thread::spawn(move || {
            // Capture the physical USB device identity before Windows removes its drive letter.
            let drive_for_query = drive.clone();
            let identity_script = format!(r#"$d = '{}'; try {{
  $disk = Get-CimInstance Win32_LogicalDisk -Filter "DeviceID='$d'" -ErrorAction Stop |
    Get-CimAssociatedInstance -Association Win32_LogicalDiskToPartition -ErrorAction Stop |
    Get-CimAssociatedInstance -Association Win32_DiskDriveToDiskPartition -ErrorAction Stop |
    Select-Object -First 1
  if ($disk -and $disk.PNPDeviceID) {{ [Console]::Out.Write([string]$disk.PNPDeviceID) }}
}} catch {{ }}"#, drive_for_query);
            let mut identity_cmd = Command::new("powershell.exe");
            identity_cmd.args(["-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &identity_script])
                .stdout(Stdio::piped()).stderr(Stdio::null());
            #[cfg(windows)]
            identity_cmd.creation_flags(CREATE_NO_WINDOW);
            let physical_id = identity_cmd.output().ok().filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .filter(|s| !s.is_empty());

            let engine = "scripts\\Safe-USB-Eject.ps1";
            let mut cmd = Command::new("powershell.exe");
            cmd.args([
                "-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
                "-File", engine, "-DriveLetter", &drive, "-GuiMode", "-ConfirmedByGui",
            ])
            .current_dir(std::env::current_exe().ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default()))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
            #[cfg(windows)]
            cmd.creation_flags(CREATE_NO_WINDOW);

            let spawn = cmd.spawn();
            let mut child = match spawn {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(AppEvent::EngineLog(format!("Could not start safety engine: {e}")));
                    let _ = tx.send(AppEvent::EngineFinished { code: Some(1), drive: drive.clone(), physical_id: physical_id.clone() });
                    return;
                }
            };

            let Some(mut stdin) = child.stdin.take() else {
                let _ = tx.send(AppEvent::EngineLog("Could not open safety-engine input channel.".into()));
                let _ = tx.send(AppEvent::EngineFinished { code: Some(1), drive: drive.clone(), physical_id: physical_id.clone() });
                return;
            };
            let Some(stdout) = child.stdout.take() else {
                let _ = tx.send(AppEvent::EngineLog("Could not open safety-engine output channel.".into()));
                let _ = tx.send(AppEvent::EngineFinished { code: Some(1), drive: drive.clone(), physical_id: physical_id.clone() });
                return;
            };
            // Keep stderr drained independently so the child cannot block if it emits diagnostics.
            if let Some(stderr) = child.stderr.take() {
                let tx_err = tx.clone();
                thread::spawn(move || {
                    for line in BufReader::new(stderr).lines().flatten() {
                        let line = line.trim().to_string();
                        if !line.is_empty() {
                            let _ = tx_err.send(AppEvent::EngineLog(format!("Engine: {line}")));
                        }
                    }
                });
            }

            for line in BufReader::new(stdout).lines().flatten() {
                if let Some(msg) = line.strip_prefix("__SAFEUSB_LOG__") {
                    let _ = tx.send(AppEvent::EngineLog(msg.to_string()));
                } else if let Some(prompt) = line.strip_prefix("__SAFEUSB_PROMPT__") {
                    let (reply_tx, reply_rx) = mpsc::channel();
                    if tx.send(AppEvent::EnginePrompt { text: prompt.to_string(), reply: reply_tx }).is_err() {
                        let _ = stdin.write_all(b"\n");
                        let _ = stdin.flush();
                        continue;
                    }
                    let response = reply_rx.recv_timeout(Duration::from_secs(180))
                        .unwrap_or_else(|_| String::new());
                    let _ = writeln!(stdin, "{response}");
                    let _ = stdin.flush();
                } else {
                    let line = line.trim().to_string();
                    if !line.is_empty() {
                        let _ = tx.send(AppEvent::EngineLog(line));
                    }
                }
            }
            let code = child.wait().ok().and_then(|s| s.code());
            if code == Some(0) {
                if let Some(id) = physical_id.clone() {
                    let tx_removed = tx.clone();
                    thread::spawn(move || loop {
                        thread::sleep(Duration::from_secs(2));
                        let mut check = Command::new("powershell.exe");
                        check.args(["-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command",
                            "$id = [Environment]::GetEnvironmentVariable('SAFEUSB_PNP_ID'); if (Get-CimInstance Win32_DiskDrive -ErrorAction SilentlyContinue | Where-Object { $_.PNPDeviceID -eq $id }) { exit 0 } else { exit 1 }"])
                            .env("SAFEUSB_PNP_ID", &id).stdout(Stdio::null()).stderr(Stdio::null());
                        #[cfg(windows)]
                        check.creation_flags(CREATE_NO_WINDOW);
                        match check.status() {
                            Ok(status) if !status.success() => { let _ = tx_removed.send(AppEvent::PhysicalRemovalDetected); break; }
                            Ok(_) => continue,
                            Err(_) => continue,
                        }
                    });
                }
            }
            let _ = tx.send(AppEvent::EngineFinished { code, drive, physical_id });
        });
    }

    fn push_log(&mut self, line: String) {
        self.logs.push(line);
        if self.logs.len() > 250 {
            let excess = self.logs.len() - 250;
            self.logs.drain(0..excess);
        }
    }

    fn process_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                AppEvent::Drives(result) => {
                    self.scanning = false;
                    match result {
                        Ok(rows) => {
                            let old = self.selected.clone();
                            self.drives = rows;
                            self.selected = old.filter(|d| self.drives.iter().any(|r| &r.drive == d))
                                .or_else(|| self.drives.first().map(|d| d.drive.clone()));
                            self.status = if self.drives.is_empty() {
                                "No removable drives found. Connect a USB drive; this list refreshes automatically.".into()
                            } else {
                                format!("{} removable drive(s) found · auto-refresh every 3 seconds", self.drives.len())
                            };
                        }
                        Err(e) => {
                            self.status = format!("Drive scan failed; the window remains responsive. {e}");
                            self.push_log(e);
                        }
                    }
                    self.next_scan = Instant::now() + Duration::from_secs(3);
                }
                AppEvent::EngineLog(line) => self.push_log(line),
                AppEvent::EnginePrompt { text, reply } => {
                    let kind = if text.to_ascii_uppercase().contains("FORCE") {
                        PromptKind::ForceClose
                    } else {
                        PromptKind::ConfirmEject
                    };
                    self.prompt = Some(PendingPrompt { text, reply, kind });
                }
                AppEvent::PhysicalRemovalDetected => {
                    self.awaiting_physical_removal = false;
                    self.status_ready = true;
                    self.status = "EJECT COMPLETED — SUCCESS. Closing app…".into();
                    self.push_log("Physical USB removal detected. Closing application.".into());
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                AppEvent::EngineFinished { code, drive, physical_id } => {
                    self.ejecting = false;
                    self.eject_started = None;
                    self.prompt = None;
                    match code {
                        Some(0) => {
                            // One successful eject is enough for this build. Show a clear
                            // completion message briefly, then exit instead of inviting a
                            // second operation that the user does not want to test yet.
                            self.completed_once = true;
                            self.awaiting_physical_removal = true;
                            self.exit_after_success_at = None;
                            self.status_ready = true;
                            self.status = format!("SCRIPT RAN SUCCESSFULLY for {drive}. Safe-removal request sent. Green means the script completed successfully.");
                            self.push_log("SCRIPT RAN SUCCESSFULLY. Windows safe-removal request completed.".into());
                            self.push_log("Remove the USB device physically. The app will close when removal is detected.".into());
                            if physical_id.is_none() {
                                self.status_ready = true;
                                self.status = format!("SCRIPT RAN SUCCESSFULLY for {drive}, but physical removal cannot be verified. Only remove it after Windows confirms safe removal.");
                                self.push_log("Could not identify the physical USB device for removal monitoring; automatic close is disabled for safety.".into());
                                self.awaiting_physical_removal = false;
                            }
                        }
                        Some(n) => { self.status_ready = false; self.status = format!("NOT READY — Eject was not verified (engine exit code {n}). Do not unplug unless Windows confirms safe removal."); },
                        None => { self.status_ready = false; self.status = "NOT READY — Safety engine ended without a verifiable exit code. Check logs and Windows notification.".to_string(); },
                    }
                    if self.close_when_finished {
                        // Honor an explicit close request after the helper has exited.
                        self.close_when_finished = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    } else if !self.completed_once {
                        self.next_scan = Instant::now();
                        self.start_scan();
                    }
                }
            }
        }
    }
}

impl eframe::App for SafeUsbApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keep the window responsive during an operation, but never trap the user
        // indefinitely if the helper process stops responding. The explicit close
        // option below warns that the ejection result may remain unverified.
        if (self.ejecting || self.awaiting_physical_removal) && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            if self.ejecting {
                self.close_warning = true;
                self.close_when_finished = true;
                self.status = "Ejection helper is still running. Wait for a result, or use the explicit close option if it is stuck.".into();
            } else {
                self.status = "Script ran successfully. Physically remove the USB drive before closing this app.".into();
                self.push_log("Close requested while USB is still physically connected; waiting for removal.".into());
            }
        }
        self.process_events(ctx);
        if self.exit_after_success_at.is_some_and(|deadline| Instant::now() >= deadline) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if !self.completed_once && !self.ejecting && !self.scanning && Instant::now() >= self.next_scan {
            self.start_scan();
        }

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("SAFE USB EJECT").size(28.0).strong().color(egui::Color32::from_rgb(78, 166, 255)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("TCDOVERLORD · RUST UI").small().color(egui::Color32::LIGHT_BLUE));
                });
            });
            ui.label("Select a removable drive. Nothing happens until you confirm.");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("↻  Refresh").clicked() && !self.scanning && !self.ejecting {
                    self.start_scan();
                }
                if ui.add_enabled(!self.completed_once && !self.ejecting && !self.scanning && self.selected.is_some(),
                    egui::Button::new("Safely Eject Selected")).clicked() {
                    self.confirm_drive = self.selected.clone();
                }
                if ui.add_enabled(!self.ejecting && !self.awaiting_physical_removal, egui::Button::new("Exit")).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if self.scanning {
                    ui.spinner();
                    ui.label("Scanning in background…");
                }
            });

            ui.add_space(10.0);
            egui::Grid::new("drive_table").striped(true).num_columns(5).show(ui, |ui| {
                ui.strong("Drive");
                ui.strong("Volume label");
                ui.strong("Capacity / free space");
                ui.strong("File system");
                ui.strong("Select");
                ui.end_row();

                for d in &self.drives {
                    ui.label(&d.drive);
                    ui.label(&d.label);
                    ui.label(format!("{:.2} GB total · {:.2} GB free", d.size_gb, d.free_gb));
                    ui.label(&d.file_system);
                    let selected = self.selected.as_deref() == Some(d.drive.as_str());
                    if ui.selectable_label(selected, if selected { "Selected" } else { "Choose" }).clicked() {
                        self.selected = Some(d.drive.clone());
                    }
                    ui.end_row();
                }
            });

            ui.add_space(8.0);
            let (status_title, status_color) = if self.status_ready {
                ("GREEN · SCRIPT SUCCESSFULLY COMPLETED", egui::Color32::from_rgb(40, 190, 90))
            } else {
                ("RED · NOT READY / SUCCESS NOT VERIFIED", egui::Color32::from_rgb(205, 55, 55))
            };
            egui::Frame::group(ui.style())
                .fill(status_color.gamma_multiply(0.16))
                .stroke(egui::Stroke::new(1.5, status_color))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(status_title).strong().color(status_color).size(15.0));
                    });
                    ui.label(egui::RichText::new(&self.status).color(status_color));
                });

            ui.separator();
            ui.heading("Activity");
            egui::ScrollArea::vertical().max_height(180.0).stick_to_bottom(true).show(ui, |ui| {
                for line in self.logs.iter().rev().take(40).rev() {
                    ui.label(egui::RichText::new(line).monospace().size(12.0));
                }
            });
            ui.add_space(4.0);
            ui.label(egui::RichText::new(if self.status_ready { "GREEN means the safety script returned success; check Windows safe-removal notification before unplugging. The app may close automatically after device removal is detected." } else { "RED means not ready or success has not been verified. Save files and close anything using the drive. Windows remains the final authority on safe removal." }).small().color(if self.status_ready { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(255, 100, 100) }));
        });

        if self.ejecting && self.close_warning {
            egui::Window::new("Ejection still running")
                .collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.colored_label(egui::Color32::YELLOW, "The safety helper has not reported completion.");
                    ui.label("Closing now can leave the ejection result unverified. Do not unplug the device unless Windows confirms safe removal.");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Keep Waiting").clicked() { self.close_warning = false; self.close_when_finished = false; }
                        if ui.button("Close UI Anyway…").clicked() {
                            // If the engine is awaiting a prompt, answer with an empty
                            // response so it takes its cancellation path before the UI exits.
                            self.close_when_finished = false;
                            if let Some(p) = self.prompt.take() { let _ = p.reply.send(String::new()); }
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
        }

        if let Some(drive) = self.confirm_drive.clone() {
            egui::Window::new("Confirm safe ejection")
                .collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(format!("You are about to request safe removal of {drive}."));
                    ui.label("Save all files and close anything using this drive first.");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.confirm_drive = None;
                        }
                        if ui.button("Confirm Eject").clicked() {
                            let drive = drive.clone();
                            self.confirm_drive = None;
                            self.start_eject(drive);
                        }
                    });
                });
        }

        if self.prompt.is_some() {
            let (kind, text) = {
                let p = self.prompt.as_ref().unwrap();
                (p.kind, p.text.clone())
            };
            egui::Window::new(if kind == PromptKind::ForceClose { "Applications still using the drive" } else { "Safety engine confirmation" })
                .collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    if kind == PromptKind::ForceClose {
                        ui.colored_label(egui::Color32::YELLOW, "Warning: force-closing applications can cause loss of unsaved work.");
                        ui.label("Graceful close was already requested and locks were checked again.");
                        ui.label("Only non-critical processes may be force-closed. Windows critical processes remain protected.");
                    } else {
                        ui.label(&text);
                    }
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(if kind == PromptKind::ForceClose { "Cancel Eject" } else { "Cancel" }).clicked() {
                            if let Some(p) = self.prompt.take() { let _ = p.reply.send(String::new()); }
                        }
                        if kind == PromptKind::ForceClose && ui.button("Force Close Remaining Apps").clicked() {
                            if let Some(p) = self.prompt.take() { let _ = p.reply.send("FORCE".into()); }
                        }
                        if kind == PromptKind::ConfirmEject && ui.button("Confirm EJECT").clicked() {
                            if let Some(p) = self.prompt.take() { let _ = p.reply.send("EJECT".into()); }
                        }
                    });
                });
        }

        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Safe USB Eject — TCDOVERLORD")
            .with_inner_size([920.0, 640.0])
            .with_min_inner_size([760.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native("Safe USB Eject — TCDOVERLORD", options, Box::new(|_cc| Ok(Box::new(SafeUsbApp::new()))))
}
