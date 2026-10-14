#requires -Version 5.1
param(
    [string]$DriveLetter,
    [switch]$GuiMode,
    [switch]$ConfirmedByGui
)
<#+
.SYNOPSIS
Safely closes applications using a removable USB drive and requests Windows to eject it.

.DESCRIPTION
1. Lists removable USB volumes.
2. Uses Windows Restart Manager to identify processes holding files on the selected drive.
3. Attempts a graceful close first.
4. Offers an optional force-close only when necessary.
5. Requests the normal Windows Shell eject action after the final lock check.

The script will not intentionally stop Windows critical processes.
+#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($GuiMode) {
    function Write-Host {
        param(
            [Parameter(Position=0, ValueFromRemainingArguments=$true)]
            [object[]]$Object,
            [ConsoleColor]$ForegroundColor
        )
        $message = (@($Object) | ForEach-Object { [string]$_ }) -join ' '
        [Console]::Out.WriteLine("__SAFEUSB_LOG__" + $message)
        [Console]::Out.Flush()
    }
    function Read-Host {
        param([string]$Prompt)
        [Console]::Out.WriteLine("__SAFEUSB_PROMPT__" + $Prompt)
        [Console]::Out.Flush()
        $answer = [Console]::In.ReadLine()
        if ($null -eq $answer) { return '' }
        return $answer
    }
}

$ProjectRoot = Split-Path -Parent $PSScriptRoot
$LogDirectory = Join-Path $ProjectRoot 'logs'
if (-not (Test-Path $LogDirectory)) {
    New-Item -ItemType Directory -Path $LogDirectory -Force | Out-Null
}
$LogFile = Join-Path $LogDirectory ("SafeUSB_{0}.log" -f (Get-Date -Format 'yyyy-MM-dd'))

function Write-Log {
    param(
        [Parameter(Mandatory)] [string] $Message,
        [ValidateSet('INFO','WARN','ERROR','SUCCESS')] [string] $Level = 'INFO'
    )

    $line = "{0} [{1}] {2}" -f (Get-Date -Format 'yyyy-MM-dd HH:mm:ss'), $Level, $Message
    Add-Content -LiteralPath $LogFile -Value $line

    switch ($Level) {
        'WARN'    { Write-Host $Message -ForegroundColor Yellow }
        'ERROR'   { Write-Host $Message -ForegroundColor Red }
        'SUCCESS' { Write-Host $Message -ForegroundColor Green }
        default   { Write-Host $Message }
    }
}

function Test-IsAdministrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Get-RemovableVolumes {
    $volumes = Get-CimInstance Win32_LogicalDisk -Filter "DriveType = 2" |
        Where-Object { $_.DeviceID -and $_.Size -gt 0 } |
        Sort-Object DeviceID

    foreach ($volume in $volumes) {
        [pscustomobject]@{
            DriveLetter = $volume.DeviceID
            Label       = if ($volume.VolumeName) { $volume.VolumeName } else { 'NO LABEL' }
            FileSystem  = $volume.FileSystem
            SizeGB      = [math]::Round($volume.Size / 1GB, 2)
            FreeGB      = [math]::Round($volume.FreeSpace / 1GB, 2)
        }
    }
}

