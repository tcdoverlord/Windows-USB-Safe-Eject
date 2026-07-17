# Changelog

All notable changes to this project should be documented in this file.

The project owner controls version numbers, releases, tags, and milestones.

## [v1.0.0] - 2026-07-17

### Added

- Windows batch launcher: `SAFE_USB_EJECT.bat`
- PowerShell engine: `scripts/Safe-USB-Eject.ps1`
- Removable USB drive detection
- Restart Manager lock detection
- Graceful application-close workflow
- Optional force-close workflow with explicit approval
- Protected-process safeguards
- Lock rechecking before eject
- Volume dismount request
- Windows Shell eject request
- Dated activity logging
- Explicit eject confirmation requiring `EJECT`
- Safety documentation
- Installation and usage instructions
- Repository images and project artwork
- License and GitHub-ready README
- Project continuity documentation
- Angel project identity file
- Repository version dashboard

### Status

Released and intentionally paused.

The project is considered complete for its current purpose and has entered maintenance mode.

### Testing Notes

Repository structure and documented implementation were reviewed.

The following were not verified during the continuity review:

- Live USB-device behavior
- Multiple Windows 10 and Windows 11 builds
- Every USB hardware type
- Every supported file system
- Automated test coverage
- Code signing

Do not claim successful testing unless the relevant test was actually performed and recorded.

## Unreleased

No active development is currently planned.

Future entries should be added only when the owner deliberately resumes development.
