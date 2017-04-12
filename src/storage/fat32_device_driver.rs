use block_device::BlockDevice;
use super::directory_entry::DirectoryEntry;
use super::get_bytes::*;
use collections::vec::*;
use core::option::*;

//number of used bytes?!!
const BYTE_PER_SECTOR_OFFSET: usize = 0x0B;
const SECTORS_PER_CLUSTER_OFFSET: usize = 0x0D;
const NUMBER_OF_RESERVED_SECTORS_OFFSET: usize = 0x0E;
const NUMBER_OF_FATS_OFFSET: usize = 0x010;
const NUMBER_OF_SECTORS_PER_FAT_OFFSET: usize = 0x024;
const CLUSTER_NUMBER_ROOT_DIRECTORY_OFFSET: usize = 0x02C;

/*
dbg:
in the file:
2048 :partition
(2048 + 32) * 512 = 1064960 :FAT (table-entries)
(2048 + 4022) * 512 = 3107840 :first cluster (data)
*/

pub struct Fat32DeviceDriver<'a> {
    block_device: &'a BlockDevice,
    block_size_cluster: usize,
    number_of_reserved_blocks: usize,
    data_region_block_offset: usize,
    root_directory_cluster_offset: usize,
}

impl<'a> Fat32DeviceDriver<'a> {
    /// Partition::get_partition_type() == 0x0B has to be checked before
    pub fn new(block_device: &'a BlockDevice) -> Fat32DeviceDriver<'a> {
        if !(block_device.block_size() >= 512 && block_device.block_size() % 512 == 0) {
            panic!("wrong block_size");
        }
        let block = block_device.read_blocks(0, 1);

        let byte_per_sector = two_bytes_at_offset(&block, BYTE_PER_SECTOR_OFFSET) as usize;
        let sectors_per_cluster = block[SECTORS_PER_CLUSTER_OFFSET] as usize;
        let number_of_reserved_sectors =
            two_bytes_at_offset(&block, NUMBER_OF_RESERVED_SECTORS_OFFSET) as usize;
        let cluster_number_root_directory =
            four_bytes_at_offset(&block, CLUSTER_NUMBER_ROOT_DIRECTORY_OFFSET) as usize;

        if !(byte_per_sector % 512 == 0) {
            panic!("wrong byte_per_sector");
        }
        let block_size_sector = byte_per_sector / block_device.block_size();
        let block_size_cluster = sectors_per_cluster * block_size_sector;
        let number_of_reserved_blocks = number_of_reserved_sectors / block_size_sector;
        let number_of_fats = block[NUMBER_OF_FATS_OFFSET] as usize;
        let number_of_sectors_per_fat =
            four_bytes_at_offset(&block, NUMBER_OF_SECTORS_PER_FAT_OFFSET) as usize;
        let data_region_block_offset = number_of_fats * number_of_sectors_per_fat *
                                       block_size_sector +
                                       number_of_reserved_blocks;
        let root_directory_cluster_offset = cluster_number_root_directory;

        Fat32DeviceDriver {
            block_device: block_device,
            block_size_cluster: block_size_cluster,
            number_of_reserved_blocks: number_of_reserved_blocks,
            data_region_block_offset: data_region_block_offset,
            root_directory_cluster_offset: root_directory_cluster_offset,
        }
    }

    /// only short name
    /// only in root directory
    // sdram
    pub fn read_file_to_vec(&self, name_extension: &str) -> Option<Vec<u8>> {
        let opt_file = self.file_directory_entry(name_extension);
        let file = match opt_file {
            Some(f) => f,
            None => return None,
        };
        let mut full = self.compile_clusters_begin_with_number(file.first_cluster());
        full.split_off(file.file_size());
        Some(full)
    }

    // sdram
    fn compile_clusters_begin_with_number(&self, offset: usize) -> Vec<u8> {
        let mut all = Vec::new();
        let mut current_offset = offset;
        //[0x?0000002; 0x?FFFFFF6] //max should be calculated first
        while (current_offset & 0x0FFFFFFF) >= 0x2 && (current_offset & 0x0FFFFFFF) <= 0xFFFFFF6 {
            //println!("current_offset: {0:08.x}", current_offset);
            all.append(&mut self.read_cluster_data_region(current_offset));
            current_offset = self.read_in_fat(current_offset);
        }
        all
    }

    fn file_directory_entry(&self, name_extension: &str) -> Option<DirectoryEntry> {
        let root = self.read_root_directory();
        let number: usize = root.len() / 32;
        // better: implement check is_last_entry and break
        for i in 0..number {
            let mut directory_entry = Vec::with_capacity(32);
            for j in 0..32 {
                directory_entry.push(root[i * 32 + j]);
            }
            let dir_entr = DirectoryEntry::new(&directory_entry);
            if dir_entr.is_file() && dir_entr.name_extension() == name_extension {
                return Some(dir_entr);
            }
        }
        None
    }

    // sdram
    fn read_root_directory(&self) -> Vec<u8> {
        self.read_cluster_data_region(self.root_directory_cluster_offset)
    }

    /// will crash if (offset * 4) >= block_size
    // buffer
    fn read_in_fat(&self, offset: usize) -> usize {
        //better: buffer for read block_device with offset
        let block = self.block_device
            .read_blocks(self.number_of_reserved_blocks, 1);
        //4: byte-size of u32
        (four_bytes_at_offset(&block, offset * 4) & 0x0FFFFFFF) as usize
    }

    // sdram
    fn read_cluster_data_region(&self, cluster_entry_offset: usize) -> Vec<u8> {
        //- 2 because the first two cluster-entries in the FAT are reserved
        //and dont represent clusters in the data section
        self.block_device
            .read_blocks(self.data_region_block_offset +
                         (cluster_entry_offset - 2) * self.block_size_cluster,
                         self.block_size_cluster)
    }
}
