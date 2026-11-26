# Capture the client area of the Rust panel window into a PNG that is pixel-comparable
# with harness/antd_shots (browser element screenshots at the same device scale).
#
#   powershell -File harness/capture_rust.ps1 -Out harness/baseline/rust-button-solid.png
#   powershell -File harness/capture_rust.ps1 -Out ... -Hover 164,28
#
# -Hover moves the real cursor to client-area coordinates: gpui only applies hover /
# active styles from a real pointer, and the antd side hovers the same data-cell in the browser.
#
# NOTE: keep this file ASCII-only. Windows PowerShell 5.1 reads scripts as ANSI unless a
# UTF-8 BOM is present, so any Chinese comment silently breaks string terminators.
param(
    [Parameter(Mandatory = $true)][string]$Out,
    [string]$Process = "panel_shot",
    [int]$TimeoutSec = 90,
    [string]$Hover = ""
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Windows.Forms, System.Drawing

$q = [char]34
$cs = "using System;using System.Runtime.InteropServices;using System.Drawing;"
$cs += "public struct RECT{public int L,T,R,B;}"
$cs += "public struct POINT{public int X,Y;}"
$cs += "public class U32{"
$cs += "[DllImport($q" + "user32.dll" + $q + ")]public static extern bool SetProcessDPIAware();"
$cs += "[DllImport($q" + "user32.dll" + $q + ")]public static extern bool GetClientRect(IntPtr h,out RECT r);"
$cs += "[DllImport($q" + "user32.dll" + $q + ")]public static extern bool ClientToScreen(IntPtr h,ref POINT p);"
$cs += "[DllImport($q" + "user32.dll" + $q + ")]public static extern bool SetForegroundWindow(IntPtr h);"
$cs += "[DllImport($q" + "user32.dll" + $q + ")]public static extern bool BringWindowToTop(IntPtr h);"
$cs += "[DllImport($q" + "user32.dll" + $q + ")]public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);"
$cs += "[DllImport($q" + "user32.dll" + $q + ")]public static extern bool SetCursorPos(int x,int y);"
$cs += "}"
Add-Type -TypeDefinition $cs

[void][U32]::SetProcessDPIAware()

$deadline = (Get-Date).AddSeconds($TimeoutSec)
$proc = $null
while ($null -eq $proc) {
    $proc = Get-Process | Where-Object { $_.ProcessName -like "$Process*" -and $_.MainWindowHandle -ne 0 } | Select-Object -First 1
    if ($null -ne $proc) { break }
    if ((Get-Date) -gt $deadline) { Write-Error "process $Process not found within $TimeoutSec s"; exit 1 }
    Start-Sleep -Milliseconds 250
}

$hwnd = $proc.MainWindowHandle
# Background shells lack foreground rights: tap Alt so SetForegroundWindow is allowed,
# otherwise CopyFromScreen grabs whatever window overlaps the panel.
[U32]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
[U32]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
[void][U32]::SetForegroundWindow($hwnd)
[void][U32]::BringWindowToTop($hwnd)
Start-Sleep -Milliseconds 400

$cr = New-Object RECT
[void][U32]::GetClientRect($hwnd, [ref]$cr)
$origin = New-Object POINT
[void][U32]::ClientToScreen($hwnd, [ref]$origin)
$left = $origin.X
$top = $origin.Y
$width = $cr.R - $cr.L
$height = $cr.B - $cr.T

if ($Hover -ne "") {
    $xy = $Hover.Split(",")
    [void][U32]::SetCursorPos($left + [int]$xy[0], $top + [int]$xy[1])
    Start-Sleep -Milliseconds 350
}

$bmp = New-Object System.Drawing.Bitmap($width, $height, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($left, $top, 0, 0, (New-Object System.Drawing.Size($width, $height)))
$g.Dispose()

$outFull = [System.IO.Path]::GetFullPath($Out)
New-Item -ItemType Directory -Force -Path ([System.IO.Path]::GetDirectoryName($outFull)) | Out-Null
if (Test-Path $outFull) { Remove-Item $outFull -Force }
$bmp.Save($outFull, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()

Write-Output ("saved $outFull ${width}x${height} client-origin $left,$top")
