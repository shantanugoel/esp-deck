## To see backtraces
```
xtensa-esp32s3-elf-addr2line.exe -e .\target\xtensa-esp32s3-espidf\release\esp-deck -a -f -p <list of addresses from backtrace>
```

## To run
```
cargo espflash flash --release --baud 1500000 --flash-size 16mb
```

## To monitor
`no-stub` fixes a bug where espflash takes over/hangs the terminal window
```
espflash monitor --no-stub
```