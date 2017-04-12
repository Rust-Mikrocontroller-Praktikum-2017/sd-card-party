use super::block_device::BlockDevice;

use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct FileStorageDebug {
    file_buffer: Vec<u8>,
}

impl BlockDevice for FileStorageDebug {
    fn read_blocks(&self, offset: usize, number: usize) -> Vec<u8> {
        if self.file_buffer.len() >= (offset + number) * self.block_size() {
            let mut buf: Vec<u8> = Vec::with_capacity(number * self.block_size());
            for b in self.file_buffer[(offset * self.block_size())..
                ((offset + number) * self.block_size())]
                        .iter() {
                buf.push(*b);
            }
            buf
        } else {
            Vec::new()
        }
    }

    #[allow(unused_variables)]
    fn write_blocks(&self, offset: usize, blocks: &[u8]) -> Result<usize, ()> {
        unimplemented!();
    }

    fn number_of_blocks(&self) -> usize {
        self.file_buffer.len() / self.block_size()
    }

    fn block_size(&self) -> usize {
        512
    }
}

impl FileStorageDebug {
    pub fn new(path: &Path) -> FileStorageDebug {
        let mut file = match File::open(path) {
            Err(_) => panic!("file not present"),
            Ok(the_file) => the_file,
        };
        let mut buf = Vec::new();
        match file.read_to_end(&mut buf) {
            Err(_) => println!("no data"),
            Ok(number) => println!("{} bytes read", number),
        }
        FileStorageDebug { file_buffer: buf }
    }
}
