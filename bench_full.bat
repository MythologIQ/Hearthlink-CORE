@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat" x64
set LIBCLANG_PATH=C:\Program Files\llvm15.0.7\bin
set CMAKE_GENERATOR=Visual Studio 17 2022
set PATH=G:\MythologIQ\CORE\bin;%PATH%
cd /d G:\MythologIQ\CORE\core-runtime
cargo bench --features onnx,gguf
