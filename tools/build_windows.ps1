# Build freestanding AION Win64 GUI and Windows self-test on Windows 10/11.
# Needs clang and lld-link available on PATH (LLVM). No C runtime dependency.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$tmp = Join-Path $env:TEMP ("aion_winbuild_" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $root 'bin') | Out-Null
function Exec([string]$tool, [Parameter(ValueFromRemainingArguments = $true)][string[]]$argv) {
    & $tool @argv
    if ($LASTEXITCODE -ne 0) { throw "$tool failed with code $LASTEXITCODE" }
}
try {
    $defs = @{
       kernel32 = @('ExitProcess','GetModuleHandleA','GetStdHandle','CreateFileA','ReadFile','WriteFile','CloseHandle')
       user32 = @('RegisterClassExA','CreateWindowExA','DefWindowProcA','ShowWindow','LoadCursorA','UpdateWindow','GetMessageA','TranslateMessage','DispatchMessageA','PostQuitMessage','BeginPaint','EndPaint','GetClientRect','InvalidateRect','SetTimer','KillTimer','FillRect','SetCapture','ReleaseCapture')
       gdi32 = @('CreateCompatibleDC','CreateCompatibleBitmap','SelectObject','DeleteDC','DeleteObject','CreateSolidBrush','CreatePen','MoveToEx','LineTo','Ellipse','RoundRect','BitBlt','SetTextColor','SetBkMode','TextOutA','CreateFontA')
    }
    foreach ($dll in $defs.Keys) {
        $def = Join-Path $tmp "$dll.def"
        "LIBRARY $($dll.ToUpper()).dll`r`nEXPORTS`r`n" + (($defs[$dll] | ForEach-Object { " $_" }) -join "`r`n") | Set-Content -Encoding ascii -Path $def
        Exec 'lld-link' @("/def:$def", '/machine:x64', '/dll', "/out:$tmp\$dll.dll", "/implib:$tmp\$dll.lib")
    }
    foreach ($source in @('aion_win32','windows_selftest')) {
        Exec 'clang' @('--target=x86_64-pc-windows-msvc','-O2','-ffreestanding','-fno-stack-protector','-c', "$root\windows_viewer\$source.c",'-o',"$tmp\$source.obj")
    }
    Exec 'clang' @('--target=x86_64-pc-windows-msvc','-O2','-ffreestanding','-fno-stack-protector','-c',"$root\native\aion_sim.c",'-o',"$tmp\aion_sim.obj")
    Exec 'clang' @('--target=x86_64-pc-windows-msvc','-O2','-ffreestanding','-fno-stack-protector','-c',"$root\native\checkpoint_portable.c",'-o',"$tmp\checkpoint_portable.obj")
    Exec 'clang' @('--target=x86_64-pc-windows-msvc','-O2','-ffreestanding','-fno-stack-protector','-c',"$root\native\experiment.c",'-o',"$tmp\experiment.obj")
    Exec 'lld-link' @("/out:$root\bin\AION_CORE_0_8_C_REFERENCE.exe",'/subsystem:windows','/entry:mainCRTStartup','/machine:x64','/nodefaultlib','/opt:ref',"$tmp\aion_win32.obj","$tmp\aion_sim.obj","$tmp\checkpoint_portable.obj","$tmp\experiment.obj","$tmp\kernel32.lib","$tmp\user32.lib","$tmp\gdi32.lib")
    Exec 'lld-link' @("/out:$root\bin\AION_CORE_0_8_C_REFERENCE_TEST.exe",'/subsystem:console','/entry:mainCRTStartup','/machine:x64','/nodefaultlib','/opt:ref',"$tmp\windows_selftest.obj","$tmp\aion_sim.obj","$tmp\checkpoint_portable.obj","$tmp\experiment.obj","$tmp\kernel32.lib")
    Write-Host "EXE created in $root\bin"
}
finally { Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue }
