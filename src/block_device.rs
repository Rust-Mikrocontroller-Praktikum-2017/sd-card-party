use collections::vec::*;

pub trait BlockDevice {
    /// note: <---> means in a better version
    /// if offset * block_size() is larger than the last address, get_data(...)
    /// shall return an empty vector
    /// if number == 0, get_data(...) shall return an empty vector
    /// if (offset + number) * block_size() is larger than the last address,
    ///     get_data(...) shall return an vector with all blocks till the end
    ///     ---"...all blocks..." -> ...all data...
    /// read_blocks(...).len() % block_size() == 0 must always be true
    ///     ---should be called "read"
    ///     ---"...must..." -> ...must not...
    fn read_blocks(&self, offset: usize, number: usize) -> Vec<u8>;
    /// unimplemented!()
    /// ---should be called "write"
    fn write_blocks(&self, offset: usize, blocks: &[u8]) -> Result<usize, ()>;
    /// ---should be size instead
    fn number_of_blocks(&self) -> usize;
    /// block_size should be 512 byte
    fn block_size(&self) -> usize;
    // ---better: additional method len() an last "block" contains filling values at the end
    //  three "/" above ->compiler error??!
    //fn len(&self) ->usize;
}
