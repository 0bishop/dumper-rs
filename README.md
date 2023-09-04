# lib-load
A little lib binding around LoadLibraryA and GetProcAddress from winapi crate in order to hide imports functions from winapi

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
    let dll = LoadMod::new("my_dll/target/release/my_dll.dll").unwrap();

    // Use this proc macro and all the work around will be handled
    let result = call_function!(&dll_handle, fn() -> i32, "hello_from_dll!");

    result();
    // Memory will be automatically freed
}
```

output
```bash
lib-load> cargo run --release
    Finished release [optimized] target(s) in 0.00s
     Running `target\release\loader.exe`

Hello from Rust DLL!
```
