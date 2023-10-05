# lib-load
A little lib binding in order to hide imports functions from winapi.
0 Call to winapi, everything has been recoded from scratch

## Usage
*Refer to main.rs for more documentation about Error handling.*

dll:
```rs
#[no_mangle]
pub extern "C" fn hello_from_dll() {
    println!("Hello from Rust DLL!");
}
```

main:
```rs
mod loader;
use loader::LoadMod;

{
    let args = std::env::args().collect::<Vec<_>>();
    let dll_handle = catch!(DumpMod::new(args.get(1).expect("Error getting argv[1]"), true), "Failed to load DLL");

    dll_handle.disp();
    let addr = dll_handle.search_fn("WriteConsoleA").expect("Error searching WriteConsoleA").get_addr();
    let function: fn(*const c_void, *const c_void, u32, *const u32, *const c_void) -> u32  = unsafe { std::mem::transmute(addr) };

    let getstdhandle = dumper::call_function!(&dll_handle, fn(u32) -> *const c_void, "GetStdHandle");
    
    function(getstdhandle(-11i32 as u32) as *const c_void, "Hello World!\0".as_ptr() as *const c_void, 13, 0 as *const u32, 0 as *const c_void);

    let test = dll_handle.search_fn("WriteConsoleA").expect("Failed to get ptr");
    println!("\nRVA: 0x{:x}", test.get_rva());
    println!("{}", test);

    let exit = dumper::call_function!(&dll_handle, fn(i32) , "ExitProcess");
    exit(10);
}
```

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
