use super::*;
use embed_stm::rcc::Rcc;
use stm32f7::embedded::interfaces::gpio::Gpio;


const RX_BUFFER_PTR_OFFSET: usize = 0;
const RX_BUFFER_SIZE: usize = 512;
const RX_PERIPHERAL_ADDRESS: *mut u8 = 0x00 as *mut u8;
const RX_DMA_STREAM: dma::Stream = dma::Stream::S3;
const RX_DMA_CHANNEL: dma::Channel = dma::Channel::C4;

const TX_BUFFER_PTR_OFFSET: usize = RX_BUFFER_SIZE;
const TX_BUFFER_SIZE: usize = 512;
const TX_PERIPHERAL_ADDRESS: *mut u8 = 0x00 as *mut u8;
const TX_DMA_STREAM: dma::Stream = dma::Stream::S6;
const TX_DMA_CHANNEL: dma::Channel = dma::Channel::C4;

const SDMMC_SDRAM_SIZE: usize = RX_BUFFER_SIZE + TX_BUFFER_SIZE;

impl SdHandle {
    // New function because I was too lazy to rewrite the init function
    pub fn new(sdmmc: &'static mut Sdmmc, dma: &dma::DmaManagerRc, sdram_addr: &mut usize) -> Self {
        assert_eq!(RX_BUFFER_SIZE % 4, 0);
        assert_eq!(TX_BUFFER_SIZE % 4, 0);

        let rx_buffer_ptr = (*sdram_addr + RX_BUFFER_PTR_OFFSET) as *mut u8;
        let rx_transaction_count = (RX_BUFFER_SIZE / 4) as u16;

        let tx_buffer_ptr = (*sdram_addr + TX_BUFFER_PTR_OFFSET) as *mut u8;
        let tx_transaction_count = (TX_BUFFER_SIZE / 4) as u16;

        *sdram_addr += SDMMC_SDRAM_SIZE;

        SdHandle {
            registers: sdmmc,
            lock_type: LockType::Unlocked,
            rx_dma_transfer: dma::DmaTransfer {
                dma: dma.clone(),
                stream: RX_DMA_STREAM,
                channel: RX_DMA_CHANNEL,
                priority: dma::PriorityLevel::VeryHigh,
                direction: dma::Direction::PeripheralToMemory,
                circular_mode: dma::CircularMode::Disable,
                double_buffering_mode: dma::DoubleBufferingMode::Disable,
                flow_controller: dma::FlowContoller::Peripheral,
                peripheral_increment_offset_size: dma::PeripheralIncrementOffsetSize::UsePSize,
                peripheral: dma::DmaTransferNode {
                    address: RX_PERIPHERAL_ADDRESS,
                    burst_mode: dma::BurstMode::Incremental4,
                    increment_mode: dma::IncrementMode::Fixed,
                    transaction_width: dma::Width::Word,
                },
                memory: dma::DmaTransferNode {
                    address: rx_buffer_ptr,
                    burst_mode: dma::BurstMode::Incremental4,
                    increment_mode: dma::IncrementMode::Increment,
                    transaction_width: dma::Width::Word,
                },
                transaction_count: rx_transaction_count,
                direct_mode: dma::DirectMode::Disable,
                fifo_threshold: dma::FifoThreshold::Full,
                interrupt_transfer_complete: dma::InterruptControl::Enable,
                interrupt_half_transfer: dma::InterruptControl::Disable,
                interrupt_transfer_error: dma::InterruptControl::Enable,
                interrupt_direct_mode_error: dma::InterruptControl::Disable,
                interrupt_fifo: dma::InterruptControl::Enable,
            },
            tx_dma_transfer: dma::DmaTransfer {
                dma: dma.clone(),
                stream: TX_DMA_STREAM,
                channel: TX_DMA_CHANNEL,
                priority: dma::PriorityLevel::VeryHigh,
                direction: dma::Direction::MemoryToPeripheral,
                circular_mode: dma::CircularMode::Disable,
                double_buffering_mode: dma::DoubleBufferingMode::Disable,
                flow_controller: dma::FlowContoller::Peripheral,
                peripheral_increment_offset_size: dma::PeripheralIncrementOffsetSize::UsePSize,
                peripheral: dma::DmaTransferNode {
                    address: TX_PERIPHERAL_ADDRESS,
                    burst_mode: dma::BurstMode::Incremental4,
                    increment_mode: dma::IncrementMode::Fixed,
                    transaction_width: dma::Width::Word,
                },
                memory: dma::DmaTransferNode {
                    address: tx_buffer_ptr,
                    burst_mode: dma::BurstMode::Incremental4,
                    increment_mode: dma::IncrementMode::Increment,
                    transaction_width: dma::Width::Word,
                },
                transaction_count: tx_transaction_count,
                direct_mode: dma::DirectMode::Disable,
                fifo_threshold: dma::FifoThreshold::Full,
                interrupt_transfer_complete: dma::InterruptControl::Enable,
                interrupt_half_transfer: dma::InterruptControl::Disable,
                interrupt_transfer_error: dma::InterruptControl::Enable,
                interrupt_direct_mode_error: dma::InterruptControl::Disable,
                interrupt_fifo: dma::InterruptControl::Enable,
            },
            context: Context::None,
            state: State::Reset,
            error_code: low_level::NONE,
            sd_card: CardInfo::new(),
        }
    }

