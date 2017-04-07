#![allow(dead_code)]

use core::mem::transmute;
use board::dma;
use volatile;
use super::*;

pub struct Dma {
    controller: &'static mut dma::Dma
}

impl Dma {
    pub fn init(dma: &'static mut dma::Dma) -> Dma {
        // Reset all registers to default
        dma.lifcr.write(Default::default());
        dma.hifcr.write(Default::default());
        dma.s0cr.write(Default::default());
        dma.s0ndtr.write(Default::default());
        dma.s0par.write(Default::default());
        dma.s0m0ar.write(Default::default());
        dma.s0m1ar.write(Default::default());
        dma.s0fcr.write(Default::default());
        dma.s1cr.write(Default::default());
        dma.s1ndtr.write(Default::default());
        dma.s1par.write(Default::default());
        dma.s1m0ar.write(Default::default());
        dma.s1m1ar.write(Default::default());
        dma.s1fcr.write(Default::default());
        dma.s2cr.write(Default::default());
        dma.s2ndtr.write(Default::default());
        dma.s2par.write(Default::default());
        dma.s2m0ar.write(Default::default());
        dma.s2m1ar.write(Default::default());
        dma.s2fcr.write(Default::default());
        dma.s3cr.write(Default::default());
        dma.s3ndtr.write(Default::default());
        dma.s3par.write(Default::default());
        dma.s3m0ar.write(Default::default());
        dma.s3m1ar.write(Default::default());
        dma.s3fcr.write(Default::default());
        dma.s4cr.write(Default::default());
        dma.s4ndtr.write(Default::default());
        dma.s4par.write(Default::default());
        dma.s4m0ar.write(Default::default());
        dma.s4m1ar.write(Default::default());
        dma.s4fcr.write(Default::default());
        dma.s5cr.write(Default::default());
        dma.s5ndtr.write(Default::default());
        dma.s5par.write(Default::default());
        dma.s5m0ar.write(Default::default());
        dma.s5m1ar.write(Default::default());
        dma.s5fcr.write(Default::default());
        dma.s6cr.write(Default::default());
        dma.s6ndtr.write(Default::default());
        dma.s6par.write(Default::default());
        dma.s6m0ar.write(Default::default());
        dma.s6m1ar.write(Default::default());
        dma.s6fcr.write(Default::default());
        dma.s7cr.write(Default::default());
        dma.s7ndtr.write(Default::default());
        dma.s7par.write(Default::default());
        dma.s7m0ar.write(Default::default());
        dma.s7m1ar.write(Default::default());
        dma.s7fcr.write(Default::default());

        Dma {
            controller: dma,
        }
    }

    fn _sxcr(&mut self, stream: Stream) -> &mut volatile::ReadWrite<dma::S0cr> {
        match stream {
            Stream::S0 => &mut self.controller.s0cr,
            Stream::S1 => &mut self.controller.s1cr,
            Stream::S2 => &mut self.controller.s2cr,
            Stream::S3 => &mut self.controller.s3cr,
            Stream::S4 => &mut self.controller.s4cr,
            Stream::S5 => &mut self.controller.s5cr,
            Stream::S6 => &mut self.controller.s6cr,
            Stream::S7 => &mut self.controller.s7cr,
        }
    }

    pub unsafe fn sxcr_channel(&mut self, stream: Stream) -> Channel {
        transmute(self._sxcr(stream).read().chsel())
    }

    pub unsafe fn sxcr_mburst(&mut self, stream: Stream) -> BurstMode {
        transmute(self._sxcr(stream).read().mburst())
    }

    pub unsafe fn sxcr_pburst(&mut self, stream: Stream) -> BurstMode {
        transmute(self._sxcr(stream).read().pburst())
    }

    pub unsafe fn sxcr_ct(&mut self, stream: Stream) -> MemoryIndex {
        transmute(bool_to_u8(self._sxcr(stream).read().ct()))
    }

