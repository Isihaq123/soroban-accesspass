@echo off
:: ============================================================
:: run-tests.bat
:: Sets up the MSVC + Windows SDK environment and runs all
:: Soroban AccessPass unit tests.
:: Run from the repo root: scripts\run-tests.bat
:: ============================================================

set MSVC_VER=14.44.35207
set VS_BASE=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools
set SDK_VER=10.0.22621.0

set PATH=%VS_BASE%\VC\Tools\MSVC\%MSVC_VER%\bin\Hostx86\x64;%PATH%
set LIB=%VS_BASE%\VC\Tools\MSVC\%MSVC_VER%\lib\x64;^
C:\Program Files (x86)\Windows Kits\10\Lib\%SDK_VER%\um\x64;^
C:\Program Files (x86)\Windows Kits\10\Lib\%SDK_VER%\ucrt\x64

set INCLUDE=%VS_BASE%\VC\Tools\MSVC\%MSVC_VER%\include;^
C:\Program Files (x86)\Windows Kits\10\Include\%SDK_VER%\ucrt;^
C:\Program Files (x86)\Windows Kits\10\Include\%SDK_VER%\um;^
C:\Program Files (x86)\Windows Kits\10\Include\%SDK_VER%\shared

echo [run-tests] Environment configured. Running cargo test...
cargo test
