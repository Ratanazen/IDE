use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;

pub const MH_MAGIC_64: u32 = 0xfeedfacf;
pub const MH_CIGAM_64: u32 = 0xcffaedfe;
pub const CPU_TYPE_ARM64: u32 = 0x0100000c;
pub const CPU_TYPE_X86_64: u32 = 0x01000007;

pub const LC_SEGMENT_64: u32 = 0x19;
pub const LC_LOAD_DYLIB: u32 = 0x0c;
pub const LC_LOAD_WEAK_DYLIB: u32 = 0x18 | 0x80000000;
pub const LC_CODE_SIGNATURE: u32 = 0x1d;
pub const LC_BUILD_VERSION: u32 = 0x32;

#[derive(Error, Debug)]
pub enum MachOError {
    #[error("File is too small to contain a Mach-O header (< 32 bytes)")]
    TooSmall,
    #[error("Not a valid 64-bit Mach-O binary (magic: {0:#x})")]
    InvalidMagic(u32),
    #[error("I/O error reading Mach-O binary: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct MachOInfo {
    pub magic: u32,
    pub is_64bit: bool,
    pub is_arm64: bool,
    pub is_x86_64: bool,
    pub num_commands: u32,
    pub size_of_commands: u32,
    pub has_code_signature_command: bool,
    pub has_build_version_command: bool,
    pub load_commands: Vec<u32>,
}

impl MachOInfo {
    pub fn parse_from_bytes(bytes: &[u8]) -> Result<Self, MachOError> {
        if bytes.len() < 32 {
            return Err(MachOError::TooSmall);
        }

        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let is_little_endian = magic == MH_MAGIC_64;
        let is_big_endian = magic == MH_CIGAM_64;

        if !is_little_endian && !is_big_endian {
            return Err(MachOError::InvalidMagic(magic));
        }

        let read_u32 = |offset: usize| -> u32 {
            let slice = [bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]];
            if is_little_endian {
                u32::from_le_bytes(slice)
            } else {
                u32::from_be_bytes(slice)
            }
        };

        let cputype = read_u32(4);
        let num_commands = read_u32(16);
        let size_of_commands = read_u32(20);

        let is_arm64 = cputype == CPU_TYPE_ARM64;
        let is_x86_64 = cputype == CPU_TYPE_X86_64;

        let mut offset = 32;
        let mut load_commands = Vec::new();
        let mut has_code_signature_command = false;
        let mut has_build_version_command = false;

        for _ in 0..num_commands {
            if offset + 8 > bytes.len() {
                break;
            }
            let cmd = read_u32(offset);
            let cmdsize = read_u32(offset + 4) as usize;

            load_commands.push(cmd);

            if cmd == LC_CODE_SIGNATURE {
                has_code_signature_command = true;
            } else if cmd == LC_BUILD_VERSION {
                has_build_version_command = true;
            }

            if cmdsize == 0 || offset + cmdsize > bytes.len() {
                break;
            }
            offset += cmdsize;
        }

        Ok(Self {
            magic,
            is_64bit: true,
            is_arm64,
            is_x86_64,
            num_commands,
            size_of_commands,
            has_code_signature_command,
            has_build_version_command,
            load_commands,
        })
    }

    pub fn parse_from_file<P: AsRef<Path>>(path: P) -> Result<Self, MachOError> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Self::parse_from_bytes(&buffer)
    }
}
