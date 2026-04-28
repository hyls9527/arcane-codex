Start-Sleep -Seconds 2

# 移动鼠标到 Settings 按钮位置 (根据截图估算)
$pos = New-Object System.Drawing.Point(1170, 235)
[System.Windows.Forms.Cursor]::Position = $pos

# 点击
Add-Type -MemberDefinition @"
[DllImport("user32.dll")]
public static extern void mouse_event(int flags, int dx, int dy, int data, IntPtr extraInfo);
"@ -Name Mouse -Namespace Win32

[Win32.Mouse]::mouse_event(0x00000002, 0, 0, 0, 0)  # 按下
Start-Sleep -Milliseconds 50
[Win32.Mouse]::mouse_event(0x00000004, 0, 0, 0, 0)  # 释放

Write-Host "Clicked Settings"
