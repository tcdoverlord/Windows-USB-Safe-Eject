# Safety model

## Preserved behavior

The PowerShell engine remains responsible for:
1. Enumerating removable logical volumes and revalidating the selected drive.
2. Registering the drive root with Windows Restart Manager to identify applications reported as using it.
3. Requesting graceful application closure first.
4. Rechecking the locks.
5. Offering a separately confirmed force-close only if locks remain.
6. Protecting critical Windows processes.
7. Rechecking locks after any force-close and stopping if locks remain.
8. Requesting the normal Windows Shell eject operation.
9. Waiting for Windows to stop reporting the logical volume as mounted before reporting success.
10. Writing a dated log.

## Rust UI responsibilities

- Scanning drives asynchronously so slow CIM calls do not block the UI.
- Showing a clear drive selection and an explicit confirmation.
- Launching the engine without a console window.
- Relaying prompts to the user and forwarding only their explicit responses.
- Reporting nonzero or unverified exit results as failure, never as success.

## Limitations

Restart Manager does not detect every possible open handle or pending write. The Windows Shell and device stack remain the final authority. The UI does not attempt to bypass Windows in-use warnings. If ejection is unverified, do not unplug until Windows explicitly confirms safe removal.
