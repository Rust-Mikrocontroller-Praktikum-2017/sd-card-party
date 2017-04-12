use super::*;
use embed_stm::rcc::Rcc;
use stm32f7::system_clock;
use stm32f7::embedded::interfaces::gpio::Gpio;

impl SdHandle {
    // New function because I was too lazy to rewrite the init function
    pub fn new(sdmmc: &'static mut Sdmmc, /*dma: &'static Dma,*/) -> Self {
        SdHandle {
            registers: sdmmc,
            lock_type: LockType::Unlocked,
            //tx_buffer_ptr: 0,
            tx_transfer_size: 0,
            //rx_buffer_ptr: 0,
            rx_transfer_size: 0,
            context: Context::None,
            state: State::Reset,
            error_code: low_level::NONE,
            //dma_handle: dma, // TODO: vermutlich wird mehr gebraucht... -> Rx und Tx Dma Handle, evtl. bei DMA schon implementiert
            sd_card: CardInfo::new(),
        }
    }

    /// Initializes the SD according to the specified parameters in the 
    /// handle and create the associated handle.
    /// Returns Status::Error if no SD card is present.
    pub fn init(&mut self, gpio: &mut Gpio, rcc: &mut Rcc) -> Status {
        print!("Entering init() with state {}. ", self.state.to_str());
        if self.state == State::Reset {
            println!("State is reset. ");
            use embedded::interfaces::gpio::Port::*;
            use embedded::interfaces::gpio::Pin::*;
            use embedded::interfaces::gpio::Resistor;

            self.lock_type = LockType::Unlocked;
            // enable clock of GPIO PortC and wait the required 2 peripheral clock cycles
            rcc.ahb1enr.update(|r| r.set_gpiocen(true));
            loop {
                if rcc.ahb1enr.read().gpiocen() {break;};
            }
            println!("Enabled GPIO C clock.");
            // SD detect port -> check if an SD Card is present
            let sd_not_present = gpio.to_input((PortC, Pin13),
                                                Resistor::PullUp)
                                .unwrap();
            if sd_not_present.get() {
                println!(" Please insert SD card!");
                return Status::Error;
            }

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
        print!("Entering init_card(). ");
        // Default Clock configuration
        self.registers.clkcr.update(|clkcr| clkcr.set_negedge(false));
        self.registers.clkcr.update(|clkcr| clkcr.set_bypass(false));
        self.registers.clkcr.update(|clkcr| clkcr.set_pwrsav(false));
        self.registers.clkcr.update(|clkcr| clkcr.set_widbus(BusMode::Default as u8));
        self.registers.clkcr.update(|clkcr| clkcr.set_hwfc_en(false));
        self.registers.clkcr.update(|clkcr| clkcr.set_clkdiv(0x76));
        print!("Set clock default configuration. ");

        // Power up the SD card
        self.registers.clkcr.update(|clkcr| clkcr.set_clken(false)); // disable SDMMC clock
        ::wait(500);
        self.registers.power.update(|power| power.set_pwrctrl(PowerSupply::On as u8));
        ::wait(500);
        self.registers.clkcr.update(|clkcr| clkcr.set_clken(true)); // enable SDMMC clock
        print!("Power up completed. ");
        
        // Required power up waiting time before starting the SD initialization sequence
        ::wait(2);

        // Identify card operating voltage
        let errorstate = self.power_on();
        print!("Completed power_on(). ");
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
    fn init_card_low_level(&self) -> low_level::SdmmcErrorCode {
        // TODO: implement!
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
        print!("After sending CMD0: ");
        match cmd0_error {
            low_level::NONE => print!("No error. "),
            low_level::TIMEOUT => print!("Software timeout. "),
            _ => print!("Some error other than timeout. "),
        };
        if cmd0_error != low_level::NONE {return cmd0_error;};

        // Send CMD8 to get operating conditions and distinguish between SD V1.0 and V2.0.
        // CMD8 is only supported by cards supporting V2.0
        if self.cmd_send_if_cond() == low_level::NONE {
            // Card supports version 2.0
            print!("Version 2 ");
        } else {
            // Card supports only version 1.0
            print!("Version 1 ");
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
        print!("Enabled peripheral clocks. ");

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
        print!("Intialized pins for Command and Data line and for peripheral clock. ");

        // TODO: set priority for SDMMC1 Interrupt and enable it, see HAL_NVIC_SetPriority and HAL_NVIC_EnableIRQ
        // TODO?: enum for interrupt numbers, or does it exist already?
        // let priority_group = ((cortex_m::peripheral::scb::Registers.aircr & (7 << 8)) >> 8);

        // TODO?: link dma-handle
        // TODO: de-init and configure dma streams for rx and tx
        // TODO: configure dma Rx and Rx parameters
        // TODO: enable DMA_Rx and DMA_Tx interrupts
        Status::Ok
    }
}