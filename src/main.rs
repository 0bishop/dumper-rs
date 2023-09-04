mod loader;
use loader::LoadMod;

use std::os::raw::c_int;

fn main() {
    
    // Load the DLL dynamically
    let dll_handle = LoadMod::new("my_dll/target/release/my_dll.dll");
    if dll_handle.is_err() {
        println!("Failed to load DLL: {}", dll_handle.err().unwrap());
        return;
    }
    let dll_handle = dll_handle.unwrap();
    // Load the function pointer dynamically
    // Convert the function pointer to the defined function type
    let result = loader::call_function!(&dll_handle, fn() -> i32, "hello_from_dll");

    let out = result();
    // Now, you can call the hello_from_dll function
    println!("Function result: {}", out);

    let add_function = loader::call_function!(&dll_handle, fn(c_int, c_int) -> c_int, "add");
    let result = add_function(1, 2);
    println!("Function result: {}", result);

    // The DLL will be freed when the Handle goes out of scope
}
