#![allow(dead_code)]

use board::{dma, rcc};
use dma::detail::Dma;

mod detail;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    StreamNotReady,

    TransactionCountNotAMultipleOf(u16),
    UnalignedMemoryAddress,
    UnalignedPeripheralAddress,
    CannotUseMemoryToMemoryTransferWithCircularMode,
    CannotUseMemoryToMemoryTransferWithDirectMode,
    MemoryAccessWouldCrossOneKilobyteBoundary,
    PeripheralAccessWouldCrossOneKilobyteBoundary,
    InvalidFifoThresholdMemoryBurstCombination,
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
pub enum Channel {
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

impl BurstMode {
    fn get_size(&self) -> u32 {
        match *self {
            BurstMode::SingleTransfer => 1,
            _ => 1 << (*self as u32 + 1)
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MemoryIndex {
    M0 = 0,
    M1 = 1,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DoubleBufferingMode {
    Disable,
    UseSecondBuffer(*mut u8),
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

impl Width {
    fn get_size(&self) -> u32 {
        1 << *self as u32
    }
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
pub enum FifoThreshold {
    Quarter = 0b00,
    Half = 0b01,
    ThreeQuarter = 0b10,
    Full = 0b11,
}

impl FifoThreshold {
    fn get_numerator(&self) -> u32 {
        match *self {
            FifoThreshold::Quarter => 1,
            FifoThreshold::Half => 2,
            FifoThreshold::ThreeQuarter => 3,
            FifoThreshold::Full => 4,
        }
    }

    fn get_denominator(&self) -> u32 {
        4
    }
}

#[derive(Debug)]
pub struct DmaTransferNode {
    pub increment_mode: IncrementMode,
    pub burst_mode: BurstMode,
    pub address: *mut u8,
    pub transaction_width: Width,
}

pub struct DmaTransfer<'a> {
    pub dma: &'a mut DmaManager,
    pub stream: Stream,
    pub channel: Channel,
    pub priority: PriorityLevel,
    pub direction: Direction,
    pub circular_mode: CircularMode,
    pub double_buffering_mode: DoubleBufferingMode,
    pub flow_controller: FlowContoller,
    pub peripheral_increment_offset_size: PeripheralIncrementOffsetSize,
    pub peripheral: DmaTransferNode,
    pub memory: DmaTransferNode,
    pub transaction_count: u16,
    pub direct_mode: DirectMode,
    pub fifo_threshold: FifoThreshold,
}

impl<'a> DmaTransfer<'a> {
    pub fn is_valid(&self) -> Option<Error> {
        const FIFO_SIZE: u32 = 16;
        let apply_circular_mode_limitations = self.circular_mode == CircularMode::Enable || self.double_buffering_mode != DoubleBufferingMode::Disable;
        let mwidth = self.memory.transaction_width.get_size();
        let pwidth = match self.peripheral_increment_offset_size {
            PeripheralIncrementOffsetSize::Force32Bit => 4,
            PeripheralIncrementOffsetSize::UsePSize => self.peripheral.transaction_width.get_size(),
        };
        let mburst_size = self.memory.burst_mode.get_size() * mwidth;
        let pburst_size = self.peripheral.burst_mode.get_size() * pwidth;
        let mcount_factor = (mburst_size / pwidth) as u16;
        let pcount_factor = pburst_size as u16;
        let mdata_before_first_kb_boundary = 1024 - (self.memory.address as u32 % 1024);
        let pdata_before_first_kb_boundary = 1024 - (self.peripheral.address as u32 % 1024);
        let mdata_size = mwidth * match self.memory.increment_mode {
             IncrementMode::Increment => self.transaction_count as u32,
             IncrementMode::Fixed => 1,
        };
        let pdata_size = pwidth * match self.peripheral.increment_mode {
             IncrementMode::Increment => self.transaction_count as u32,
             IncrementMode::Fixed => 1,
        };

        if mcount_factor == 0 || self.transaction_count % mcount_factor != 0 {
            Some(Error::TransactionCountNotAMultipleOf(mcount_factor))
        } else if self.transaction_count % pcount_factor != 0 {
            Some(Error::TransactionCountNotAMultipleOf(pcount_factor))
        } else if self.peripheral.address as u32 % self.peripheral.transaction_width.get_size() != 0 {
            Some(Error::UnalignedPeripheralAddress)
        } else if self.memory.address as u32 % self.memory.transaction_width.get_size() != 0 {
            Some(Error::UnalignedMemoryAddress)
        } else if apply_circular_mode_limitations && self.direction == Direction::MemoryToMemory {
            Some(Error::CannotUseMemoryToMemoryTransferWithCircularMode)
        } else if self.direct_mode == DirectMode::Enable && self.direction == Direction::MemoryToMemory {
            Some(Error::CannotUseMemoryToMemoryTransferWithDirectMode)
        } else if mdata_before_first_kb_boundary > mdata_size && mdata_before_first_kb_boundary % mburst_size != 0 {
            Some(Error::MemoryAccessWouldCrossOneKilobyteBoundary)
        } else if pdata_before_first_kb_boundary > pdata_size && pdata_before_first_kb_boundary % pburst_size != 0 {
            Some(Error::PeripheralAccessWouldCrossOneKilobyteBoundary)
        } else if (self.fifo_threshold.get_numerator() * FIFO_SIZE) % (self.fifo_threshold.get_denominator() * mburst_size) != 0 {
            Some(Error::InvalidFifoThresholdMemoryBurstCombination)
        } else {
            None
        }
    }

    pub fn is_ready(&self) -> bool {
        self.dma.controller.sxcr_en(self.stream) == StreamControl::Disable
    }

    pub fn is_running(&self) -> bool {
        self.dma.controller.sxcr_en(self.stream) == StreamControl::Enable
    }

    pub fn is_finished(&self) -> bool {
        self.dma.controller.tcif(self.stream) == InterruptState::Raised
    }

    pub fn is_error(&self) -> bool {
        self.is_transfer_error() || self.is_direct_mode_error()
    }

    pub fn is_transfer_error(&self) -> bool {
        self.dma.controller.teif(self.stream) == InterruptState::Raised        
    }

    pub fn is_direct_mode_error(&self) -> bool {
        self.dma.controller.dmeif(self.stream) == InterruptState::Raised        
    }

    pub fn is_active(&self) -> bool {
        self.is_running() && !self.is_finished() && !self.is_error()
    }

    pub fn prepare(&mut self) -> Result<(), Error> {
        let result = self.is_valid();

        if result.is_none() {
            if self.is_ready() {
                self.configure();

                Ok(())
            } else {
                Err(Error::StreamNotReady)
            }
        } else {
            Err(result.unwrap())
        }
    }

    pub fn start(&mut self) {
        self.dma.controller.set_sxcr_en(self.stream, StreamControl::Enable);
    }

    pub fn stop(&mut self) {
        self.dma.controller.set_sxcr_en(self.stream, StreamControl::Disable);
    }

    pub fn wait(&self) {
        while self.is_active() {}
    }

    pub fn run_and_wait(&mut self) {
        self.start();
        self.wait()
    }

    fn configure(&mut self) {
        self.dma.controller.clear_htif(self.stream);
        self.dma.controller.clear_tcif(self.stream);
        self.dma.controller.clear_teif(self.stream);
        self.dma.controller.clear_feif(self.stream);
        self.dma.controller.clear_dmeif(self.stream);

        self.dma.controller.set_sxcr_channel(self.stream, self.channel);
        self.dma.controller.set_sxcr_pl(self.stream, self.priority);
        self.dma.controller.set_sxcr_dir(self.stream, self.direction);
        self.dma.controller.set_sxcr_circ(self.stream, self.circular_mode);
        self.dma.controller.set_sxcr_dbm(self.stream, self.double_buffering_mode);
        self.dma.controller.set_sxcr_pfctrl(self.stream, self.flow_controller);
        self.dma.controller.set_sxcr_psize(self.stream, self.peripheral.transaction_width);
        self.dma.controller.set_sxcr_pinc(self.stream, self.peripheral.increment_mode);
        self.dma.controller.set_sxcr_pburst(self.stream, self.peripheral.burst_mode);
        self.dma.controller.set_sxcr_pincos(self.stream, self.peripheral_increment_offset_size);
        self.dma.controller.set_sxpar(self.stream, self.peripheral.address);
        self.dma.controller.set_sxcr_msize(self.stream, self.memory.transaction_width);
        self.dma.controller.set_sxcr_minc(self.stream, self.memory.increment_mode);
        self.dma.controller.set_sxcr_mburst(self.stream, self.memory.burst_mode);
        self.dma.controller.set_sxmxar(self.stream, MemoryIndex::M0, self.memory.address);
        self.dma.controller.set_sxndtr(self.stream, self.transaction_count);
        self.dma.controller.set_sxfcr_dmdis(self.stream, self.direct_mode);
        self.dma.controller.set_sxfcr_fth(self.stream, self.fifo_threshold);
    }
}

pub struct DmaManager {
    controller: Dma
}

impl DmaManager {
    pub fn init_dma1(dma: &'static mut dma::Dma, rcc: &mut rcc::Rcc) -> DmaManager {
       // enable DMA1 clock and wait until the clock is up
        rcc.ahb1enr.update(|r| r.set_dma1en(true));
        loop {
            if rcc.ahb1enr.read().dma1en() {break;};
        }

        DmaManager {
            controller: Dma::init(dma),
        }
    }

    pub fn init_dma2(dma: &'static mut dma::Dma, rcc: &mut rcc::Rcc) -> DmaManager {
       // enable DMA1 clock and wait until the clock is up
        rcc.ahb1enr.update(|r| r.set_dma2en(true));
        loop {
            if rcc.ahb1enr.read().dma2en() {break;};
        }

        DmaManager {
            controller: Dma::init(dma),
        }
    }
}