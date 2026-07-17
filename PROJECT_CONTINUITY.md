# Project Continuity

## Mission

This document is the long-term engineering memory for this project.

Its purpose is to preserve enough knowledge that the original author, another developer, or an AI assistant can understand the project's purpose, current state, engineering decisions, and safest path forward months or years after active development has stopped.

If this document is kept current, it should answer:

- What is this project?
- Why was it created?
- Why did development stop?
- What Six-Color phase was it left in?
- What has been completed?
- What remains optional?
- What has been tested?
- What has not been tested?
- What should never be changed without good reason?
- How should development safely resume?

The goal is to preserve engineering knowledge, reduce rediscovery, and provide a complete handoff for future development.

## Project Snapshot

| Field | Current State |
|---|---|
| Project | Safe USB Eject by TCDOVERLORD |
| Current version | v1.0.0 |
| Project owner | TCDOVERLORD |
| Repository status | Released and intentionally paused |
| Development mode | Maintenance only |
| Six-Color phase | Blue |
| Last continuity review | 2026-07-17 |
| Recommended priority | Low unless a defect is reported |
| Resume difficulty | Low to moderate |

## Current Decision

Development is intentionally stopping at version 1.0.0 because the owner is satisfied with the current GitHub release.

This is not an abandoned or failed project. It reached its original purpose and is being preserved as a stable release. Future development is optional.

## Why the Project Exists

Safe USB Eject is a safety-first Windows utility that:

- Detects removable USB drives.
- Finds applications holding a selected drive open.
- Requests graceful application closure first.
- Offers force-close only with explicit user approval.
- Rechecks locks before continuing.
- Requests a volume dismount and Windows eject.
- Records activity in dated log files.

## Six-Color Development Position

**Current color: Blue**

For this project handoff, Blue means the utility has moved through planning and implementation and has been released in a usable form. Active feature development has stopped by owner choice.

The next phase is not required. Work should resume only for:

- A confirmed defect.
- A Windows compatibility problem.
- A security or safety concern.
- A feature the owner deliberately approves.
- A planned new release.

Do not restart development merely because optional roadmap items remain.

## What Is Complete

The v1.0.0 repository includes:

- `SAFE_USB_EJECT.bat` launcher.
- `scripts/Safe-USB-Eject.ps1` PowerShell engine.
- Removable-drive detection.
- Restart Manager lock detection.
- Graceful process-close workflow.
- Optional force-close workflow.
- Protected-process safeguards.
- Lock rechecking before eject.
- Volume dismount request.
- Windows Shell eject request.
- Dated logging.
- User confirmation requiring `EJECT`.
- Safety documentation.
- Installation and usage instructions.
- License and GitHub-ready README.

## Known Incomplete or Optional Work

The README lists these optional roadmap ideas:

- Graphical interface.
- System tray launcher.
- Device model and serial display.
- Configurable protected-process list.
- Optional Sysinternals Handle integration.
- Signed release build.
- Final screenshots replacing placeholders.
- Star History chart after the public repository has enough history.

These are not blockers for v1.0.0.

## Test Status

### Documented recommended test

`docs/SAFETY.md` describes a manual test using a spare USB drive and a file opened in Notepad.

### Verified from repository inspection

- The expected launcher, PowerShell script, documentation, image, license, log folder, and ignore file are present.
- The PowerShell script declares Windows PowerShell 5.1 or later.
- The script uses strict mode, error handling, logging, administrator detection, Restart Manager integration, and user confirmations.

### Not verified in this continuity review

- No live USB-device test was performed during this review.
- No test was performed across multiple Windows 10 or Windows 11 builds.
- No test was performed with every USB file system or hardware type.
- No automated test suite is present.
- Code signing has not been verified.

Do not change this section to claim successful testing unless the tests were actually performed and recorded.

## Known Issues

No confirmed critical issue is recorded in the supplied v1.0.0 project package.

That statement does not mean the software is proven defect-free. Any future confirmed defect should be recorded here and in `CHANGELOG.md`.

## Stable Areas to Protect

When continuing this project, preserve these behaviors unless a documented reason requires a change:

- Graceful close must happen before force-close is offered.
- Force-close must never be automatic.
- Critical Windows processes must remain protected.
- The tool must stop when locks remain.
- The user must explicitly confirm eject.
- Logs must remain available for diagnosis.
- The batch launcher and PowerShell engine should remain separate unless a replacement design is approved.
- Existing working behavior should be backed up before modification.

## Files to Read First

Read these files in this order:

1. `PROJECT_CONTINUITY.md`
2. `README.md`
3. `docs/SAFETY.md`
4. `scripts/Safe-USB-Eject.ps1`
5. `SAFE_USB_EJECT.bat`
6. `CHANGELOG.md`
7. `LICENSE`

## Smallest Responsible Resume Plan

When development resumes:

1. Create a backup or version tag for v1.0.0.
2. Record the reason for reopening development.
3. Reproduce the issue or define the approved feature.
4. Change the smallest responsible module.
5. Test first with a spare USB drive containing non-critical copied data.
6. Record the operating system, drive type, file system, steps, and result.
7. Update `README.md`, `PROJECT_CONTINUITY.md`, and `CHANGELOG.md`.
8. Commit, push, and verify the repository.
9. Create a new release only after the result is reviewed.

## Recovery Point

The v1.0.0 source package is the current known release baseline.

Before future work, preserve it using at least one of these methods:

- Git tag: `v1.0.0`
- GitHub Release: `v1.0.0`
- Version Vault source archive
- Separate local backup

Never overwrite the only known working copy.

## Suggested Git Status

After adding the continuity files, the expected new files are:

```text
VERSION
PROJECT_CONTINUITY.md
CHANGELOG.md
ANGEL_PROJECT_INFO.md
docs/SAFETY.md
```

No source-code modification is required for this handoff.

## Recommended Commit

```text
docs: add Angel repository continuity files
```

## Handoff Statement

Safe USB Eject v1.0.0 is considered complete for its current purpose and is intentionally entering maintenance mode.

A future developer should begin with the documented safety rules, protect the v1.0.0 baseline, verify any problem before changing code, and make the smallest reversible improvement.
