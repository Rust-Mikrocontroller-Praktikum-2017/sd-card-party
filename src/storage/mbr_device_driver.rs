use block_device::BlockDevice;
use super::partition::Partition;
use collections::vec::*;

const PARTITION_TABLE_OFFSET: usize = 0x01BE;

pub struct MbrDeviceDriver<'a> {
    first_partition: Partition<'a>,
}

impl<'a> MbrDeviceDriver<'a> {
    pub fn new(block_device: &'a BlockDevice) -> MbrDeviceDriver<'a> {
        if !(block_device.block_size() >= 512 && block_device.block_size() % 512 == 0) {
            panic!("512");
        }

        let mbr = block_device.read_blocks(0, 1);

        let mut first_entry: Vec<u8> = Vec::with_capacity(16);
        for i in 0..16 {
            first_entry.push(mbr[PARTITION_TABLE_OFFSET + i]);
        }
        let first_partition = Partition::new(block_device, &first_entry);

        MbrDeviceDriver { first_partition: first_partition }
    }

    pub fn get_first_partition(&self) -> &Partition {
        &self.first_partition
    }
}