if (-not ('RestartManager.NativeMethods' -as [type])) {
    Add-Type -Language CSharp -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.InteropServices;

namespace RestartManager
{
    public static class NativeMethods
    {
        const int CCH_RM_SESSION_KEY = 32;
        const int ERROR_MORE_DATA = 234;

        [StructLayout(LayoutKind.Sequential)]
        struct RM_UNIQUE_PROCESS
        {
            public int dwProcessId;
            public System.Runtime.InteropServices.ComTypes.FILETIME ProcessStartTime;
        }

        enum RM_APP_TYPE
        {
            RmUnknownApp = 0,
            RmMainWindow = 1,
            RmOtherWindow = 2,
            RmService = 3,
            RmExplorer = 4,
            RmConsole = 5,
            RmCritical = 1000
        }

        [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
        struct RM_PROCESS_INFO
        {
            public RM_UNIQUE_PROCESS Process;
            [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 256)]
            public string strAppName;
            [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 64)]
            public string strServiceShortName;
            public RM_APP_TYPE ApplicationType;
            public uint AppStatus;
            public uint TSSessionId;
            [MarshalAs(UnmanagedType.Bool)]
            public bool bRestartable;
        }

        [DllImport("rstrtmgr.dll", CharSet = CharSet.Unicode)]
        static extern int RmStartSession(out uint pSessionHandle, int dwSessionFlags, string strSessionKey);

        [DllImport("rstrtmgr.dll")]
        static extern int RmEndSession(uint pSessionHandle);

        [DllImport("rstrtmgr.dll", CharSet = CharSet.Unicode)]
        static extern int RmRegisterResources(uint pSessionHandle,
            uint nFiles, string[] rgsFilenames,
            uint nApplications, IntPtr rgApplications,
            uint nServices, string[] rgsServiceNames);

        [DllImport("rstrtmgr.dll")]
        static extern int RmGetList(uint dwSessionHandle,
            out uint pnProcInfoNeeded,
            ref uint pnProcInfo,
            [In, Out] RM_PROCESS_INFO[] rgAffectedApps,
            ref uint lpdwRebootReasons);

        public static int[] GetLockingProcessIds(string path)
        {
            uint handle;
            string key = Guid.NewGuid().ToString("N").Substring(0, CCH_RM_SESSION_KEY);
            int result = RmStartSession(out handle, 0, key);
            if (result != 0) throw new Exception("RmStartSession failed: " + result);

            try
            {
                string[] resources = new string[] { path };
                result = RmRegisterResources(handle, (uint)resources.Length, resources, 0, IntPtr.Zero, 0, null);
                if (result != 0) throw new Exception("RmRegisterResources failed: " + result);

                uint needed = 0;
                uint count = 0;
                uint reasons = 0;
                result = RmGetList(handle, out needed, ref count, null, ref reasons);

                if (result == ERROR_MORE_DATA)
                {
                    var processInfo = new RM_PROCESS_INFO[needed];
                    count = needed;
                    result = RmGetList(handle, out needed, ref count, processInfo, ref reasons);
                    if (result != 0) throw new Exception("RmGetList failed: " + result);

                    var ids = new List<int>();
                    for (int i = 0; i < count; i++)
                        ids.Add(processInfo[i].Process.dwProcessId);
                    return ids.ToArray();
                }

                if (result == 0) return new int[0];
                throw new Exception("RmGetList failed: " + result);
            }
            finally
            {
                RmEndSession(handle);
            }
        }
    }
}
'@
}

function Get-LockingProcesses {
    param([Parameter(Mandatory)] [string] $DriveRoot)

    $ids = [RestartManager.NativeMethods]::GetLockingProcessIds($DriveRoot)
    $protectedNames = @('System','Idle','Registry','smss','csrss','wininit','services','lsass','winlogon')

    foreach ($id in ($ids | Sort-Object -Unique)) {
        if ($id -eq $PID) { continue }
        try {
            $process = Get-Process -Id $id -ErrorAction Stop
            [pscustomobject]@{
                Id        = $process.Id
                Name      = $process.ProcessName
                MainTitle = $process.MainWindowTitle
                Protected = $protectedNames -contains $process.ProcessName
                Process   = $process
            }
        } catch {
            Write-Log "A locking process with PID $id ended before it could be inspected." 'INFO'
        }
    }
}

function Request-GracefulClose {
    param([Parameter(Mandatory)] [object[]] $LockingProcesses)

    foreach ($item in $LockingProcesses) {
        if ($item.Protected) {
            Write-Log "Skipping protected Windows process: $($item.Name) (PID $($item.Id))." 'WARN'
            continue
        }

        try {
            if ($item.Process.MainWindowHandle -ne 0) {
                Write-Log "Requesting a graceful close: $($item.Name) (PID $($item.Id))"
                [void]$item.Process.CloseMainWindow()
            } else {
                Write-Log "No closeable window found for $($item.Name) (PID $($item.Id))." 'WARN'
            }
        } catch {
            Write-Log "Could not request close for $($item.Name): $($_.Exception.Message)" 'WARN'
        }
    }

    Start-Sleep -Seconds 3
}

