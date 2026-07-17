# Safety Guide

Safe USB Eject is designed to reduce the risk of removing a USB drive while applications are still using it.

No software can guarantee that data loss is impossible. Use this utility carefully and keep backups of important files.

## Core Safety Rules

- Save all open work before using the utility.
- Close applications normally whenever possible.
- Use a spare USB drive for initial testing.
- Use copied, non-critical test data.
- Never force-close a process unless you understand the possible consequences.
- Never remove a drive while Windows is actively writing data.
- Stop immediately if the tool reports that locks remain.
- Review the log when behavior is unexpected.

## Intended Workflow

The utility should:

1. Detect removable USB drives.
2. Allow the user to select a drive.
3. Identify applications holding the selected drive open.
4. Request graceful application closure first.
5. Offer force-close only with explicit user approval.
6. Recheck for locks.
7. Stop if locks remain.
8. Require explicit eject confirmation.
9. Request a volume dismount and Windows eject.
10. Record activity in a dated log file.

## Protected Behavior

The following safety behaviors should not be removed without a documented and reviewed reason:

- Graceful close before force-close.
- Explicit approval before force-close.
- Protection for critical Windows processes.
- Lock rechecking before eject.
- Immediate stop when unresolved locks remain.
- Explicit user confirmation before eject.
- Diagnostic logging.

## Recommended Manual Test

Use a spare USB drive containing copied, non-critical data.

### Test preparation

1. Connect the spare USB drive.
2. Copy a small text file to the drive.
3. Open the file in Notepad.
4. Leave Notepad open.
5. Start `SAFE_USB_EJECT.bat`.

### Expected behavior

1. The utility detects the removable drive.
2. The utility identifies that Notepad is using the drive.
3. The utility requests a graceful close.
4. The user can decline any force-close request.
5. The utility rechecks for remaining locks.
6. The utility stops if locks remain.
7. The utility requests explicit confirmation before ejecting.
8. The activity is written to a dated log file.

### Record the result

Document:

- Windows edition and build
- USB drive type
- USB drive capacity
- File system
- Test steps
- Expected result
- Actual result
- Log file name
- Pass or fail
- Any unexpected behavior

## Force-Close Warning

Force-closing an application can cause:

- Unsaved work to be lost
- Application settings to be damaged
- Temporary files to remain
- File operations to be interrupted
- Data on the USB drive to become inconsistent

Force-close must never be automatic.

## Administrator Rights

Some lock detection or eject operations may require administrator privileges.

Administrator access does not make an unsafe action safe. Continue to follow all confirmation and lock-checking rules.

## Unsupported Claims

Do not claim that this project:

- Guarantees zero data loss
- Works with every USB device
- Works on every Windows build
- Supports every file system
- Has passed automated testing
- Is digitally signed

unless those claims have been specifically tested and documented.

## Incident Response

If unexpected behavior occurs:

1. Stop using the affected USB drive.
2. Do not repeatedly reconnect and remove it.
3. Preserve the relevant log file.
4. Record the Windows version, device details, and exact steps.
5. Back up readable data before attempting repairs.
6. Record the issue in `PROJECT_CONTINUITY.md` and `CHANGELOG.md`.
7. Resume development only after the problem is reproducible or clearly defined.

## Final Safety Statement

Use this utility as a cautious assistant, not as a substitute for backups, normal application shutdown, or Windows storage safeguards.
