# Angel Project Information

## Project Identity

| Field | Value |
|---|---|
| Project | Safe USB Eject by TCDOVERLORD |
| Owner | TCDOVERLORD |
| Current version | v1.0.0 |
| Six-Color phase | Blue |
| Engineering state | Working release |
| Development mode | Maintenance only |
| Repository status | Released and intentionally paused |
| Primary platform | Windows |
| Primary language | PowerShell |
| Launcher | Windows batch file |
| Resume difficulty | Low to moderate |
| Last reviewed | 2026-07-17 |

## Project Summary

Safe USB Eject is a safety-first Windows utility for identifying removable USB drives, detecting applications that are holding a selected drive open, requesting graceful application closure, rechecking locks, and then requesting a safe Windows eject.

The project is considered complete for its current purpose. It is not abandoned. Development is intentionally paused because the owner is satisfied with the current v1.0.0 release.

## Read First

1. `PROJECT_CONTINUITY.md`
2. `README.md`
3. `docs/SAFETY.md`
4. `scripts/Safe-USB-Eject.ps1`
5. `SAFE_USB_EJECT.bat`
6. `CHANGELOG.md`
7. `LICENSE`

## Stable Areas to Protect

- Graceful close must be attempted before force-close is offered.
- Force-close must require explicit user approval.
- Critical Windows processes must remain protected.
- The tool must stop when locks remain.
- Eject must require explicit user confirmation.
- Diagnostic logging must remain available.
- Existing working behavior must be preserved before modification.

## Resume Rule

Resume development only for:

- A confirmed defect.
- A Windows compatibility problem.
- A security or safety concern.
- An owner-approved feature.
- A deliberately planned new release.