function Stop-RemainingProcesses {
    param([Parameter(Mandatory)] [object[]] $LockingProcesses)

    foreach ($item in $LockingProcesses) {
        if ($item.Protected) {
            Write-Log "Refusing to force-stop protected process: $($item.Name) (PID $($item.Id))." 'WARN'
            continue
        }

        try {
            Stop-Process -Id $item.Id -Force -ErrorAction Stop
            Write-Log "Force-closed $($item.Name) (PID $($item.Id))." 'WARN'
        } catch {
            Write-Log "Could not force-close $($item.Name): $($_.Exception.Message)" 'ERROR'
        }
    }

    Start-Sleep -Seconds 2
}

function Invoke-ShellEject {
    param([Parameter(Mandatory)] [string] $DriveLetter)

    $shell = New-Object -ComObject Shell.Application
    $computer = $shell.Namespace(17)
    $item = $computer.ParseName($DriveLetter)
    if (-not $item) {
        throw "Windows Shell could not locate $DriveLetter."
    }

    $ejectVerb = $item.Verbs() | Where-Object {
        ($_.Name -replace '&','').Trim() -match '^(Eject|Safely Remove)$'
    } | Select-Object -First 1

    if ($ejectVerb) {
        $ejectVerb.DoIt()
        return
    }

    # Fallback: invoke the canonical Eject verb directly.
    $item.InvokeVerb('Eject')
}

if (-not $GuiMode) { Clear-Host }
Write-Host '============================================================' -ForegroundColor Green
Write-Host '               SAFE USB EJECT - TCDOVERLORD' -ForegroundColor Green
Write-Host '============================================================' -ForegroundColor Green
Write-Host