    pub unsafe fn sxcr_dbm(&mut self, stream: Stream) -> DoubleBufferingMode {
        transmute(bool_to_u8(self._sxcr(stream).read().dbm()))
    }

    pub unsafe fn sxcr_pl(&mut self, stream: Stream) -> PriorityLevel {
        transmute(self._sxcr(stream).read().pl())
    }

    pub unsafe fn sxcr_pincos(&mut self, stream: Stream) -> PeripheralIncrementOffsetSize {
        transmute(bool_to_u8(self._sxcr(stream).read().pincos()))
    }

    pub unsafe fn sxcr_msize(&mut self, stream: Stream) -> Width {
        transmute(self._sxcr(stream).read().msize())
    }

    pub unsafe fn sxcr_psize(&mut self, stream: Stream) -> Width {
        transmute(self._sxcr(stream).read().psize())
    }

    pub unsafe fn sxcr_minc(&mut self, stream: Stream) -> IncrementMode {
        transmute(bool_to_u8(self._sxcr(stream).read().minc()))
    }

    pub unsafe fn sxcr_pinc(&mut self, stream: Stream) -> IncrementMode {
        transmute(bool_to_u8(self._sxcr(stream).read().pinc()))
    }

    pub unsafe fn sxcr_circ(&mut self, stream: Stream) -> CircularMode {
        transmute(bool_to_u8(self._sxcr(stream).read().circ()))
    }

    pub unsafe fn sxcr_dir(&mut self, stream: Stream) -> Direction {
        transmute(self._sxcr(stream).read().dir())
    }

    pub unsafe fn sxcr_pfctrl(&mut self, stream: Stream) -> FlowContoller {
        transmute(bool_to_u8(self._sxcr(stream).read().pfctrl()))
    }

    pub unsafe fn sxcr_tcie(&mut self, stream: Stream) -> InterruptControl {
        transmute(bool_to_u8(self._sxcr(stream).read().tcie()))
    }

    pub unsafe fn sxcr_htie(&mut self, stream: Stream) -> InterruptControl {
        transmute(bool_to_u8(self._sxcr(stream).read().htie()))
    }

    pub unsafe fn sxcr_teie(&mut self, stream: Stream) -> InterruptControl {
        transmute(bool_to_u8(self._sxcr(stream).read().teie()))
    }

    pub unsafe fn sxcr_dmeie(&mut self, stream: Stream) -> InterruptControl {
        transmute(bool_to_u8(self._sxcr(stream).read().dmeie()))
    }

    pub unsafe fn sxcr_en(&mut self, stream: Stream) -> StreamControl {
        transmute(bool_to_u8(self._sxcr(stream).read().en()))
    }

    pub fn set_sxcr_channel(&mut self, stream: Stream, channel: Channel) {
        self._sxcr(stream).update(|x| x.set_chsel(channel as u8))
    }

    pub fn set_sxcr_mburst(&mut self, stream: Stream, burst_mode: BurstMode) {
        self._sxcr(stream).update(|x| x.set_mburst(burst_mode as u8))
    }

    pub fn set_sxcr_pburst(&mut self, stream: Stream, burst_mode: BurstMode) {
        self._sxcr(stream).update(|x| x.set_pburst(burst_mode as u8))
    }

    pub fn set_sxcr_ct(&mut self, stream: Stream, target: MemoryIndex) {
        self._sxcr(stream).update(|x| x.set_ct(target as u8 != 0))
    }

    pub fn set_sxcr_dbm(&mut self, stream: Stream, mode: DoubleBufferingMode) {
        self._sxcr(stream).update(|x| x.set_dbm(mode as u8 != 0))
    }

    pub fn set_sxcr_pl(&mut self, stream: Stream, priority: PriorityLevel) {
        self._sxcr(stream).update(|x| x.set_pl(priority as u8))
    }

    pub fn set_sxcr_pincos(&mut self, stream: Stream, pincos: PeripheralIncrementOffsetSize) {
        self._sxcr(stream).update(|x| x.set_pincos(pincos as u8 != 0))
    }

