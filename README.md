# Safe USB Eject by TCDOVERLORD

A Windows utility designed to make USB drive removal easier and safer.

🌐 **[Visit the Safe USB Eject Website](https://tcdoverlord.github.io/Windows-USB-Safe-Eject/)**

## Features

- Select an available USB drive.
- Follow a guided safe-removal workflow.
- Use the Rust graphical interface and companion PowerShell script.
- Review status information during the process.

## Download

- [SafeUSB-Eject.exe](dist/SafeUSB-Eject.exe)
- [Companion PowerShell script](dist/scripts/Safe-USB-Eject.ps1)

**Important:** Keep the companion PowerShell script in its expected location relative to the executable. Do not assume the executable is standalone.

## Demonstration

![Safe USB Eject animated demonstration](assets/vids/usbsafeject_gif_video.gif)

## How to Build

Requirements: Windows, Rust/Cargo, and PowerShell.

Clone the repository:

```powershell
git clone https://github.com/tcdoverlord/Windows-USB-Safe-Eject.git
cd Windows-USB-Safe-Eject
.\BUILD-EXE.bat
```

The build script compiles the application and copies the executable and companion script into `dist`.

## Safety Notes

- Close files and applications that may be using the USB drive.
- Do not disconnect the drive if Windows reports that it is still in use.
- If ejecting fails, resolve the issue before retrying.
- Verify Windows' status before physically disconnecting the device.

Read the [safety documentation](docs/SAFETY.md) before use.

## Issues and Contributions

[Report a bug or request an improvement](https://github.com/tcdoverlord/Windows-USB-Safe-Eject/issues).

## License

Review the repository's `LICENSE` file for the applicable terms. Confirm the license metadata in `Cargo.toml` matches those terms before redistributing the project.

---

Created by [TCDOVERLORD](https://github.com/tcdoverlord).