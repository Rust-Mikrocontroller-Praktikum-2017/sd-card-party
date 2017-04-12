#![no_std]
#![no_main]
#![feature(plugin)]
#![plugin(clippy)]
#![feature(alloc)]
#![feature(collections)]

#![allow(dead_code)]

#[macro_use]
extern crate stm32f7_discovery as stm32f7;
extern crate volatile;
extern crate r0;
// hardware register structs with accessor methods
extern crate embedded_stm32f7 as embed_stm;
extern crate alloc;
extern crate collections;

#[macro_use]
extern crate bitflags;

use stm32f7::{system_clock, sdram, lcd, board, embedded};
use embedded::interfaces::gpio::{self, Gpio};

mod dma;
mod sd;
mod storage;
mod block_device;

// TODO(ca) We need some proper modular SDRAM management
const SDRAM_START: usize = 0xC000_0000;
const SDRAM_END: usize = 0xC080_0000;
const SDRAM_LCD_SECTION_SIZE: usize = 0x0010_0000;
const SDRAM_SDMMC_SECTION_SIZE: usize = 0x0000_0800;
const SDRAM_FAT_SECTION_SIZE: usize = 0x0000_F800;

#[no_mangle]
pub unsafe extern "C" fn reset() -> ! {
    extern "C" {
        static __DATA_LOAD: u32;
        static __DATA_END: u32;
        static mut __DATA_START: u32;
        static mut __BSS_START: u32;
        static mut __BSS_END: u32;
    }

    let data_load = &__DATA_LOAD;
    let data_start = &mut __DATA_START;
    let data_end = &__DATA_END;
    let bss_start = &mut __BSS_START;
    let bss_end = &__BSS_END;

    // initializes the .data section
    //(copy the data segment initializers from flash to RAM)
    r0::init_data(data_start, data_end, data_load);
    // zeroes the .bss section
    r0::zero_bss(bss_start, bss_end);

    // initialize the heap; needed for text support
    stm32f7::heap::init();

    // initialize the FPU, so the CPU won't hang when using f32/f64 types
    let scb = stm32f7::cortex_m::peripheral::scb_mut();
    scb.cpacr.modify(|v| v | 0b1111 << 20);

    main(board::hw());
}

