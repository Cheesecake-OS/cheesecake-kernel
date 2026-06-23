@echo off
setlocal

set QEMU=qemu-system-x86_64
set PROFILE=release

set BIN=target\x86_64-unknown-none\release\cheesecake_kernel.img

set OVMF_CODE=D:\qemu\share\edk2-x86_64-code.fd
set OVMF_VARS=D:\qemu\share\edk2-i386-vars.fd

cd cheesecake_kernel
cargo build --release --target x86_64-unknown-none || exit /b 1
cd ..

cargo run --release -p make_image || exit /b 1

%QEMU% ^
  -drive if=pflash,format=raw,readonly=on,file="%OVMF_CODE%" ^
  -drive if=pflash,format=raw,file="%OVMF_VARS%" ^
  -drive format=raw,file="%BIN%" ^
  -serial stdio -m 128M -no-reboot -no-shutdown ^
  -accel whpx,kernel-irqchip=off 2>nul || ^
%QEMU% ^
  -drive if=pflash,format=raw,readonly=on,file="%OVMF_CODE%" ^
  -drive if=pflash,format=raw,file="%OVMF_VARS%" ^
  -drive format=raw,file="%BIN%" ^
  -serial stdio -m 128M -no-reboot -no-shutdown ^
  -accel tcg

endlocal
