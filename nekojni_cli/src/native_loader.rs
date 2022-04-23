use dlopen::raw::Library;
use goblin::{
    elf::header::*,
    mach::{constants::cputype::*, header::*, Mach},
    pe::{characteristic::*, header::*},
    Object,
};
use nekojni::{__macro_internals::*, *};
use std::path::PathBuf;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum EntryPointPlatform {
    Windows,
    Macos,
    Linux,
}
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum EntryPointArch {
    X86,
    X86_64,
    AArch64,
}

#[derive(Debug)]
pub struct LoadedBinary {
    pub info: Vec<&'static JavaModuleInfo>,
}

#[derive(Clone, Debug)]
pub struct ParsedBinary {
    pub platform: EntryPointPlatform,
    pub arch: EntryPointArch,
    entry_points: Vec<String>,
    pub path: PathBuf,
}
impl ParsedBinary {
    pub fn parse(path: PathBuf) -> Result<ParsedBinary> {
        let data = std::fs::read(&path)?;
        nekojni_parse_binary(path, &data)
    }

    pub fn can_load(&self) -> bool {
        let plaf_match = match self.platform {
            EntryPointPlatform::Windows => std::env::consts::OS == "windows",
            EntryPointPlatform::Macos => std::env::consts::OS == "macos",
            EntryPointPlatform::Linux => std::env::consts::OS == "linux",
        };
        let arch_match = match self.arch {
            EntryPointArch::X86 => std::env::consts::ARCH == "x86",
            EntryPointArch::X86_64 => std::env::consts::ARCH == "x86_64",
            EntryPointArch::AArch64 => std::env::consts::ARCH == "aarch64",
        };
        plaf_match && arch_match
    }
    pub fn load(&self) -> Result<Option<LoadedBinary>> {
        if !self.can_load() {
            Ok(None)
        } else {
            let lib = Library::open(&self.path)?;
            let mut modules = Vec::new();
            unsafe {
                for entry_point in &self.entry_points {
                    let func: extern "C" fn() -> &'static JavaModuleInfo =
                        lib.symbol(&entry_point)?;
                    let info = func();

                    jni_assert!(
                        info.magic == MAGIC_NUMBER,
                        "Native library has a bad magic number. Something is very wrong.",
                    );
                    jni_assert!(
                        info.major_version == MAJOR_VERSION,
                        "Native library is not compatible: Wrong major version 0x{:08x}.",
                        info.major_version,
                    );
                    jni_assert!(
                        info.marker_len == MARKER_STR.len(),
                        "Native library is not compatible: Wrong marker string length."
                    );
                    jni_assert!(
                        info.get_marker_ptr() == MARKER_STR,
                        "Native library is not compatible: Wrong marker string."
                    );

                    modules.push(info);
                }
            }
            std::mem::forget(lib); // only way to make this truly safe...
            Ok(Some(LoadedBinary { info: modules }))
        }
    }
}

fn nekojni_parse_binary(path: PathBuf, so_data: &[u8]) -> Result<ParsedBinary> {
    let object = goblin::Object::parse(so_data)?;
    match object {
        Object::Elf(elf) => {
            jni_assert!(elf.header.e_type == ET_DYN, "ELF binary must be a dynamic library.");
            let arch = match elf.header.e_machine {
                EM_386 => EntryPointArch::X86,
                EM_X86_64 => EntryPointArch::X86_64,
                EM_AARCH64 => EntryPointArch::AArch64,
                _ => jni_bail!("ELF binary has unsupported machine architecture."),
            };

            let mut entry_points = Vec::new();
            for sym in elf.dynsyms.to_vec() {
                if let Some(name) = elf.dynstrtab.get_at(sym.st_name) {
                    if name.starts_with("__njni_modinfo_v1__") {
                        entry_points.push(name.to_string());
                    }
                }
            }

            Ok(ParsedBinary { platform: EntryPointPlatform::Linux, arch, entry_points, path })
        }
        Object::PE(pe) => {
            jni_assert!(
                is_dll(pe.header.coff_header.characteristics),
                "PE binary must be a dynamic library.",
            );
            let arch = match pe.header.coff_header.machine {
                COFF_MACHINE_X86 => EntryPointArch::X86,
                COFF_MACHINE_X86_64 => EntryPointArch::X86_64,
                COFF_MACHINE_ARM64 => EntryPointArch::AArch64,
                _ => jni_bail!("PE binary has unsupported machine architecture."),
            };

            let mut entry_points = Vec::new();
            for export in pe.exports {
                if let Some(name) = export.name {
                    if name.starts_with("__njni_modinfo_v1__") {
                        entry_points.push(name.to_string());
                    }
                }
            }

            Ok(ParsedBinary { platform: EntryPointPlatform::Windows, arch, entry_points, path })
        }
        Object::Mach(mach) => match mach {
            Mach::Fat(_) => jni_bail!("Mach-O fat binaries are not supported by nekojni."),
            Mach::Binary(mach) => {
                jni_assert!(
                    (mach.header.flags & MH_DYLDLINK) != 0,
                    "Mach-O binary must be a dynamic library.",
                );
                let arch = match mach.header.cputype {
                    CPU_TYPE_X86 => EntryPointArch::X86,
                    CPU_TYPE_X86_64 => EntryPointArch::X86_64,
                    CPU_TYPE_ARM64 => EntryPointArch::AArch64,
                    _ => jni_bail!("Mach-O binary has unsupported machine architecture."),
                };

                let mut entry_points = Vec::new();
                if let Some(symbols) = mach.symbols {
                    for symbol in symbols.iter() {
                        let symbol = symbol?;
                        let name = symbol.0;
                        if name.starts_with("__njni_modinfo_v1__") {
                            entry_points.push(name.to_string());
                        }
                    }
                }

                Ok(ParsedBinary { platform: EntryPointPlatform::Macos, arch, entry_points, path })
            }
        },
        Object::Archive(_) => jni_bail!("Archives are not supported by nekojni."),
        Object::Unknown(magic) => {
            jni_bail!("Unknown dynamic library format with magic number 0x{magic:016x}")
        }
    }
}
