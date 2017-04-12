use super::get_bytes::*;
use collections::vec::*;
use collections::string::*;

// offset and number could be a tuple
const NAME_OFFSET: usize = 0; //11
const ATTRIBUTE_OFFSET: usize = 11; //1
const FIRST_CLUSTER_HIGH_OFFSET: usize = 20; //2
const FIRST_CLUSTER_LOW_OFFSET: usize = 26; //2
const FILE_SIZE_OFFSET: usize = 28; //4

//File [sic!] cant be a proper BlockDevice yet ->see BlockDevice comments
//  should be a handle, that knows the mbr driver (?)
/// just a simple container
/// can represent a file or a directory
pub struct DirectoryEntry {
    name_extension: String,
    is_file: bool,
    first_cluster_entry_number: usize,
    file_size: usize,
}

impl DirectoryEntry {
    pub fn new(directory_entry: &[u8]) -> DirectoryEntry {
        if directory_entry.len() != 32 {
            panic!("32");
        }

        let mut name_vec = Vec::with_capacity(8);
        for i in 0..8 {
            name_vec.push(directory_entry[i] as u16);
        }
        let name = String::from(String::from_utf16_lossy(&name_vec).trim()).to_lowercase();

        let mut extension_vec = Vec::with_capacity(3);
        for i in 8..11 {
            extension_vec.push(directory_entry[i] as u16);
        }
        let extension = String::from(String::from_utf16_lossy(&extension_vec).trim())
            .to_lowercase();

        let mut name_extension = String::with_capacity(11);
        name_extension.push_str(&name);
        if !extension.is_empty() {
            name_extension.push('.');
            name_extension.push_str(&extension);
        }

        let high = two_bytes_at_offset(&directory_entry, FIRST_CLUSTER_HIGH_OFFSET) as u32;
        let low = two_bytes_at_offset(&directory_entry, FIRST_CLUSTER_LOW_OFFSET) as u32;
        let first_cluster_entry_number = (high << 16 | low) as usize;

        let mut is_file = true;
        let attr = directory_entry[ATTRIBUTE_OFFSET];
        let no_name = directory_entry[NAME_OFFSET];

        let mut is_volume_id = false;
        let mut is_directory = false;

        if attr & 0x08 != 0 {
            is_volume_id = true;
        }

        if attr & 0x10 != 0 {
            is_directory = true;
        }

        if is_volume_id || is_directory {
            is_file = false;
        } else if no_name == 0xE5 || no_name == 0 {
            is_file = false;
        }

        let file_size = four_bytes_at_offset(&directory_entry, FILE_SIZE_OFFSET) as usize;

        DirectoryEntry {
            name_extension: name_extension,
            is_file: is_file,
            first_cluster_entry_number: first_cluster_entry_number,
            file_size: file_size,
        }
    }

    pub fn first_cluster(&self) -> usize {
        self.first_cluster_entry_number
    }

    pub fn is_file(&self) -> bool {
        self.is_file
    }

    pub fn file_size(&self) -> usize {
        self.file_size
    }

    pub fn name_extension(&self) -> &String {
        &self.name_extension
    }
}
