@echo off
setlocal
set PROFILE=release

cd cheesecake_kernel
cargo build --release --target x86_64-unknown-none || exit /b 1
cd ..

cargo run --release -p make_image || exit /b 1

echo.
echo Image generated:
echo target\x86_64-unknown-none\release\cheesecake_kernel.img

endlocal
