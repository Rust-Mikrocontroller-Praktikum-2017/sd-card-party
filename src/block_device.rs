pub trait BlockDevice {
    /// if offset * block_size() is larger than the last address, get_data(...) shall return an empty vector
    /// if number == 0, get_data(...) shall return an empty vector
    /// if (offset + number) * block_size() is larger than the last address,
    ///     get_data(...) shall return an vector with all blocks till the end
    /// read_blocks(...).len() % block_size() == 0 must always be true
    fn read_blocks(&self, offset: usize, number: usize) ->Vec<u8>;
    //unimplemented!()
    fn write_blocks(&self, offset: usize, blocks: Vec<u8>) -> Result<usize, ()>;
    fn number_of_blocks(&self) -> usize;
    /// block_size should be 512 byte
    fn block_size(&self) -> usize;
}