    pub fn set_sxcr_msize(&mut self, stream: Stream, msize: Width) {
        self._sxcr(stream).update(|x| x.set_msize(msize as u8))
    }

    pub fn set_sxcr_psize(&mut self, stream: Stream, msize: Width) {
        self._sxcr(stream).update(|x| x.set_psize(msize as u8))
    }

    pub fn set_sxcr_minc(&mut self, stream: Stream, mode: IncrementMode) {
        self._sxcr(stream).update(|x| x.set_minc(mode as u8 != 0))
    }

    pub fn set_sxcr_pinc(&mut self, stream: Stream, mode: IncrementMode) {
        self._sxcr(stream).update(|x| x.set_pinc(mode as u8 != 0))
    }

    pub fn set_sxcr_circ(&mut self, stream: Stream, mode: CircularMode) {
        self._sxcr(stream).update(|x| x.set_circ(mode as u8 != 0))
    }

    pub fn set_sxcr_dir(&mut self, stream: Stream, direction: Direction) {
        self._sxcr(stream).update(|x| x.set_dir(direction as u8))
    }

    pub fn set_sxcr_pfctrl(&mut self, stream: Stream, fc: FlowContoller) {
        self._sxcr(stream).update(|x| x.set_pfctrl(fc as u8 != 0))
    }

    pub fn set_sxcr_tcie(&mut self, stream: Stream, ic: InterruptControl) {
        self._sxcr(stream).update(|x| x.set_tcie(ic as u8 != 0))
    }

    pub fn set_sxcr_htie(&mut self, stream: Stream, ic: InterruptControl) {
        self._sxcr(stream).update(|x| x.set_htie(ic as u8 != 0))
    }

    pub fn set_sxcr_teie(&mut self, stream: Stream, ic: InterruptControl) {
        self._sxcr(stream).update(|x| x.set_teie(ic as u8 != 0))
    }

    pub fn set_sxcr_dmeie(&mut self, stream: Stream, ic: InterruptControl) {
        self._sxcr(stream).update(|x| x.set_dmeie(ic as u8 != 0))
    }

    pub fn set_sxcr_en(&mut self, stream: Stream, sc: StreamControl) {
        self._sxcr(stream).update(|x| x.set_en(sc as u8 != 0))
    }

    fn _sxndtr(&mut self, stream: Stream) -> &mut volatile::ReadWrite<dma::S0ndtr> {
        match stream {
            Stream::S0 => &mut self.controller.s0ndtr,
            Stream::S1 => &mut self.controller.s1ndtr,
            Stream::S2 => &mut self.controller.s2ndtr,
            Stream::S3 => &mut self.controller.s3ndtr,
            Stream::S4 => &mut self.controller.s4ndtr,
            Stream::S5 => &mut self.controller.s5ndtr,
            Stream::S6 => &mut self.controller.s6ndtr,
            Stream::S7 => &mut self.controller.s7ndtr,
        }
    }

    pub fn sxndtr(&mut self, stream: Stream) -> u16 {
        self._sxndtr(stream).read().ndt()
    }

    pub fn set_sxndtr(&mut self, stream: Stream, count: u16) {
        self._sxndtr(stream).update(|x| x.set_ndt(count))
    }

    fn _sxpar(&mut self, stream: Stream) -> &mut volatile::ReadWrite<dma::S0par> {
        match stream {
            Stream::S0 => &mut self.controller.s0par,
            Stream::S1 => &mut self.controller.s1par,
            Stream::S2 => &mut self.controller.s2par,
            Stream::S3 => &mut self.controller.s3par,
            Stream::S4 => &mut self.controller.s4par,
            Stream::S5 => &mut self.controller.s5par,
            Stream::S6 => &mut self.controller.s6par,
            Stream::S7 => &mut self.controller.s7par,
        }
    }