    /// Initializes the SD according to the specified parameters in the 
    /// handle and create the associated handle.
    /// Returns Status::Error if no SD card is present.
    pub fn init(&mut self, gpio: &mut Gpio, rcc: &mut Rcc) -> Status {
        //print!("Entering init() with state {}. ", self.state.to_str());
        if self.state == State::Reset {
            //println!("State is reset. ");

            self.lock_type = LockType::Unlocked;
            // enable clock of GPIO PortC and wait the required 2 peripheral clock cycles
            rcc.ahb1enr.update(|r| r.set_gpiocen(true));
            loop {
                if rcc.ahb1enr.read().gpiocen() {break;};
            }
            //println!("Enabled GPIO C clock.");
            /*
            // SD detect port -> check if an SD Card is present
            let sd_not_present = gpio.to_input((PortC, Pin13),
                                                Resistor::PullUp)
                                .unwrap();
            if sd_not_present.get() {
                println!(" Please insert SD card!");
                return Status::Error;
            }
            */

            self.init_pins(gpio, rcc);
        }
        self.state = State::Busy;
        // Initialize card parameters
        self.init_card();
        // Initialize error code
        self.error_code = low_level::NONE;
        // Initialize the operation
        self.context = Context::None;

        // SD card is initialized and ready
        self.state = State::Ready;
        Status::Ok
    }

    /// Initialize SD Card
    pub fn init_card(&mut self) -> Status {
        //print!("Entering init_card(). ");
        // Default Clock configuration
        self.registers.clkcr.update(|clkcr| clkcr.set_negedge(false));
        self.registers.clkcr.update(|clkcr| clkcr.set_bypass(false));
        self.registers.clkcr.update(|clkcr| clkcr.set_pwrsav(false));
        self.registers.clkcr.update(|clkcr| clkcr.set_widbus(BusMode::Default as u8));
        self.registers.clkcr.update(|clkcr| clkcr.set_hwfc_en(false));
        self.registers.clkcr.update(|clkcr| clkcr.set_clkdiv(0x76));
        //print!("Set clock default configuration. ");

        // Power up the SD card
        self.registers.clkcr.update(|clkcr| clkcr.set_clken(false)); // disable SDMMC clock
        ::wait(500);
        self.registers.power.update(|power| power.set_pwrctrl(PowerSupply::On as u8));
        ::wait(500);
        self.registers.clkcr.update(|clkcr| clkcr.set_clken(true)); // enable SDMMC clock
        //print!("Power up completed. ");
        
        // Required power up waiting time before starting the SD initialization sequence
        ::wait(2);

        // Identify card operating voltage
        let errorstate = self.power_on();
        //print!("Completed power_on(). ");
        if errorstate != low_level::NONE {
            self.state = State::Ready;
            self.error_code |= errorstate;
            return Status::Error;
        }

        // Card initialization
        let errorstate = self.init_card_low_level();
        if errorstate != low_level::NONE {
            self.state = State::Ready;
            self.error_code |= errorstate;
            return Status::Error;
        }

        // enable wide-bus operation
        // self.set_bus_operation_mode(BusMode::Wide4);

        Status::Ok
    }

