# Safe USB Eject — Rust UI Preview

**Owner:** TCDOVERLORD  
**Stable baseline:** PowerShell v1.0.0, intentionally preserved  
**Rust UI preview:** v0.2.0  
**Status:** Source package; not compiled or tested on Windows in this environment.

## Why this rebuild exists

The previous GUI versions became unreliable after changes to process callbacks and synchronous drive scans. This project replaces the GUI with a Rust/egui desktop application. Drive discovery runs in background threads, so a slow Windows USB mount query should not freeze the interface. The existing safety engine is retained for its Restart Manager lock checks and safe-eject flow instead of reimplementing safety-critical behavior untested.

## Features

- Native desktop window with a drive list, selection, status and activity log.
- Background removable-drive scanning on startup, manual refresh, and every three seconds.
- Drive scanning does not run on the GUI thread.
- GUI confirmation before the safety engine is started.
- Safety engine runs without opening a console window.
- Engine prompts are relayed into the GUI, including the optional force-close decision.
- Normal window-close requests are deferred while an eject operation is in progress.
- Graceful close is attempted before force-close is offered; critical Windows processes remain protected.
- The engine rechecks locks and stops if locks remain.
- Logs are retained under `logs/` relative to the project/engine directory.
- The stable original PowerShell engine is included as `legacy/Safe-USB-Eject-v1.0.0.ps1`.

## Build on Windows

1. Install Rust using [rustup.rs](https://rustup.rs/). Choose the stable toolchain and reopen your terminal.
2. Extract this folder to a normal local directory.
3. Double-click `BUILD-EXE.bat`, or run `cargo build --release`.
4. The launcher and script are copied to `dist/`.
5. Run `dist\SafeUSB-Eject.exe`.

The first Cargo build downloads dependencies from crates.io and requires internet access. Subsequent builds use the local Cargo cache.

## Safety notes

- The GUI selection is not a guarantee that the drive is still present; the engine revalidates the selected drive before acting.
- Restart Manager detection is best-effort. Windows can still report the drive is in use.
- If Windows blocks the eject, cancel its warning, close programs and files using the drive, and retry. Do not force through a warning when you are unsure whether writes are pending.
- Force-close is optional and requires a separate GUI decision after a graceful-close attempt and lock recheck.
- The app does not claim ejection success solely because a Shell eject request was sent. The engine verifies that Windows no longer reports the volume as mounted.
- The EXE is unsigned; Windows SmartScreen may warn.
- Do not use this preview on a drive containing your only copy of important data until it passes the QA checklist.

## Why this is a hybrid for now

The GUI and background workflow are Rust. The known safety engine remains PowerShell temporarily so the move to Rust does not simultaneously replace the previously working lock-detection/eject logic. Once this preview passes Windows QA, the engine can be ported in a separate change with its own safety tests.

## Repository layout

- `src/main.rs` — Rust GUI, background drive scan, child-process supervision.
- `scripts/Safe-USB-Eject.ps1` — adapted safety engine with a GUI communication protocol.
- `legacy/Safe-USB-Eject-v1.0.0.ps1` — preserved baseline engine.
- `docs/SAFETY.md` — safety behavior and limitations.
- `TEST-CHECKLIST.md` — manual Windows validation.


### Window-close recovery (0.2.3)
If the safety helper has not reported completion, the app explains that the result is unverified and offers **Keep Waiting** or **Close UI Anyway…**. Closing the UI this way is not confirmation that the USB was safely ejected.


### One-eject test behavior (v0.2.4)
After the safety engine reports exit code 0, the app displays “Eject completed successfully. Exiting app…” for about two seconds and closes automatically. It will not scan or start another eject after a successful operation. Failed or unverifiable operations remain visible for review.


## Physical removal confirmation (v0.2.5)
After the safety engine returns success, the app displays **SCRIPT RAN SUCCESSFULLY** and waits for the same physical USB disk to disappear from Windows device enumeration. The window closes after physical removal is detected. While waiting, the normal Exit button and window close are blocked. If the physical device identity cannot be determined, the app explains that automatic removal detection is unavailable and allows manual exit. Windows' safe-removal notification remains the final authority; do not unplug unless Windows confirms safe removal.


### Red/green status indicator (v0.2.6)
- **Red — NOT READY / SUCCESS NOT VERIFIED:** the script is still running, failed, or has not returned a verified success code.
- **Green — SCRIPT SUCCESSFULLY COMPLETED:** the safety script returned exit code 0. This confirms script success, not a guarantee that the physical device has been removed; follow Windows' safe-removal notification before unplugging.
