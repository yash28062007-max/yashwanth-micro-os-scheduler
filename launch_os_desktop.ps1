# Yashwanth Micro-OS - Interactive Desktop Controller
$Host.UI.RawUI.WindowTitle = "Yashwanth Micro-OS - Desktop Terminal"
Clear-Host
Write-Host "===============================================================================" -ForegroundColor Cyan
Write-Host "           YASHWANTH MICRO-OS - INTERACTIVE DESKTOP CONTROLLER" -ForegroundColor White
Write-Host "===============================================================================" -ForegroundColor Cyan
Write-Host "[INFO] Project: Yashwanth Micro-OS (x86_64 Minimal Round Robin Kernel)" -ForegroundColor Yellow
Write-Host "[INFO] Directory: $PSScriptRoot" -ForegroundColor Yellow
Write-Host "[INFO] Active ISO: $PSScriptRoot\build\yashwanth_micro_os.iso" -ForegroundColor Yellow
Write-Host ""

function global:Start-VBox {
    Write-Host "[*] Starting Yashwanth Micro-OS in VirtualBox (GUI Window)..." -ForegroundColor Green
    & "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" startvm "Yashwanth Micro-OS" --type gui
}

function global:Stop-VBox {
    Write-Host "[*] Powering off Yashwanth Micro-OS in VirtualBox..." -ForegroundColor Red
    & "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" controlvm "Yashwanth Micro-OS" poweroff
}

function global:Run-QEMU {
    Write-Host "[*] Running in QEMU via WSL..." -ForegroundColor Magenta
    wsl bash -c "cd '$PSScriptRoot' && ./scripts/run_qemu.sh"
}

function global:Show-Screen {
    Write-Host "[*] Capturing live VirtualBox screenshot..." -ForegroundColor Cyan
    & "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" controlvm "Yashwanth Micro-OS" screenshotpng "$PSScriptRoot\results\vbox_interactive.png"
    Start-Process "$PSScriptRoot\results\vbox_interactive.png"
}

# Auto-launch VirtualBox on launch
Start-VBox

Write-Host ""
Write-Host "===============================================================================" -ForegroundColor Cyan
Write-Host " VirtualBox VM has been launched! The display window is now opening on screen." -ForegroundColor Green
Write-Host "===============================================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Interactive helper commands in this terminal:" -ForegroundColor White
Write-Host "   Start-VBox   - Launch the VM in VirtualBox" -ForegroundColor Gray
Write-Host "   Stop-VBox    - Power off the VirtualBox VM" -ForegroundColor Gray
Write-Host "   Run-QEMU     - Run in QEMU and execute trace analysis" -ForegroundColor Gray
Write-Host "   Show-Screen  - Capture and open live framebuffer screenshot" -ForegroundColor Gray
Write-Host ""
Write-Host "Press Ctrl+C or close this window when finished." -ForegroundColor DarkGray
Write-Host "-------------------------------------------------------------------------------" -ForegroundColor DarkGray
