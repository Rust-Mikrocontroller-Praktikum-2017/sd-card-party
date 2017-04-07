#![allow(dead_code)]

use core::mem::size_of;
use board::dma;
use dma::detail::Dma;

mod detail;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    StreamNotReady,

    InvalidCount,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Stream {
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum Channel {
    C0 = 0b000,
    C1 = 0b001,
    C2 = 0b010,
    C3 = 0b011,
    C4 = 0b100,
    C5 = 0b101,
    C6 = 0b110,
    C7 = 0b111,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BurstMode {
    SingleTransfer = 0b00,
    Incremental4 = 0b01,
    Incremental8 = 0b10,
    Incremental16 = 0b11,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MemoryIndex {
    M0 = 0,
    M1 = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DoubleBufferingMode {
    Disable = 0,
    SwitchMemoryTargetAfterTransfer = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PriorityLevel {
    Low = 0b00,
    Medium = 0b01,
    High = 0b10,
    VeryHigh = 0b11,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PeripheralIncrementOffsetSize {
    UsePSize = 0,
    Force32Bit = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Width {
    Byte = 0b00,
    HalfWord = 0b01,
    Word = 0b10,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IncrementMode {
    Fixed = 0,
    Increment = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CircularMode {
    Disable = 0,
    Enable = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    PeripheralToMemory = 0b00,
    MemoryToPeripheral = 0b01,
    MemoryToMemory = 0b10,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlowContoller {
    DMA = 0,
    Peripheral = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InterruptControl {
    Disable = 0,
    Enable = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InterruptState {
    NotRaised = 0,
    Raised = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StreamControl {
    Disable = 0,
    Enable = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FifoStatus {
    FirstQuarter = 0b000,  //  0 < fifo_level < 1/4
    SecondQuarter = 0b001, // 1/4 ≤ fifo_level < 1/2
    ThirdQuarter = 0b010,  // 1/2 ≤ fifo_level < 3/4
    FourthQuarter = 0b011, // 3/4 ≤ fifo_level < full
    Empty = 0b100,
    Full = 0b101,
}

// Yes, this one is inverted
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DirectMode {
    Enable = 0,
    Disable = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum FifoThreshold {
    Quarter = 0b00,
    Half = 0b01,
    ThreeQuarter = 0b10,
    Full = 0b11,
}

#[derive(Debug)]
pub struct DmaTransfer {
    stream: Stream,
    channel: Channel,
    priority: PriorityLevel,
    direction: Direction,
    source: *mut u8,
    destination: *mut u8,
    width: Width,
    count: u16,
}

impl DmaTransfer {
    pub fn setup(&self, dma: &mut Dma) -> Result<(), Error> {
        let result = self.is_valid();

        if result.is_ok() {
            if self.is_ready(dma) {
                self.configure(dma);
            } else {
                return Err(Error::StreamNotReady);
            }
        }
        
        result
    }

    pub fn is_valid(&self) -> Result<(), Error> {
        if self.count < 1 {
            Err(Error::InvalidCount)
        } else {
            Ok(())
        }
    }

    pub fn is_ready(&self, dma: &mut Dma) -> bool {
        true
    }

    fn configure(&self, dma: &mut Dma) {
        let pa = match self.direction {
            Direction::PeripheralToMemory | Direction::MemoryToMemory => self.source,
            Direction::MemoryToPeripheral => self.destination,
        };
        let ma = match self.direction {
            Direction::PeripheralToMemory | Direction::MemoryToMemory => self.destination,
            Direction::MemoryToPeripheral => self.source,
        };

        dma.set_sxcr_channel(self.stream, self.channel);
        dma.set_sxcr_pl(self.stream, self.priority);
        dma.set_sxcr_dir(self.stream, self.direction);
        dma.set_sxpar(self.stream, pa as u32);
        dma.set_sxmxar(self.stream, MemoryIndex::M0, ma as u32);
    }
}

pub struct DmaManager {
    controller: Dma
}

impl DmaManager {
    pub fn init(dma: &'static mut dma::Dma) -> DmaManager {
        // Sanity check
        assert_eq!(size_of::<*mut u8>(), size_of::<u32>());

        DmaManager {
            controller: Dma::init(dma)
        }
    }

    pub fn setup_transfer(&mut self, transfer: DmaTransfer) -> Result<(), Error> {
        transfer.setup(&mut self.controller)
    }
}