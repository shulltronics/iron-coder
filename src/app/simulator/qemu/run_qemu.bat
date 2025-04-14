@echo off
set FIRMWARE_PATH="..\programs\RustLab\target\thumbv6m-none-eabi\release\RustLab"
set QEMU_PATH="C:\Program Files\qemu\qemu-system-arm.exe"

REM Run QEMU with Cortex-M0+ (RP2040-like) model
%QEMU_PATH% -M mps2-an385 -m 16M -nographic -kernel %FIRMWARE_PATH% -serial mon:stdio