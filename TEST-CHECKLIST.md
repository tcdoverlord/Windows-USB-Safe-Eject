# Windows QA checklist — Rust UI preview

Test only with a spare USB drive containing disposable files.

- [ ] Build with `BUILD-EXE.bat` on Windows 10/11.
- [ ] Launch from `dist\SafeUSB-Eject.exe`; no console window should appear.
- [ ] Open app with no USB connected; window remains responsive.
- [ ] Connect a USB while app is open; drive appears after background scan.
- [ ] Disconnect a safely ejected drive; row disappears on refresh.
- [ ] Connect/disconnect several times; no freeze, crash, duplicate rows or 0 GB/Unknown transient rows.
- [ ] Click Refresh repeatedly during device mount; window remains responsive.
- [ ] Select a drive, then disconnect it before confirming; engine must refuse because it is no longer detected as removable.
- [ ] Cancel GUI confirmation; no eject request occurs.
- [ ] Successful eject: no console window; GUI refreshes and reports only verified results.
- [ ] Keep a file open on the USB; engine requests graceful closure first.
- [ ] If locks remain, choose Cancel and confirm the drive remains mounted.
- [ ] On a disposable test, verify force-close is a separate explicit choice and protected processes are never terminated.
- [ ] Force-close path rechecks locks and stops if locks remain.
- [ ] Simulate Windows refusing the eject; UI reports failure/unverified, not success.
- [ ] Verify dated logs are created and readable.
- [ ] Test paths containing spaces.
- [ ] Confirm original `legacy/Safe-USB-Eject-v1.0.0.ps1` remains unchanged.
