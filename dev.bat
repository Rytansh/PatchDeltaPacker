@echo off

if "%1"=="manifest" (
    cargo run --release --bin patch_packer -- --threads %2 manifest --root %3
    goto :eof
)

if "%1"=="patch" (
    cargo run --release --bin patch_packer -- --threads %2 build --old %3 --new %4 --output %5
    goto :eof
)

if "%1"=="launcher" (
    cargo run --release --bin launcher -- --threads %2 --game %3
    goto :eof
)

if "%1"=="server" (
    cargo run --release --bin patch_server -- --packages %2
    goto :eof
)

echo Unknown command.