    # [doc = "De-initialize SD Card"]
    pub fn de_init(&mut self) -> Status {
        self.state = State::Busy;
        self.power_off();
        self.de_init_low_level();
        
        self.error_code = low_level::NONE;
        self.state = State::Reset;

        Status::Ok
    }

    // represents SD_InitCard()
    fn init_card_low_level(&mut self) -> low_level::SdmmcErrorCode {
        // check if power is on
        if self.registers.power.read().pwrctrl() == 0 {return low_level::REQUEST_NOT_APPLICABLE;};

        if self.sd_card.card_type != CardType::Secured {
            // get card identification number data (CID)
            // prompt all cards to send their CID
            let cid_err = self.cmd_all_send_cid();
            if cid_err != low_level::NONE {
                return cid_err
            } else {
                self.sd_card.cid = self.get_all_response_registers();
            }

            // get RCA
            // ask the card with CMD3 to publish a new Relative Address (RCA)
            match self.cmd_send_relative_addr() {
                Ok(rca) => self.sd_card.relative_card_address = rca,
                Err(err) => return err,
            }
            
            // get card specific data (CSD)
            let rca = self.sd_card.relative_card_address as u32;
            let csd_err = self.cmd_send_csd(rca);
            if csd_err != low_level::NONE {
                return csd_err;
            } else {
                self.sd_card.csd = self.get_all_response_registers();
            }
        }

        // Get the Card Class, which is the CCC field in the CSD register
        self.sd_card.class = (self.sd_card.csd[1] >> 20) as u16;
        // TODO: fill CSD register struct (has to be declared first)

        // select the card by sending CMD7
        let rca = self.sd_card.relative_card_address as u32;
        let select_err = self.cmd_select_deselect_card(rca);
        if select_err != low_level::NONE { return select_err;}

        // TODO: save clock info in self, and set parameters here accordingly
        // Default Clock configuration
        self.registers.clkcr.update(|clkcr| clkcr.set_negedge(false));
        self.registers.clkcr.update(|clkcr| clkcr.set_bypass(false));
        self.registers.clkcr.update(|clkcr| clkcr.set_pwrsav(false));
        self.registers.clkcr.update(|clkcr| clkcr.set_widbus(BusMode::Default as u8));
        self.registers.clkcr.update(|clkcr| clkcr.set_hwfc_en(false));
        self.registers.clkcr.update(|clkcr| clkcr.set_clkdiv(0x76));

        low_level::NONE
    }

    /// De-initialize the low-level hardware (MSP layer)
    fn de_init_low_level(&self) -> Status {
        unimplemented!();
    }

    /// Enquires the card's operating voltage and configures the clock controls
    /// and other information that will be needed during operation.
    // represents SD_PowerOn()
    fn power_on(&mut self) -> low_level::SdmmcErrorCode {
        // send CMD0 to go to idle state
        let cmd0_error = self.cmd_go_idle_state();
        /*print!("After sending CMD0: ");
        match cmd0_error {
            low_level::NONE => print!("No error. "),
            low_level::TIMEOUT => print!("Software timeout. "),
            _ => print!("Some error other than timeout. "),
        };*/
        if cmd0_error != low_level::NONE {return cmd0_error;};

        // Send CMD8 to get operating conditions and distinguish between SD V1.0 and V2.0.
        // CMD8 is only supported by cards supporting V2.0
        if self.cmd_send_if_cond() == low_level::NONE {
            // Card supports version 2.0
            print!("Card supports Version 2. ");
            self.sd_card.version = CardVersion::V2x;

            // Voltage trial
            let voltage_result = self.voltage_trial(CardCapacity::High);
            match voltage_result {
                Ok(response) => {
                    if response & CardCapacity::High as u32 == 0 {
                        println!("Card is an SDSC card.");
                        self.sd_card.card_type = CardType::Sdsc;
                    } else {
                        println!("Card is an SDHC or SDXC card.");
                        self.sd_card.card_type = CardType::SdhcSdxc;
                    }
                },
                Err(v_err) => return v_err
            }


        } else {
            // Card supports only version 1.0
            print!("Card supports Version 1. ");
            self.sd_card.version = CardVersion::V1x;

            // Voltage trial
            let voltage_result = self.voltage_trial(CardCapacity::Standard);
            match voltage_result {
                Ok(_) => {
                    self.sd_card.card_type = CardType::Sdsc;
                    println!("Card is an SDSC card.");
                }
                Err(v_err) => return v_err
            }
        }

        low_level::NONE
    }

