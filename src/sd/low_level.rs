// represents a group of defines in stm32f7xx_ll_sdmmc.h, e.g. SDMMC_ERROR_NONE
// TODO: copy descriptions from the c-code
bitflags! {
    #[allow(non_upper_case_globals)]
    pub flags SdmmcErrorCode: u32 {
        const NONE = 0x00,
        const CMD_CRC_FAIL = 0x01,
        const DATA_CRC_FAIL = 0x02,
        const CMD_RSP_TIMEOUT = 0x04,
        const DATA_TIMEOUT = 0x08,
        const TX_UNDERRUN = 0x10,
        const RX_OVERRUN = 0x20,
        const ADDR_MISALIGNED = 0x40,
        const BLOCK_LEN_ERR = 0x80,

        const ERASE_SEQ_ERR = 0x100,
        const BAD_ERASE_PARAM = 0x200,
        const WRITE_PROT_VIOLATION = 0x400,
        const LOCK_UNLOCK_FAILED = 0x800,

        const COM_CRC_FAILED = 0x001000,
        const ILLEGAL_CMD = 0x002000,
        const CARD_ECC_FAILED = 0x004000,
        const CC_ERR = 0x008000,
        const GENERAL_UNKNOWN_ERR = 0x010000,
        const STREAM_READ_UNDERRUN = 0x020000,
        const STREAM_WRITE_OVERRUN = 0x040000,
        const CID_CSD_OVERWRITE = 0x080000,
        const WP_ERASE_SKIP = 0x100000,
        const CARD_ECC_DISABLED = 0x200000,
        const ERASE_RESET = 0x400000,

        const AKE_SEQ_ERR = 0x00800000,
        const INVALID_VOLTRANGE = 0x01000000,
        const ADDR_OUT_OF_RANGE = 0x02000000,
        const REQUEST_NOT_APPLICABLE = 0x04000000,
        const INVALID_PARAM = 0x08000000,
        const UNSUPPORTED_FEATURE = 0x10000000,
        const BUSY = 0x20000000,

        const DMA_NONE = 0x40000000,
        const DMA_TRANSFER = 0x40000001,
        const DMA_FIFO = 0x40000002,
        const DMA_DIRECT_MODE = 0x40000004,
        const DMA_TIMEOUT = 0x40000020,
        const DMA_PARAMETER = 0x40000040,
        const DMA_NO_TRANSFER = 0x40000080, //Abort requested with no transfer ongoing
        const DMA_NOT_SUPPORTED = 0x40000100,

        const TIMEOUT = 0x80000000,
    }
}