use windows_sys::Win32::System::{
    Diagnostics::Debug::IMAGE_NT_HEADERS64,
    SystemServices::{
        IMAGE_DOS_HEADER,
        IMAGE_EXPORT_DIRECTORY
    },
    Threading::PEB,
    WindowsProgramming::LDR_DATA_TABLE_ENTRY,
    Kernel::LIST_ENTRY
};

use std::ffi::CStr;
use std::fmt;
use std::os::raw::{c_char, c_void};

#[derive(Debug, Clone)]
pub struct Function {
    name : String,
    addr : *const c_void,
    number: u32,
    ord : usize,
    rva: usize,
}

#[derive(Debug, Clone)]
pub struct DumpMod {
    handle : *const c_void,
    name : String,
    exports : Vec<Function>,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "0x{:x} {:<15} 0x{:<15x} 0x{:<15x} {}",
            self.addr as usize, self.ord, self.rva, self.number, self.name
        )
    }
}

impl fmt::Display for DumpMod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:<14} {:<15} {:<15} {:<15} {}",
            "addr", "ordinal", "RVA", "number", "name"
        )?;
        for functions in &self.exports {
            writeln!(f, "{}", functions)?;
        }
        writeln!(f, "Module name: {} base address: 0x{:x}", self.name, self.handle as usize)
    }
}

pub trait SearchKey {
    fn matches(&self, function: &Function) -> bool;
}

impl SearchKey for &str {
    fn matches(&self, function: &Function) -> bool {
        self == &function.name
    }
}

impl SearchKey for usize {
    fn matches(&self, function: &Function) -> bool {
        *self == function.ord
    }
}

impl Function {

    #[allow(dead_code)]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_addr(&self) -> *const c_void {
        self.addr
    }

    pub fn get_number(&self) -> u32 {
        self.number
    }

    pub fn get_ord(&self) -> usize {
        self.ord
    }

    pub fn get_rva(&self) -> usize {
        self.rva
    }
}

impl DumpMod {
    #[cfg(target_arch = "x86")]
    const PEB_OFFSET: i32 = 0x30;

    #[cfg(target_arch = "x86_64")]
    const PEB_OFFSET: i32 = 0x60;

    /* The only_dump_syscalls parameter is used to only dump the syscalls from ntdll.dll */
    pub fn new(path : &str, only_dump_ntdll_syscalls : bool) -> Result<Self, String> {
        let handle = match Self::get_handle_by_peb(path) {
            Ok(handle) => handle,
            Err(_) => return Err("Failed to get handle".to_string()),
        };
        let name = path.to_string();
        let exports = Self::dump(&name, handle, only_dump_ntdll_syscalls)?;
        Ok(DumpMod {
            handle,
            name,
            exports,
        })
    }

    unsafe fn get_peb(offset: i32) -> Result<u64, String> {
        let out: u64;
        if offset == 0x60 {
            std::arch::asm!(
                "mov {}, gs:[{:e}]",
                lateout(reg) out,
                in(reg) offset,
                options(nostack, pure, readonly)
            );
        } else if offset == 0x30 {
            std::arch::asm!(
                "mov {}, fs:[{:e}]",
                lateout(reg) out,
                in(reg) offset,
                options(nostack, pure, readonly),
            );
        } else {
            return Err("Invalid offset".to_string());
        }
        Ok(out)
    }

