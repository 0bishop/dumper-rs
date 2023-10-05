// #![no_std]
// #![no_main]

mod dumper;
use dumper::DumpMod;

use std::os::raw::c_void;

macro_rules! catch {
    ($result:expr, $message:expr) => {
        match $result {
            Ok(val) => val,
            Err(err) => {
                eprintln!("Error: {}: {:?}", $message, err);
                return;
            }
        }
    };
}

fn main() {
    // Load the DLL dynamically
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