    pub fn sxpar(&mut self, stream: Stream) -> u32 {
        self._sxpar(stream).read().pa()
    }

    pub fn set_sxpar(&mut self, stream: Stream, address: u32) {
        self._sxpar(stream).update(|x| x.set_pa(address))
    }

    fn _sxmxar(&mut self, stream: Stream, buffer: MemoryIndex) -> &mut volatile::ReadWrite<dma::S0m0ar> {
        match buffer {
            MemoryIndex::M0 => match stream {
                Stream::S0 => &mut self.controller.s0m0ar,
                Stream::S1 => &mut self.controller.s1m0ar,
                Stream::S2 => &mut self.controller.s2m0ar,
                Stream::S3 => &mut self.controller.s3m0ar,
                Stream::S4 => &mut self.controller.s4m0ar,
                Stream::S5 => &mut self.controller.s5m0ar,
                Stream::S6 => &mut self.controller.s6m0ar,
                Stream::S7 => &mut self.controller.s7m0ar,
            },
            MemoryIndex::M1 => match stream {
                Stream::S0 => &mut self.controller.s0m1ar,
                Stream::S1 => &mut self.controller.s1m1ar,
                Stream::S2 => &mut self.controller.s2m1ar,
                Stream::S3 => &mut self.controller.s3m1ar,
                Stream::S4 => &mut self.controller.s4m1ar,
                Stream::S5 => &mut self.controller.s5m1ar,
                Stream::S6 => &mut self.controller.s6m1ar,
                Stream::S7 => &mut self.controller.s7m1ar,
            }
        }
    }

    pub fn sxmxar(&mut self, stream: Stream, mi: MemoryIndex) -> u32 {
        self._sxmxar(stream, mi).read().m0a()
    }

    pub fn set_sxmxar(&mut self, stream: Stream, mi: MemoryIndex, address: u32) {
        self._sxmxar(stream, mi).update(|x| x.set_m0a(address))
    }

    fn _sxfcr(&mut self, stream: Stream) -> &mut volatile::ReadWrite<dma::S0fcr> {
        match stream {
            Stream::S0 => &mut self.controller.s0fcr,
            Stream::S1 => &mut self.controller.s1fcr,
            Stream::S2 => &mut self.controller.s2fcr,
            Stream::S3 => &mut self.controller.s3fcr,
            Stream::S4 => &mut self.controller.s4fcr,
            Stream::S5 => &mut self.controller.s5fcr,
            Stream::S6 => &mut self.controller.s6fcr,
            Stream::S7 => &mut self.controller.s7fcr,
        }
    }

    pub unsafe fn sxfcr_feie(&mut self, stream: Stream) -> InterruptControl {
        transmute(bool_to_u8(self._sxfcr(stream).read().feie()))
    }

    pub unsafe fn sxfcr_fs(&mut self, stream: Stream) -> FifoStatus {
         transmute(self._sxfcr(stream).read().fs())
    }

    pub unsafe fn sxfcr_dmdis(&mut self, stream: Stream) -> DirectMode {
        transmute(bool_to_u8(self._sxfcr(stream).read().dmdis()))
    }

    pub unsafe fn sxfcr_fth(&mut self, stream: Stream) -> FifoThreshold {
        transmute(self._sxfcr(stream).read().fth())
    }

    pub fn set_sxfcr_feie(&mut self, stream: Stream, ic: InterruptControl) {
        self._sxfcr(stream).update(|x| x.set_feie(ic as u8 != 0))
    }

    pub fn set_sxfcr_dmdis(&mut self, stream: Stream, mode: DirectMode) {
        self._sxfcr(stream).update(|x| x.set_dmdis(mode as u8 != 0))
    }

    pub fn set_sxfcr_fth(&mut self, stream: Stream, ft: FifoThreshold) {
        self._sxfcr(stream).update(|x| x.set_fth(ft as u8))
    }
}

fn bool_to_u8(b: bool) -> u8 {
    if b {
        1
    } else {
        0
    }
}