try {
    $volumes = @(Get-RemovableVolumes)
    if ($volumes.Count -eq 0) {
        Write-Log 'No removable USB drives were detected.' 'ERROR'
        exit 2
    }

    $selected = $null
    if (-not [string]::IsNullOrWhiteSpace($DriveLetter)) {
        $normalizedDrive = $DriveLetter.Trim().TrimEnd('\')
        if ($normalizedDrive.Length -eq 1) { $normalizedDrive += ':' }
        $selected = $volumes | Where-Object { $_.DriveLetter -ieq $normalizedDrive } | Select-Object -First 1
        if (-not $selected) {
            Write-Log "The selected drive $normalizedDrive is no longer detected as removable. Refresh the list and try again." 'ERROR'
            exit 3
        }
        Write-Host ("Selected from GUI: {0}  {1}  {2} GB free of {3} GB  {4}" -f $selected.DriveLetter, $selected.Label, $selected.FreeGB, $selected.SizeGB, $selected.FileSystem)
    } else {
        Write-Host 'Detected removable drives:' -ForegroundColor Cyan
        for ($i = 0; $i -lt $volumes.Count; $i++) {
            $v = $volumes[$i]
            Write-Host ("[{0}] {1}  {2}  {3} GB free of {4} GB  {5}" -f ($i + 1), $v.DriveLetter, $v.Label, $v.FreeGB, $v.SizeGB, $v.FileSystem)
        }

        Write-Host
        $selection = Read-Host 'Choose the USB drive number to eject'
        $selectedIndex = 0
        if (-not [int]::TryParse($selection, [ref]$selectedIndex) -or $selectedIndex -lt 1 -or $selectedIndex -gt $volumes.Count) {
            Write-Log 'Invalid selection. Nothing was changed.' 'ERROR'
            exit 3
        }
        $selected = $volumes[$selectedIndex - 1]
    }
    $driveLetter = $selected.DriveLetter
    $driveRoot = "$driveLetter\"

    Write-Log "Selected $driveLetter ($($selected.Label))."
    if ($GuiMode -and $ConfirmedByGui) {
        Write-Log "Eject confirmation received in the GUI for $driveLetter."
    } else {
        Write-Host
        Write-Host 'Close any open documents stored on this USB before continuing.' -ForegroundColor Yellow
        $confirm = Read-Host "Type EJECT to continue with $driveLetter"
        if ($confirm -ine 'EJECT') {
            Write-Log 'Cancelled by user. Nothing was changed.' 'WARN'
            exit 4
        }
    }

    $locking = @(Get-LockingProcesses -DriveRoot $driveRoot)
    if ($locking.Count -gt 0) {
        Write-Host
        Write-Host 'Applications currently using the USB:' -ForegroundColor Yellow
        $locking | ForEach-Object {
            Write-Host (" - {0} (PID {1}) {2}" -f $_.Name, $_.Id, $_.MainTitle)
        }

        Request-GracefulClose -LockingProcesses $locking
        $remaining = @(Get-LockingProcesses -DriveRoot $driveRoot)

        if ($remaining.Count -gt 0) {
            Write-Host
            Write-Host 'Some applications still have the USB open:' -ForegroundColor Yellow
            $remaining | ForEach-Object { Write-Host (" - {0} (PID {1})" -f $_.Name, $_.Id) }
            Write-Host
            $force = Read-Host 'Type FORCE to close non-critical remaining processes, or press Enter to cancel'
            if ($force -ieq 'FORCE') {
                Stop-RemainingProcesses -LockingProcesses $remaining
            } else {
                Write-Log 'Eject cancelled because applications still have the USB open.' 'WARN'
                exit 5
            }
        }
    } else {
        Write-Log 'No locking applications were reported by Windows Restart Manager.'
    }

    $finalLocks = @(Get-LockingProcesses -DriveRoot $driveRoot)
    if ($finalLocks.Count -gt 0) {
        Write-Log 'The USB is still in use. Eject was stopped to protect your data.' 'ERROR'
        exit 6
    }

    Write-Log 'The application-lock check is best-effort; Windows remains the final authority on whether the device can be removed.' 'WARN'
    Write-Log 'Requesting one normal Windows Shell eject. If Windows says the drive is in use, cancel that dialog, close the program or file using the drive, and retry.'
    Invoke-ShellEject -DriveLetter $driveLetter

    # Shell eject can complete asynchronously. Do not treat a short delay or a
    # failed CIM query as success; wait up to 20 seconds for Windows to stop
    # reporting the selected logical volume as mounted.
    $timeoutSeconds = 20
    $deadline = (Get-Date).AddSeconds($timeoutSeconds)
    $verifiedUnmount = $false
    $lastQueryError = $null
    while ((Get-Date) -lt $deadline) {
        try {
            $currentVolume = Get-CimInstance Win32_LogicalDisk -Filter "DeviceID = '$driveLetter'" -ErrorAction Stop
            if (-not $currentVolume) {
                $verifiedUnmount = $true
                break
            }
        } catch {
            # A query error is not proof of ejection. Remember it and keep
            # waiting; the timeout path below will report an unverified result.
            $lastQueryError = $_.Exception.Message
        }
        Start-Sleep -Milliseconds 500
    }

    if (-not $verifiedUnmount) {
        if ($lastQueryError) {
            Write-Log "Eject outcome could not be verified after $timeoutSeconds seconds. Windows volume query error: $lastQueryError. Do not unplug unless Windows explicitly confirms safe removal." 'ERROR'
        } else {
            Write-Log "Windows did not confirm that $driveLetter was unmounted within $timeoutSeconds seconds. The eject outcome is unverified; do not unplug unless Windows explicitly confirms safe removal. Close open files and retry." 'ERROR'
        }
        exit 7
    }

    Write-Log "Windows no longer reports $driveLetter as a mounted logical volume. Safe-removal status verified." 'SUCCESS'
    exit 0
}
catch {
    Write-Log $_.Exception.Message 'ERROR'
    exit 1
}
