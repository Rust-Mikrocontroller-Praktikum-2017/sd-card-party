pub mod init;
mod low_level;
mod command;

use embed_stm::sdmmc::Sdmmc;
//use embed_stm::dma::Dma;
use stm32f7::lcd;

/// SD handle
// represents SD_HandleTypeDef
pub struct SdHandle<'a> {
    registers: &'static mut Sdmmc,
    lock_type: LockType,
    //tx_buffer_ptr: *const u8,
    tx_transfer_size: u32,
    //rx_buffer_ptr: *const u8,
    rx_transfer_size: u32,
    context: Context,
    state: State,
    error_code: low_level::SdmmcErrorCode,
    //dma_handle: &'static Dma, // TODO: vermutlich wird mehr gebraucht... -> Rx und Tx Dma Handle, evtl. bei DMA schon implementiert
    sd_card: CardInfo,
    tw: lcd::Writer<'a>,
}

// represents Status
#[derive(Debug, PartialEq, Eq)]
// TODO: remove pub
pub enum Status {
    Ok = 0x0,
    Error = 0x1,
    Busy = 0x2,
    Timeout = 0x3,
}

// represents HAL_LockTypeDef
#[derive(Debug, PartialEq, Eq)]
enum LockType {
    Locked,
    Unlocked,
}

// represents a group of defines in stm32f7xx_hal_sd.h, e.g. SD_CONTEXT_NONE
/// Context decribes which kind of operation is to be performed
#[derive(Debug, PartialEq, Eq)]
enum Context {
    None = 0x0, //TODO: No response or no data ??
    ReadSingleBlock = 0x01,
    ReadMultipleBlocks = 0x02,
    WriteSingleBlock = 0x10,
    WriteMultipleBlocks = 0x20,
    InterruptMode = 0x08,
    DmaMode = 0x80,
}

// represents HAL_SD_StateTypeDef
#[derive(Debug, PartialEq, Eq)]
enum State {
    Reset = 0x0,
    Ready = 0x1,
    Timeout = 0x2,
    Busy = 0x3,
    Programming = 0x4,
    Receiving = 0x5,
    Transfer = 0x6,
    Error = 0xF,
}

impl State {
    fn to_str(&self) -> &str {
        match *self {
            State::Reset => "Reset",
            State::Ready => "Ready",
            State::Timeout => "Timeout",
            State::Busy => "Busy",
            State::Programming => "Programming",
            State::Receiving => "Receiving",
            State::Transfer => "Transfer",
            State::Error => "Error",
        }
    }
}

// represents HAL_SD_CardInfoTypeDef
#[derive(Debug, PartialEq, Eq)]
struct CardInfo {
    card_type: CardType,
    version: CardVersion,
    // class: CardClass, -> einfach Resp2 >> 20
    relative_card_address: u32,
    number_of_blocks: usize,
    block_size: usize,
    logical_number_of_blocks: usize,
    logical_block_size: usize,
}

impl CardInfo {
    pub fn new() -> CardInfo {
        CardInfo {
            card_type: CardType::Sdsc,
            version: CardVersion::V1x,
            relative_card_address: 0x0,
            number_of_blocks: 0,
            block_size: 0,
            logical_number_of_blocks: 0,
            logical_block_size: 0,
        }
    }
}

// represents a group of defines in stm32f7xx_hal_sd.h, e.g. CARD_SDSC
#[derive(Debug, PartialEq, Eq)]
enum CardType {
    Sdsc = 0,
    SdhcSdxc = 1,
    Secured = 3,
}

#[derive(Debug, PartialEq, Eq)]
enum CardVersion {
    V1x = 0b0,
    V2x = 0b1,
}

/// Bus modes that can be selected via the clkcr register
#[derive(Debug, PartialEq, Eq)]
pub enum BusMode {
    Default = 0b00,
    Wide4 = 0b01,
    Wide8 = 0b10,
}

#[derive(Debug, PartialEq, Eq)]
enum PowerSupply {
    Off = 0b00,
    On = 0b11,
}

/// Possible values of the `WaitResp` field in the command register
#[derive(Debug, PartialEq, Eq)]
//TODO: remove 'pub'
pub enum WaitResp {
    No = 0b00,
    Short = 0b01,
    Long = 0b11,
}

#[derive(Debug, PartialEq, Eq)]
// TODO: remove pub
pub enum CardState {
    Idle = 0,
    Ready = 1,
    Ident = 2,
}

impl<'a> SdHandle<'a> {
    /// Bus can be 1, 4 or 8 bits wide.
    // represents HAL_SD_ConfigWideBusOperation
    pub fn set_bus_operation_mode(&mut self, mode: BusMode) -> Status {
        self.state = State::Busy;
        if self.sd_card.card_type != CardType::Secured {
            match mode {
                BusMode::Wide8 => self.error_code |= low_level::UNSUPPORTED_FEATURE,
                BusMode::Wide4 => self.error_code |= self.enable_wide_bus(),
                BusMode::Default => self.error_code |= self.disable_wide_bus(),
            }
        } else {
            // secured cards do not support wide bus feature
            self.error_code |= low_level::UNSUPPORTED_FEATURE;
        }

        if self.error_code == low_level::NONE {
            // Configure SDMMC peripheral
            self.registers.clkcr.update(|clkcr| clkcr.set_widbus(mode as u8));
        } else {
            // clear all static flags -> all flags in SDMMC_ICR except SDIOIT, CEATAEND and STBITERR
            self.registers.icr.update(|icr| {
                icr.set_dbckendc(true);
                icr.set_dataendc(true);
                icr.set_cmdsentc(true);
                icr.set_cmdrendc(true);
                icr.set_rxoverrc(true);
                icr.set_txunderrc(true);
                icr.set_dtimeoutc(true);
                icr.set_ctimeoutc(true);
                icr.set_dcrcfailc(true);
                icr.set_ccrcfailc(true);
            });
            self.state = State::Ready;
            return Status::Error;
        }
        self.state = State::Ready;
        Status::Ok
    }

    /// enable 4-bit wide bus mode
    fn enable_wide_bus(&mut self) -> low_level::SdmmcErrorCode {
        unimplemented!();
    }

    /// disable 4-bit wide bus mode -> set 1-bit mode
    fn disable_wide_bus(&mut self) -> low_level::SdmmcErrorCode {
        unimplemented!();
    }
}
