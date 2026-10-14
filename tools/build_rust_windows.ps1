# Must run from repository root with Rust + MSVC build tools, e.g. Windows CI.
# This is the proposed production link: it includes *no* aion_sim.c object.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
New-Item -ItemType Directory -Path bin -Force | Out-Null
& cargo build --release -p aion-rust-sim
if ($LASTEXITCODE -ne 0) { throw 'Rust engine compile failed' }
$rustLibrary = 'target\release\aion_rust_sim.lib'
if (-not (Test-Path $rustLibrary)) { throw "Missing $rustLibrary" }
# Clang accepts SSE2 inline assembly in the legacy CRT-free C visualizer.
& clang --target=x86_64-pc-windows-msvc -O2 -ffreestanding -fno-stack-protector `
    -c windows_viewer/aion_win32.c -o bin\viewer_05.obj
if ($LASTEXITCODE -ne 0) { throw 'Windows viewer compile failed' }
& clang --target=x86_64-pc-windows-msvc -O2 -ffreestanding -fno-stack-protector `
    -c windows_viewer/windows_selftest.c -o bin\test_05.obj
if ($LASTEXITCODE -ne 0) { throw 'Windows diagnostic compile failed' }
# Rust staticlib pulls in standard Rust runtime objects; MSVC linker resolves
# its Windows API and CRT dependencies against the installed Windows SDK.
& link /nologo /SUBSYSTEM:WINDOWS /ENTRY:mainCRTStartup /OUT:bin\AION_CORE_0_5_RUST.exe `
    bin\viewer_05.obj $rustLibrary user32.lib gdi32.lib kernel32.lib `
    advapi32.lib bcrypt.lib ntdll.lib ws2_32.lib userenv.lib
if ($LASTEXITCODE -ne 0) { throw 'Rust-linked Windows viewer link failed' }
& link /nologo /SUBSYSTEM:CONSOLE /ENTRY:mainCRTStartup /OUT:bin\AION_CORE_0_5_RUST_TEST.exe `
    bin\test_05.obj $rustLibrary kernel32.lib advapi32.lib bcrypt.lib ntdll.lib `
    ws2_32.lib userenv.lib
if ($LASTEXITCODE -ne 0) { throw 'Rust-linked diagnostic link failed' }
Write-Host 'Rust Windows release ready. Run both executables on Windows before publication.'
