# Changelog

## 0.2.0 — Rust UI preview (unreleased)
- Replaced the unstable PowerShell GUI with an egui Rust desktop UI.
- Moved drive enumeration into background workers with three-second refresh.
- Runs the preserved safety engine without showing a console window.
- Added a GUI protocol for confirmation and optional force-close prompts.
- Preserved the stable v1.0.0 PowerShell engine under `legacy/`.
- Not compiled or hardware-tested in the current environment.

## 0.2.1 — compile fix
- Fixed a Rust type mismatch in the no-exit-code status branch by converting the string literal to `String`.

## 0.2.2
- Prevents the UI from trapping the user indefinitely if the safety helper stops responding.
- Adds an explicit close-anyway path with an unverified-ejection warning; pending safety prompts are answered with cancellation.
- Normal Exit remains unavailable during an active ejection.

## 0.2.3 — honor close request after ejection
- If the user requests window closure while ejection is running, show the existing safety warning and remember that request.
- If the helper finishes while the warning is still pending, close the UI automatically instead of leaving it open after completion.
- Choosing “Keep Waiting” clears the pending close request; choosing “Close UI Anyway…” remains an explicit close path.
- Normal completion still refreshes the drive list when no close request is pending.
- Source update only; Windows compilation and physical USB testing are still required.

## 0.2.4
- After the safety engine returns exit code 0, display “Eject completed successfully. Exiting app…” and automatically close after two seconds.
- Disable further eject attempts after a successful operation and skip post-success drive scans, avoiding a second eject attempt in this build.
- Nonzero or unknown exit codes remain unverified and do not trigger the success auto-close.


## 0.2.5 — wait for physical USB removal
- After the PowerShell safety engine returns success, the UI explicitly reports that the script ran successfully.
- Captures the selected volume's physical disk PNP identity before ejection and watches for that physical device to disappear.
- Keeps the UI open and blocks ordinary close/Exit while waiting for physical removal; closes when the device is no longer enumerated.
- If physical identity cannot be captured, reports the limitation and leaves manual Exit available instead of pretending removal was detected.
- Windows compilation and hardware testing are still required.


## 0.2.6 — clear red/green status
- Added a prominent red status panel for NOT READY / success not verified.
- Added a prominent green status panel when the safety script returns exit code 0.
- Green status explicitly means the script completed successfully; Windows' safe-removal notification remains the authority before unplugging.
- Updated the footer guidance to explain the red/green states.
- Source-only update; build and physical-device behavior still require Windows testing.
