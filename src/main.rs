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
#[macro_use]
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

    // turn led off - initialization finished
    led.set(false);

    println!("= DMA MemoryToMemory transfer demonstration =");
    println!("");

    dma_demo_1(&dma_2);
    println!("");
    wait(1000);

    dma_demo_2(&dma_2);
    println!("");
    wait(1000);

    dma_demo_3(&dma_2);
    println!("");
    wait(1000);

    dma_demo_4(&dma_2);

    let mut last_led_toggle = system_clock::ticks();
    loop {
        let ticks = system_clock::ticks();

        // every 60 milliseconds
        if ticks - last_led_toggle >= 60 {
            // toggle the led
            let led_current = led.get();
            led.set(!led_current);
            last_led_toggle = ticks;
        }
    }
}

pub fn wait(time_ms: u32) {
    let ticks = system_clock::ticks();
    while system_clock::ticks() - ticks  < time_ms as usize {};
}


fn dma_demo_1(dma_2: &dma::DmaManagerRc) {
    const BUFFER_SIZE: usize = 8 * 4;

    println!("(1) 32b SRAM -> SRAM");

    let mut source_buffer: collections::vec::Vec<i32> = vec![1111, 9999, 6, 3, 5, -20, -100, 7];
    let mut destination_buffer: collections::vec::Vec<i32> = vec![0; 8];

    println!("s: {:?} - d: {:?}", source_buffer, destination_buffer);

    let mut dma_transfer = dma::DmaTransfer::new(
        dma_2.clone(),
        dma::Stream::S0,
        dma::Channel::C3,
        dma::Direction::MemoryToMemory,
        dma::DmaTransferNode {
            address: source_buffer.as_mut_ptr() as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        dma::DmaTransferNode {
            address: destination_buffer.as_mut_ptr() as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        (BUFFER_SIZE / 4) as u16
    );

    let finish_time;
    let start_time = system_clock::ticks();
    dma_transfer.start().expect("Failed to start DMA transfer");

    loop {
        if !dma_transfer.is_active() {
            finish_time = system_clock::ticks();
            break;
        }
    }

    dma_transfer.stop();
    println!("s: {:?} - d: {:?}", source_buffer, destination_buffer);
    println!("time: {}ms", finish_time - start_time);
}

fn dma_demo_2(dma_2: &dma::DmaManagerRc) {
    const BUFFER_SIZE: usize = 0x0002_0000;

    println!("(2) 128kb SDRAM -> SDRAM");

    let source = SDRAM_START + SDRAM_LCD_SECTION_SIZE;
    let destination = source + BUFFER_SIZE;

    use core::ptr;

    for i in 0..BUFFER_SIZE/4 {
        unsafe {
            ptr::write_volatile((source + i * 4) as *mut u32, 1 + i as u32);
            ptr::write_volatile((destination + i * 4) as *mut u32, 0);
        }
    }

    let s1 = unsafe {ptr::read_volatile((source) as *mut u32)};
    let s2 = unsafe {ptr::read_volatile((source + BUFFER_SIZE - 4) as *mut u32)};
    let d1 = unsafe {ptr::read_volatile((destination) as *mut u32)};
    let d2 = unsafe {ptr::read_volatile((destination + BUFFER_SIZE - 4) as *mut u32)};
    println!("s: {}...{}  - d: {}...{}", s1, s2, d1, d2);

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

    let start_time = system_clock::ticks();
    dma_transfer.start().expect("Failed to start DMA transfer");
    dma_transfer.wait();
    let finish_time = system_clock::ticks();
    dma_transfer.stop();

    let s1 = unsafe {ptr::read_volatile((source) as *mut u32)};
    let s2 = unsafe {ptr::read_volatile((source + BUFFER_SIZE - 4) as *mut u32)};
    let d1 = unsafe {ptr::read_volatile((destination) as *mut u32)};
    let d2 = unsafe {ptr::read_volatile((destination + BUFFER_SIZE - 4) as *mut u32)};
    println!("s: {}...{}  - d: {}...{}", s1, s2, d1, d2);
    println!("time: {}ms ", finish_time - start_time);
}

fn dma_demo_3(dma_2: &dma::DmaManagerRc) {
    const BUFFER_SIZE: usize = 4096;

    println!("(3) 4kb SRAM -> SDRAM");

    let mut source_buffer = vec![0u32; BUFFER_SIZE];
    let source = source_buffer.as_mut_ptr() as usize;
    let destination = SDRAM_START + SDRAM_LCD_SECTION_SIZE;

    use core::ptr;

    for i in 0..BUFFER_SIZE/4 {
        unsafe {
            ptr::write_volatile((source + i * 4) as *mut u32, 1 + i as u32);
            ptr::write_volatile((destination + i * 4) as *mut u32, 0);
        }
    }

    let s1 = unsafe {ptr::read_volatile((source) as *mut u32)};
    let s2 = unsafe {ptr::read_volatile((source + BUFFER_SIZE - 4) as *mut u32)};
    let d1 = unsafe {ptr::read_volatile((destination) as *mut u32)};
    let d2 = unsafe {ptr::read_volatile((destination + BUFFER_SIZE - 4) as *mut u32)};
    println!("s: {}...{}  - d: {}...{}", s1, s2, d1, d2);

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

    let start_time = system_clock::ticks();
    dma_transfer.start().expect("Failed to start DMA transfer");
    dma_transfer.wait();
    let finish_time = system_clock::ticks();
    dma_transfer.stop();

    let s1 = unsafe {ptr::read_volatile((source) as *mut u32)};
    let s2 = unsafe {ptr::read_volatile((source + BUFFER_SIZE - 4) as *mut u32)};
    let d1 = unsafe {ptr::read_volatile((destination) as *mut u32)};
    let d2 = unsafe {ptr::read_volatile((destination + BUFFER_SIZE - 4) as *mut u32)};
    println!("s: {}...{}  - d: {}...{}", s1, s2, d1, d2);
    println!("time: {}ms ", finish_time - start_time);
}

fn dma_demo_4(dma_2: &dma::DmaManagerRc) {
    const BUFFER_SIZE: usize = 0x0002_0000;

    println!("(4) 8x 128kb SDRAM -> SDRAM | parallel streams");

    let source = SDRAM_START + SDRAM_LCD_SECTION_SIZE;
    let destination_1 = source + BUFFER_SIZE;
    let destination_2 = destination_1 + BUFFER_SIZE;
    let destination_3 = destination_2 + BUFFER_SIZE;
    let destination_4 = destination_3 + BUFFER_SIZE;
    let destination_5 = destination_4 + BUFFER_SIZE;
    let destination_6 = destination_5 + BUFFER_SIZE;
    let destination_7 = destination_6 + BUFFER_SIZE;
    let destination_8 = destination_7 + BUFFER_SIZE;

    use core::ptr;

    for i in 0..BUFFER_SIZE/4 {
        unsafe {
            ptr::write_volatile((source + i * 4) as *mut u32, 1 + i as u32);
            ptr::write_volatile((destination_1 + i * 4) as *mut u32, 0);
            ptr::write_volatile((destination_2 + i * 4) as *mut u32, 0);
            ptr::write_volatile((destination_3 + i * 4) as *mut u32, 0);
            ptr::write_volatile((destination_4 + i * 4) as *mut u32, 0);
            ptr::write_volatile((destination_5 + i * 4) as *mut u32, 0);
            ptr::write_volatile((destination_6 + i * 4) as *mut u32, 0);
            ptr::write_volatile((destination_7 + i * 4) as *mut u32, 0);
            ptr::write_volatile((destination_8 + i * 4) as *mut u32, 0);
        }
    }

    let s = unsafe {ptr::read_volatile((source + BUFFER_SIZE - 4) as *mut u32)};
    let d1 = unsafe {ptr::read_volatile((destination_1 + BUFFER_SIZE - 4) as *mut u32)};
    let d2 = unsafe {ptr::read_volatile((destination_2 + BUFFER_SIZE - 4) as *mut u32)};
    let d3 = unsafe {ptr::read_volatile((destination_3 + BUFFER_SIZE - 4) as *mut u32)};
    let d4 = unsafe {ptr::read_volatile((destination_4 + BUFFER_SIZE - 4) as *mut u32)};
    let d5 = unsafe {ptr::read_volatile((destination_5 + BUFFER_SIZE - 4) as *mut u32)};
    let d6 = unsafe {ptr::read_volatile((destination_6 + BUFFER_SIZE - 4) as *mut u32)};
    let d7 = unsafe {ptr::read_volatile((destination_7 + BUFFER_SIZE - 4) as *mut u32)};
    let d8 = unsafe {ptr::read_volatile((destination_8 + BUFFER_SIZE - 4) as *mut u32)};
    println!("{} -> {}, {}, {}, {}, {}, {}, {}, {}", s, d1, d2, d3, d4, d5, d6, d7, d8);

    // Quick DMA test
    let mut dma_transfer_1 = dma::DmaTransfer::new(
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
            address: destination_1 as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        (BUFFER_SIZE / 4) as u16
    );

    let mut dma_transfer_2 = dma::DmaTransfer::new(
        dma_2.clone(),
        dma::Stream::S1,
        dma::Channel::C3,
        dma::Direction::MemoryToMemory,
        dma::DmaTransferNode {
            address: source as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        dma::DmaTransferNode {
            address: destination_2 as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        (BUFFER_SIZE / 4) as u16
    );

    let mut dma_transfer_3 = dma::DmaTransfer::new(
        dma_2.clone(),
        dma::Stream::S2,
        dma::Channel::C3,
        dma::Direction::MemoryToMemory,
        dma::DmaTransferNode {
            address: source as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        dma::DmaTransferNode {
            address: destination_3 as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        (BUFFER_SIZE / 4) as u16
    );

    let mut dma_transfer_4 = dma::DmaTransfer::new(
        dma_2.clone(),
        dma::Stream::S3,
        dma::Channel::C3,
        dma::Direction::MemoryToMemory,
        dma::DmaTransferNode {
            address: source as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        dma::DmaTransferNode {
            address: destination_4 as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        (BUFFER_SIZE / 4) as u16
    );

    let mut dma_transfer_5 = dma::DmaTransfer::new(
        dma_2.clone(),
        dma::Stream::S4,
        dma::Channel::C3,
        dma::Direction::MemoryToMemory,
        dma::DmaTransferNode {
            address: source as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        dma::DmaTransferNode {
            address: destination_5 as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        (BUFFER_SIZE / 4) as u16
    );

    let mut dma_transfer_6 = dma::DmaTransfer::new(
        dma_2.clone(),
        dma::Stream::S5,
        dma::Channel::C3,
        dma::Direction::MemoryToMemory,
        dma::DmaTransferNode {
            address: source as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        dma::DmaTransferNode {
            address: destination_6 as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        (BUFFER_SIZE / 4) as u16
    );

    let mut dma_transfer_7 = dma::DmaTransfer::new(
        dma_2.clone(),
        dma::Stream::S6,
        dma::Channel::C3,
        dma::Direction::MemoryToMemory,
        dma::DmaTransferNode {
            address: source as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        dma::DmaTransferNode {
            address: destination_7 as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        (BUFFER_SIZE / 4) as u16
    );

    let mut dma_transfer_8 = dma::DmaTransfer::new(
        dma_2.clone(),
        dma::Stream::S7,
        dma::Channel::C3,
        dma::Direction::MemoryToMemory,
        dma::DmaTransferNode {
            address: source as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        dma::DmaTransferNode {
            address: destination_8 as *mut u8,
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        (BUFFER_SIZE / 4) as u16
    );


    let mut finish_time_1 = 0;
    let mut finish_time_2 = 0;
    let mut finish_time_3 = 0;
    let mut finish_time_4 = 0;
    let mut finish_time_5 = 0;
    let mut finish_time_6 = 0;
    let mut finish_time_7 = 0;
    let mut finish_time_8 = 0;
    let start_time = system_clock::ticks();
    dma_transfer_1.start().expect("Failed to start DMA transfer");
    dma_transfer_2.start().expect("Failed to start DMA transfer");
    dma_transfer_3.start().expect("Failed to start DMA transfer");
    dma_transfer_4.start().expect("Failed to start DMA transfer");
    dma_transfer_5.start().expect("Failed to start DMA transfer");
    dma_transfer_6.start().expect("Failed to start DMA transfer");
    dma_transfer_7.start().expect("Failed to start DMA transfer");
    dma_transfer_8.start().expect("Failed to start DMA transfer");
    while finish_time_1 == 0 || finish_time_2 == 0 || finish_time_3 == 0 || finish_time_4 == 0 || finish_time_5 == 0 || finish_time_6 == 0 || finish_time_7 == 0 || finish_time_8 == 0 {
        if finish_time_1 == 0 && !dma_transfer_1.is_active() {
            finish_time_1 = system_clock::ticks();
        }
        if finish_time_2 == 0 && !dma_transfer_2.is_active() {
            finish_time_2 = system_clock::ticks();
        }
        if finish_time_3 == 0 && !dma_transfer_3.is_active() {
            finish_time_3 = system_clock::ticks();
        }
        if finish_time_4 == 0 && !dma_transfer_4.is_active() {
            finish_time_4 = system_clock::ticks();
        }
        if finish_time_5 == 0 && !dma_transfer_5.is_active() {
            finish_time_5 = system_clock::ticks();
        }
        if finish_time_6 == 0 && !dma_transfer_6.is_active() {
            finish_time_6 = system_clock::ticks();
        }
        if finish_time_7 == 0 && !dma_transfer_7.is_active() {
            finish_time_7 = system_clock::ticks();
        }
        if finish_time_8 == 0 && !dma_transfer_8.is_active() {
            finish_time_8 = system_clock::ticks();
        }
    }

    dma_transfer_1.stop();
    dma_transfer_2.stop();
    dma_transfer_3.stop();
    dma_transfer_4.stop();
    dma_transfer_5.stop();
    dma_transfer_6.stop();
    dma_transfer_7.stop();
    dma_transfer_8.stop();

    let s = unsafe {ptr::read_volatile((source + BUFFER_SIZE - 4) as *mut u32)};
    let d1 = unsafe {ptr::read_volatile((destination_1 + BUFFER_SIZE - 4) as *mut u32)};
    let d2 = unsafe {ptr::read_volatile((destination_2 + BUFFER_SIZE - 4) as *mut u32)};
    let d3 = unsafe {ptr::read_volatile((destination_3 + BUFFER_SIZE - 4) as *mut u32)};
    let d4 = unsafe {ptr::read_volatile((destination_4 + BUFFER_SIZE - 4) as *mut u32)};
    let d5 = unsafe {ptr::read_volatile((destination_5 + BUFFER_SIZE - 4) as *mut u32)};
    let d6 = unsafe {ptr::read_volatile((destination_6 + BUFFER_SIZE - 4) as *mut u32)};
    let d7 = unsafe {ptr::read_volatile((destination_7 + BUFFER_SIZE - 4) as *mut u32)};
    let d8 = unsafe {ptr::read_volatile((destination_8 + BUFFER_SIZE - 4) as *mut u32)};
    println!("{} -> {}, {}, {}, {}, {}, {}, {}, {}", s, d1, d2, d3, d4, d5, d6, d7, d8);
    println!("times: {}ms, {}ms, {}ms, {}ms, {}ms, {}ms, {}ms, {}ms", finish_time_1 - start_time, finish_time_2 - start_time, finish_time_3 - start_time, finish_time_4 - start_time, finish_time_5 - start_time, finish_time_6 - start_time, finish_time_7 - start_time, finish_time_8 - start_time);
}
