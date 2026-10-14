#!/usr/bin/env bash
# Build native Win64 PE without Wine, Rust, Windows SDK or a C runtime.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
for bin in clang lld-link; do command -v "$bin" >/dev/null || { echo "Missing $bin" >&2; exit 1; }; done
WORK="$(mktemp -d)";trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/libs" "$ROOT/bin"
cat > "$WORK/libs/kernel32.def" <<'ENDDEF'
LIBRARY KERNEL32.dll
EXPORTS
 ExitProcess
 GetModuleHandleA
 GetStdHandle
 CreateFileA
 ReadFile
 WriteFile
 CloseHandle
ENDDEF
cat > "$WORK/libs/user32.def" <<'ENDDEF'
LIBRARY USER32.dll
EXPORTS
 RegisterClassExA
 CreateWindowExA
 DefWindowProcA
 ShowWindow
 LoadCursorA
 UpdateWindow
 GetMessageA
 TranslateMessage
 DispatchMessageA
 PostQuitMessage
 BeginPaint
 EndPaint
 GetClientRect
 InvalidateRect
 SetTimer
 KillTimer
 FillRect
 SetCapture
 ReleaseCapture
ENDDEF
cat > "$WORK/libs/gdi32.def" <<'ENDDEF'
LIBRARY GDI32.dll
EXPORTS
 CreateCompatibleDC
 CreateCompatibleBitmap
 SelectObject
 DeleteDC
 DeleteObject
 CreateSolidBrush
 CreatePen
 MoveToEx
 LineTo
 Ellipse
 RoundRect
 BitBlt
 SetTextColor
 SetBkMode
 TextOutA
 CreateFontA
ENDDEF
for dll in kernel32 user32 gdi32; do
  lld-link "/def:$WORK/libs/$dll.def" /machine:x64 /dll "/out:$WORK/libs/$dll.dll" "/implib:$WORK/libs/$dll.lib" >/dev/null
done
for program in aion_win32 windows_selftest; do
  clang --target=x86_64-pc-windows-msvc -O2 -ffreestanding -fno-stack-protector \
    -c "$ROOT/windows_viewer/$program.c" -o "$WORK/$program.obj"
done
clang --target=x86_64-pc-windows-msvc -O2 -ffreestanding -fno-stack-protector -c "$ROOT/native/aion_sim.c" -o "$WORK/aion_sim.obj"
clang --target=x86_64-pc-windows-msvc -O2 -ffreestanding -fno-stack-protector -c "$ROOT/native/checkpoint_portable.c" -o "$WORK/checkpoint_portable.obj"
clang --target=x86_64-pc-windows-msvc -O2 -ffreestanding -fno-stack-protector -c "$ROOT/native/experiment.c" -o "$WORK/experiment.obj"
lld-link "/out:$ROOT/bin/AION_CORE_0_9_C_REFERENCE.exe" /subsystem:windows /entry:mainCRTStartup \
  /machine:x64 /nodefaultlib /opt:ref "$WORK/aion_win32.obj" "$WORK/aion_sim.obj" "$WORK/checkpoint_portable.obj" "$WORK/experiment.obj" \
  "$WORK/libs/kernel32.lib" "$WORK/libs/user32.lib" "$WORK/libs/gdi32.lib"
lld-link "/out:$ROOT/bin/AION_CORE_0_9_C_REFERENCE_TEST.exe" /subsystem:console /entry:mainCRTStartup \
  /machine:x64 /nodefaultlib /opt:ref "$WORK/windows_selftest.obj" "$WORK/aion_sim.obj" "$WORK/checkpoint_portable.obj" "$WORK/experiment.obj" "$WORK/libs/kernel32.lib"
echo "Built: $ROOT/bin/AION_CORE_0_9_C_REFERENCE.exe"
echo "Built: $ROOT/bin/AION_CORE_0_9_C_REFERENCE_TEST.exe"
