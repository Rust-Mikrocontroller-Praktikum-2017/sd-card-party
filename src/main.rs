#![no_std]
#![no_main]
#![feature(plugin)]
#![feature(collections)]
#![plugin(clippy)]
#![feature(collections)]

#![allow(dead_code)]

#[macro_use]
extern crate stm32f7_discovery as stm32f7;
extern crate volatile;
extern crate r0;
// hardware register structs with accessor methods
extern crate embedded_stm32f7 as embed_stm;
extern crate collections;

#[macro_use]
extern crate bitflags;

use stm32f7::{system_clock, sdram, lcd, board, embedded};
use embedded::interfaces::gpio::{self, Gpio};
use core::mem::transmute;

mod dma;
mod sd;

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
    let mut dma = dma::DmaManager::init_dma2(dma_2, rcc);

    // SD stuff
    let mut sd_handle = sd::SdHandle::new(sdmmc);
    sd_handle.init(&mut gpio, rcc);

    // TODO(ca) add further initialization code here


    // turn led off - initialization finished
    led.set(false);

    let source = [5, 100, 65535, -7];
    let mut destination = [0, 0, 0, 0];

    // Quick DMA test
    let mut dma_transfer = dma::DmaTransfer {
        dma: &mut dma,
        stream: dma::Stream::S0,
        channel: dma::Channel::C3,
        priority: dma::PriorityLevel::High,
        direction: dma::Direction::MemoryToMemory,
        circular_mode: dma::CircularMode::Disable,
        double_buffering_mode: dma::DoubleBufferingMode::Disable,
        flow_controller: dma::FlowContoller::DMA,
        peripheral_increment_offset_size: dma::PeripheralIncrementOffsetSize::UsePSize,
        peripheral: dma::DmaTransferNode {
            address: unsafe { transmute(&source) },
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        memory: dma::DmaTransferNode {
            address: unsafe { transmute(&mut destination) },
            burst_mode: dma::BurstMode::SingleTransfer,
            increment_mode: dma::IncrementMode::Increment,
            transaction_width: dma::Width::Word,
        },
        transaction_count: 4,
        direct_mode: dma::DirectMode::Disable,
        fifo_threshold: dma::FifoThreshold::Full,
    };

    println!("");

    dma_transfer.prepare().expect("Failed to prepare DMA transfer");
    dma_transfer.start();

    let mut only_once = true;

    let mut last_led_toggle = system_clock::ticks();
    loop {
        let ticks = system_clock::ticks();

        // every 0.5 seconds
        if ticks - last_led_toggle >= 60 {
            // toggle the led
            let led_current = led.get();
            led.set(!led_current);
            last_led_toggle = ticks;
        }

        if only_once && !dma_transfer.is_active() {
            println!("DMA finished: is_error: {}, source: {:?}, destination: {:?}", dma_transfer.is_error(), source, destination);
            dma_transfer.stop();
            only_once = false;
        }
    }
}

pub fn wait(time_ms: u32) {
    let ticks = system_clock::ticks();
    while system_clock::ticks() - ticks  < time_ms as usize {};
}
