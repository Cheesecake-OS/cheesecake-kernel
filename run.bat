@echo off
setlocal

set QEMU=qemu-system-x86_64
set BIN=target\x86_64-unknown-none\debug\bootimage-cheesecake_kernel.bin

cargo bootimage || exit /b 1

%QEMU% -drive format=raw,file=%BIN% -serial stdio -m 128M -no-reboot -no-shutdown -d int,cpu_reset -accel whpx,kernel-irqchip=off 2>nul || ^
%QEMU% -drive format=raw,file=%BIN% -serial stdio -m 128M -no-reboot -no-shutdown -d int,cpu_reset -accel tcg

endlocal