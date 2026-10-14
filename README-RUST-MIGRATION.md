# Rust migration decision

This iteration deliberately ports the unstable UI and background workflow first, not the Windows Restart Manager and Shell-eject engine. A complete one-step rewrite would replace both the broken GUI and the known safety behavior at the same time, making regressions harder to isolate.

The Rust app:
- owns the GUI, refresh scheduling, process supervision and user prompts;
- starts the engine without a console window;
- communicates through explicit line-prefixed log/prompt messages;
- uses worker threads for drive scanning and ejection.

After Windows QA, the Restart Manager and eject path can be ported into Rust as a separate milestone.
