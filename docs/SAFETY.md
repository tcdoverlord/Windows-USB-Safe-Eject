# Safety Notes

Safe USB Eject is designed to reduce the risk of data corruption, not to bypass Windows safety checks.

## Important behavior

- The utility asks applications to close normally first.
- Force-close is never automatic.
- Force-close can cause unsaved work in the named application to be lost.
- Protected Windows processes are not force-stopped.
- If locks remain after the final scan, the eject operation stops.
- Do not unplug the USB until Windows reports that it is safe or the drive disappears from File Explorer.

## Recommended testing

1. Test first with a spare USB drive containing copied, non-critical files.
2. Open a text file from the USB in Notepad.
3. Run the utility and verify Notepad is listed.
4. Save or close the file before allowing eject.
5. Confirm the USB disappears from File Explorer.
6. Review the generated log.