    // this repos helped me: https://github.com/trickster0/OffensiveRust
    fn get_handle_by_peb(module_name: &str) -> Result<*const c_void, String> {
        unsafe {
            let peb = *(match Self::get_peb(Self::PEB_OFFSET) {
                Ok(offset) => offset as u64,
                Err(err) => return Err(format!("Error: {}", err)),
            } as *const PEB);

            let mut p_ldr_data_table_entry: *const LDR_DATA_TABLE_ENTRY = (*peb.Ldr).InMemoryOrderModuleList.Flink as *const LDR_DATA_TABLE_ENTRY;
            let mut p_list_entry = &(*peb.Ldr).InMemoryOrderModuleList as *const LIST_ENTRY;

            loop {
                if String::from_utf16_lossy(
                    std::slice::from_raw_parts(
                        (*p_ldr_data_table_entry).FullDllName.Buffer,
                        (*p_ldr_data_table_entry).FullDllName.Length as usize / 2))
                        .to_lowercase()
                        .starts_with(module_name) {
                    return Ok((*p_ldr_data_table_entry).Reserved2[0] as *const c_void);
                }
                if p_list_entry == (*peb.Ldr).InMemoryOrderModuleList.Blink {
                    return Err("Module not found!".to_string());
                }
                p_list_entry = (*p_list_entry).Flink;
                p_ldr_data_table_entry = (*p_list_entry).Flink as *const LDR_DATA_TABLE_ENTRY;
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_handle(&self) -> *const c_void {
        self.handle
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_exports(&self) -> &Vec<Function> {
        &self.exports
    }

    fn is_syscall(fptr: *const std::ffi::c_void) -> bool {
        let aligned_ptr = fptr as *const u32;
    
        if (aligned_ptr as usize) % std::mem::align_of::<u32>() == 0 {
            return unsafe { *aligned_ptr } == 0xB8D18B4C;
        }
        false
    }
    
    fn dump(path: &str, handle: *const c_void, only_dump_ntdll_syscalls: bool) -> Result<Vec<Function>, String> {
        let isntdll = if path.contains("ntdll") && only_dump_ntdll_syscalls {true} else {false};
        let mod_base = handle as *const u8;

        let dos_header: &IMAGE_DOS_HEADER = unsafe { &*(mod_base as *const IMAGE_DOS_HEADER) };
        if dos_header.e_magic.ne(&0x5A4D) {
            return Err(191.to_string());
        }
    
        let nt_header: &IMAGE_NT_HEADERS64 = unsafe { &*(mod_base.wrapping_add(dos_header.e_lfanew as usize) as *const IMAGE_NT_HEADERS64) };
        if nt_header.Signature.ne(&0x00004550) {
            return Err(191.to_string());
        }
    
        let export_dir: &IMAGE_EXPORT_DIRECTORY = unsafe { &*(mod_base.wrapping_add(nt_header.OptionalHeader.DataDirectory[0].VirtualAddress as usize) as *const IMAGE_EXPORT_DIRECTORY) };
        if export_dir.NumberOfFunctions == 0 {
            return Err(1154.to_string());
        }
    
        let address_of_func = mod_base.wrapping_add(export_dir.AddressOfFunctions as usize) as *const u32;
        let address_of_name = mod_base.wrapping_add(export_dir.AddressOfNames as usize) as *const u32;
        let address_of_ord = mod_base.wrapping_add(export_dir.AddressOfNameOrdinals as usize) as *const u16;
        let mut exports = Vec::with_capacity(export_dir.NumberOfFunctions as usize);
    
        for i in 0..export_dir.NumberOfFunctions {
            let func_name_ptr = mod_base.wrapping_add(unsafe { *address_of_name.offset(i as isize) } as usize) as *const c_char;
            let func_name_cstr = unsafe { CStr::from_ptr(func_name_ptr) };
            let func_name_str = func_name_cstr.to_str().unwrap_or("");
    
            let func_ptr = mod_base.wrapping_add(unsafe { *address_of_func.offset(*address_of_ord.offset(i as isize) as isize) } as usize) as *const c_void;
    
            if isntdll && Self::is_syscall(func_ptr) {
                let dumped_syscall = Function {
                    name: func_name_str.to_string(),
                    addr: func_ptr,
                    number: i,
                    rva: (func_ptr as usize - nt_header.OptionalHeader.ImageBase as usize) as usize,
                    ord: (unsafe { *(func_ptr as *const usize) } >> 8 * 4) & 0xfff,
                };
    
                exports.push(dumped_syscall);
            } else if !isntdll {
                let dumped_syscall = Function {
                    name: func_name_str.to_string(),
                    addr: func_ptr,
                    number: i,
                    rva: (func_ptr as usize - nt_header.OptionalHeader.ImageBase as usize) as usize,
                    ord: (unsafe { *((((func_ptr as usize >> 3) << 3) as *const usize).offset(1)) } >> 8 * 4) & 0xfff,
                };
    
                exports.push(dumped_syscall);
            }
        }
        Ok(exports)
    }

    pub fn disp(&self) {
        println!("{}", self)
    }

    pub fn search_fn<T: SearchKey + Clone>(&self, key: T) -> Option<Function> {
        self.exports.iter().find(|function| key.matches(function)).cloned()
    }
    
}

// macro to call the function, and convert the function pointer to the defined function type
macro_rules! call_function {
    ($dll_handle:expr, $type:ty, $name:expr) => {
        {
            // Change crate::loader to your mod name
            let function: $type =  unsafe { std::mem::transmute(DumpMod::search_fn($dll_handle, $name).expect("Failed to get ptr").get_addr()) };
            function
        }
    };
}

pub(crate) use call_function;
