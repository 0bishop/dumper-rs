# dumper-rs
A little lib binding in order to hide imports functions from winapi.
0 Call to winapi, everything has been recoded from scratch

## Usage
*Refer to main.rs for more documentation about Error handling.*

output
```bash
lib-load> cargo run --release -- kernel32.dll
...
0x7ffba1126fa0 1538            0x36fa0           0x685             uaw_wcslen
0x7ffba1126fd0 1729            0x36fd0           0x686             uaw_wcsrchr
Module name: kernel32.dll base address: 0x7ffba10f0000

Hello World!
RVA: 0x20d00
0x7ffba1110d00 3276            0x20d00           0x640             WriteConsoleA
error: process didn't exit successfully: `target\release\dumper.exe kernel32.dll` (exit code: 10)
```