    // represents SD_PowerOFF()
    fn power_off(&self) -> low_level::SdmmcErrorCode {
        unimplemented!();
    }

    /// Initialize the low-level hardware, i.e. clocks, pins and interrupts
    // represents BSP_SD_MspInit
    fn init_pins(&mut self, gpio: &mut Gpio, rcc: &mut Rcc) -> Status {
        use embedded::interfaces::gpio::Port::*;
        use embedded::interfaces::gpio::Pin::*;
        use embedded::interfaces::gpio::{OutputType, OutputSpeed, AlternateFunction, Resistor};
        
        // enable SDIO Clock
        rcc.apb2enr.update(|r| r.set_sdmmc1en(true));
        loop {
            if rcc.apb2enr.read().sdmmc1en() {break;};
        }
        // TODO: enable DMA2 system_clock
        // enable GPIO clocks for Port C and D
        rcc.ahb1enr.update(|r| r.set_gpiocen(true));
        loop {
            if rcc.ahb1enr.read().gpiocen() {break;};
        }
        rcc.ahb1enr.update(|r| r.set_gpioden(true));
        loop {
            if rcc.ahb1enr.read().gpioden() {break;};
        }
        //print!("Enabled peripheral clocks. ");

        // SDMMC1 Data bits
        let d0 = (PortC, Pin8);
        let d1 = (PortC, Pin9);
        let d2 = (PortC, Pin10);
        let d3 = (PortC, Pin11);
        let d4 = (PortB, Pin8);
        let d5 = (PortB, Pin9);
        let d6 = (PortC, Pin6);
        let d7 = (PortC, Pin7);

        // clock pin
        let ck = (PortC, Pin12);

        // Command line
        let cmd = (PortD, Pin2);

        let pins = [d0,
                    d1,
                    d2,
                    d3,
                    d4,
                    d5,
                    d6,
                    d7,
                    ck,
                    cmd];
        gpio.to_alternate_function_all(&pins,
                                    AlternateFunction::AF12,
                                    OutputType::PushPull,
                                    OutputSpeed::High,
                                    Resistor::PullUp)
            .unwrap();
        //print!("Intialized pins for Command and Data line and for peripheral clock. ");

        // TODO: set priority for SDMMC1 Interrupt and enable it, see HAL_NVIC_SetPriority and HAL_NVIC_EnableIRQ
        // TODO?: enum for interrupt numbers, or does it exist already?
        // let priority_group = ((cortex_m::peripheral::scb::Registers.aircr & (7 << 8)) >> 8);

        // TODO?: link dma-handle
        // TODO: de-init and configure dma streams for rx and tx
        // TODO: configure dma Rx and Rx parameters
        // TODO: enable DMA_Rx and DMA_Tx interrupts
        Status::Ok
    }

    fn voltage_trial(&mut self, capacity: CardCapacity) -> Result<u32, low_level::SdmmcErrorCode> {
        // Parameters for voltage trial
        //println!("Starting voltage trial for {:?} capacity card.", capacity);
        let max_voltage_trial = 0xFFFF;
        let mut count = 0;
        while count < max_voltage_trial {
            count += 1;
            // send CMD55 with RCA 0x0 to indicate that the next command will be an ACMD
            //println!("Send CMD55.");
            if self.cmd_app_cmd(0x0) != low_level::NONE {return Err(low_level::UNSUPPORTED_FEATURE);};

            // send ACMD41
            //println!("Send ACMD41");
            if self.cmd_sd_send_op_cond(capacity) != low_level::NONE {return Err(low_level::UNSUPPORTED_FEATURE);};
            let response = self.registers.resp1.read().cardstatus1();
            
            // get operating voltage
            if (response >> 31) == 1 {
                return Ok(response);
            }
        }
        Err(low_level::INVALID_VOLTRANGE)
    }
}