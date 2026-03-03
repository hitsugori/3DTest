@echo off
REM ─────────────────────────────────────────────────────────────────────────────
REM 3DTest — Web Build Script (Windows Batch Version)
REM Produces a `dist\` folder ready to upload to Cloudflare Pages.
REM
REM Requirements:
REM   cargo install wasm-pack
REM   rustup target add wasm32-unknown-unknown
REM
REM Usage:
REM   build-web.bat          REM normal build
REM   build-web.bat --dev    REM faster debug build (unoptimised wasm)
REM ─────────────────────────────────────────────────────────────────────────────

setlocal EnableDelayedExpansion

set DIST=dist
set PROFILE=release
set WASM_PACK_FLAGS=--release

REM Check for --dev flag
if "%1"=="--dev" (
    set PROFILE=dev
    set WASM_PACK_FLAGS=--dev
)

echo.
echo ==> Building 3DTest for web (profile: %PROFILE%)...
echo.

REM 1. Build with wasm-pack
wasm-pack build ^
  --target web ^
  --out-dir pkg ^
  --out-name threedtest ^
  --no-opt ^
  %WASM_PACK_FLAGS%

if errorlevel 1 (
    echo Build failed.
    exit /b 1
)

REM 2. Assemble dist\
if exist "%DIST%" rmdir /s /q "%DIST%"
mkdir "%DIST%"

copy index.html "%DIST%\" >nul
copy pkg\threedtest.js "%DIST%\" >nul
copy pkg\threedtest_bg.wasm "%DIST%\" >nul

REM Optional: copy .d.ts if it exists
if exist pkg\threedtest.d.ts (
    copy pkg\threedtest.d.ts "%DIST%\" >nul
)

echo.
echo Build complete! Output → %DIST%\
echo.

endlocal