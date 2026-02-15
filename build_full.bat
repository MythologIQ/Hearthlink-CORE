@echo off
call "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvarsall.bat" x64
set LIBCLANG_PATH=G:\MythologIQ\CORE\tools\llvm16\bin
set CMAKE_GENERATOR=Ninja
set PATH=G:\MythologIQ\CORE\bin;%PATH%
cd /d G:\MythologIQ\CORE\core-runtime
cargo build --release --features onnx,gguf