#[inline(never)]
fn main(hw: board::Hardware) -> ! {
    let board::Hardware {
        rcc,
        pwr,
        flash,
        fmc,
        ltdc,
        dma_2,
        gpio_a,
        gpio_b,
        gpio_c,
        gpio_d,
        gpio_e,
        gpio_f,
        gpio_g,
        gpio_h,
        gpio_i,
        gpio_j,
        gpio_k,
        sdmmc,
        ..
    } = hw;

    let mut gpio = Gpio::new(gpio_a,
                             gpio_b,
                             gpio_c,
                             gpio_d,
                             gpio_e,
                             gpio_f,
                             gpio_g,
                             gpio_h,
                             gpio_i,
                             gpio_j,
                             gpio_k);

    system_clock::init(rcc, pwr, flash);

    // enable all gpio ports
    rcc.ahb1enr.update(|r| {
        r.set_gpioaen(true);
        r.set_gpioben(true);
        r.set_gpiocen(true);
        r.set_gpioden(true);
        r.set_gpioeen(true);
        r.set_gpiofen(true);
        r.set_gpiogen(true);
        r.set_gpiohen(true);
        r.set_gpioien(true);
        r.set_gpiojen(true);
        r.set_gpioken(true);
    });

    // configure led pin as output pin
    let led_pin = (gpio::Port::PortI, gpio::Pin::Pin1);
    let mut led = gpio.to_output(led_pin,
                                 gpio::OutputType::PushPull,
                                 gpio::OutputSpeed::Low,
                                 gpio::Resistor::NoPull)
        .expect("led pin already in use");
    // turn led on - initializing...
    led.set(true);

    // init sdram (needed for display buffer)
    sdram::init(rcc, fmc, &mut gpio);
    let mut sdram_addr: usize = SDRAM_START + SDRAM_LCD_SECTION_SIZE;

    // lcd controller
    let mut lcd = lcd::init(ltdc, rcc, &mut gpio);

    // reset background to black
    lcd.set_background_color(lcd::Color::from_hex(0));

    // clear screen
    lcd.layer_2().unwrap().clear();
    lcd.layer_1().unwrap().clear();

    // TODO(ca) maybe draw some nice loading screen or something...
    stm32f7::init_stdout(lcd.layer_1().unwrap());
    println!("Welcome to the SD Card Party!\n");

    // DMA2 init
    let dma_2 = dma::DmaManager::init_dma2(dma_2, rcc);

    // SD stuff
    // enable clock of GPIO PortC and wait the required 2 peripheral clock cycles
    rcc.ahb1enr.update(|r| r.set_gpiocen(true));
    loop {
        if rcc.ahb1enr.read().gpiocen() {break;};
    }
    // SD detect port -> check if an SD Card is present
    let sd_detect_pin = gpio.to_input((gpio::Port::PortC, gpio::Pin::Pin13),
                                       gpio::Resistor::PullUp)
                        .unwrap();
    let mut sd_handle = sd::SdHandle::new(sdmmc, &dma_2, &mut sdram_addr);
    let mut sd_initialized = false;
    let mut prompt_printed = false;

    // TODO(ca) add further initialization code here

    // turn led off - initialization finished
    led.set(false);

    let mut last_led_toggle = system_clock::ticks();
    let mut last_sd_check = system_clock::ticks();
    loop {
        let ticks = system_clock::ticks();

        // every 0.5 seconds
        if ticks - last_led_toggle >= 60 {
            // toggle the led
            let led_current = led.get();
            led.set(!led_current);
            last_led_toggle = ticks;
        }

        // test for an SD card every 500 ms
        if ticks - last_sd_check >= 500 {
            // pin is set if no SD card is present
            if sd_detect_pin.get() { // no SD card present
                if sd_initialized {
                    // De-initialize the card
                    sd_initialized = false;
                }
                if !prompt_printed { // user was not yet prompted to insert a card
                    println!("Please insert SD card!");
                    prompt_printed = true;
                }
            } else if !sd_detect_pin.get() && !sd_initialized { // SD card was inserted
                println!("Initializing card.");
                sd_handle.init(&mut gpio, rcc);
                sd_initialized = true;
                prompt_printed = false;
            }
            last_sd_check = ticks;
        }
    }
}

pub fn wait(time_ms: u32) {
    let ticks = system_clock::ticks();
    while system_clock::ticks() - ticks  < time_ms as usize {};
}

const BUFFER_SIZE: usize = 0x0002_0000;

fn dma_test_setup(dma_2: &dma::DmaManagerRc, sdram_addr: &mut usize) -> (bool, dma::DmaTransfer, usize, usize) {
    let source = *sdram_addr;
    let destination = *sdram_addr + BUFFER_SIZE;

    *sdram_addr += BUFFER_SIZE * 2;

    use core::ptr;

    for i in 0..BUFFER_SIZE/4 {
        unsafe {
            ptr::write_volatile((source + i * 4) as *mut u32, i as u32);
            ptr::write_volatile((destination + i * 4) as *mut u32, 0);
        }
    }

    // Quick DMA test
    let mut dma_transfer = dma::DmaTransfer::new(
        dma_2.clone(),
        dma::Stream::S0,
        dma::Channel::C3,
        dma::Direction::MemoryToMemory,
        dma::DmaTransferNode {
            address: source as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        dma::DmaTransferNode {
            address: destination as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        (BUFFER_SIZE / 4) as u16
    );

    dma_transfer.start().expect("Failed to start DMA transfer");

    (true, dma_transfer, source, destination)
}

fn dma_test_loop(x: &mut (bool, dma::DmaTransfer, usize, usize) ) {
    use core::ptr;

    if x.0 && !x.1.is_active() {
        let s = unsafe {ptr::read_volatile((x.2 + BUFFER_SIZE - 4) as *mut u32)};
        let d = unsafe {ptr::read_volatile((x.3 + BUFFER_SIZE - 4) as *mut u32)};
        println!("DMA finished: is_error: {}, source: {:?}, destination: {:?}", x.1.is_error(), s, d);
        x.1.stop();
        x.0 = false;
    }
}