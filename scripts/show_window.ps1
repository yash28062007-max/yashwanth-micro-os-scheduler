Add-Type -TypeDefinition @"
using System;
using System.Text;
using System.Runtime.InteropServices;

public class WinAPI {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
    [DllImport("user32.dll")]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);

    public static void RestoreWindows(string name) {
        EnumWindows((hWnd, lParam) => {
            uint pid;
            GetWindowThreadProcessId(hWnd, out pid);
            var sb = new StringBuilder(512);
            GetWindowText(hWnd, sb, 512);
            string title = sb.ToString();
            try {
                var proc = System.Diagnostics.Process.GetProcessById((int)pid);
                if (proc.ProcessName.IndexOf(name, StringComparison.OrdinalIgnoreCase) >= 0 || title.IndexOf(name, StringComparison.OrdinalIgnoreCase) >= 0) {
                    Console.WriteLine("Target Window Found: PID=" + pid + " HWND=" + hWnd + " Title='" + title + "'");
                    ShowWindow(hWnd, 9); // SW_RESTORE
                    SetForegroundWindow(hWnd);
                }
            } catch {}
            return true;
        }, IntPtr.Zero);
    }
}
"@

[WinAPI]::RestoreWindows("VirtualBox")
[WinAPI]::RestoreWindows("Yashwanth")
