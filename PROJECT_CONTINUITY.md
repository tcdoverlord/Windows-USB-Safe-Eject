# Project continuity — Safe USB Eject

- Stable baseline: v1.0.0 PowerShell utility, intentionally paused.
- New experiment: Rust/egui UI preview 0.2.0.
- Safety-critical behavior remains in the adapted PowerShell engine until the Rust UI passes Windows QA.
- Do not overwrite or delete the stable baseline.
- Release only after completing `TEST-CHECKLIST.md` on Windows 10/11 with a spare USB device.
