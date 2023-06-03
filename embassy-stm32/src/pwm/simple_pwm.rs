use core::marker::PhantomData;

use embassy_hal_common::{into_ref, PeripheralRef};

use super::*;
#[allow(unused_imports)]
use crate::gpio::sealed::{AFType, Pin};
use crate::gpio::AnyPin;
use crate::time::Hertz;
use crate::Peripheral;

pub struct Ch1;
pub struct Ch2;
pub struct Ch3;
pub struct Ch4;

pub struct PwmPin<'d, Perip, Channel> {
    _pin: PeripheralRef<'d, AnyPin>,
    phantom: PhantomData<(Perip, Channel)>,
}

macro_rules! channel_impl {
    ($new_chx:ident, $channel:ident, $pin_trait:ident) => {
        impl<'d, Perip: CaptureCompare16bitInstance> PwmPin<'d, Perip, $channel> {
            pub fn $new_chx(pin: impl Peripheral<P = impl $pin_trait<Perip>> + 'd) -> Self {
                into_ref!(pin);
                critical_section::with(|_| unsafe {
                    pin.set_low();
                    pin.set_as_af(pin.af_num(), AFType::OutputPushPull);
                    #[cfg(gpio_v2)]
                    pin.set_speed(crate::gpio::Speed::VeryHigh);
                });
                PwmPin {
                    _pin: pin.map_into(),
                    phantom: PhantomData,
                }
            }
        }
    };
}

channel_impl!(new_ch1, Ch1, Channel1Pin);
channel_impl!(new_ch2, Ch2, Channel2Pin);
channel_impl!(new_ch3, Ch3, Channel3Pin);
channel_impl!(new_ch4, Ch4, Channel4Pin);

pub struct SimplePwm<'d, T> {
    inner: PeripheralRef<'d, T>,
}

impl<'d, T: CaptureCompare16bitInstance> SimplePwm<'d, T> {
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        _ch1: Option<PwmPin<'d, T, Ch1>>,
        _ch2: Option<PwmPin<'d, T, Ch2>>,
        _ch3: Option<PwmPin<'d, T, Ch3>>,
        _ch4: Option<PwmPin<'d, T, Ch4>>,
        freq: Hertz,
    ) -> Self {
        Self::new_inner(tim, freq)
    }

    fn new_inner(tim: impl Peripheral<P = T> + 'd, freq: Hertz) -> Self {
        into_ref!(tim);

        T::enable();
        <T as crate::rcc::sealed::RccPeripheral>::reset();

        let mut this = Self { inner: tim };

        this.inner.set_frequency(freq);
        this.inner.start();

        unsafe {
            this.inner.enable_outputs(true);

            this.inner
                .set_output_compare_mode(Channel::Ch1, OutputCompareMode::PwmMode1);
            this.inner
                .set_output_compare_mode(Channel::Ch2, OutputCompareMode::PwmMode1);
            this.inner
                .set_output_compare_mode(Channel::Ch3, OutputCompareMode::PwmMode1);
            this.inner
                .set_output_compare_mode(Channel::Ch4, OutputCompareMode::PwmMode1);
        }
        this
    }

    pub fn enable(&mut self, channel: Channel) {
        unsafe {
            self.inner.enable_channel(channel, true);
        }
    }

    pub fn disable(&mut self, channel: Channel) {
        unsafe {
            self.inner.enable_channel(channel, false);
        }
    }

    pub fn tim_enable(&mut self) {
        self.inner.start();
    }

    pub fn tim_disable(&mut self) {
        self.inner.stop();
    }

    pub fn set_freq(&mut self, freq: Hertz) {
        self.inner.set_frequency(freq);
    }

    pub fn get_freq(&self) -> Hertz {
        self.inner.get_frequency()
    }

    pub fn get_max_duty(&self) -> u16 {
        unsafe { self.inner.get_max_compare_value() }
    }

    pub fn set_duty(&mut self, channel: Channel, duty: u16) {
        assert!(duty < self.get_max_duty());
        unsafe { self.inner.set_compare_value(channel, duty) }
    }

    pub fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode) {
        unsafe { self.inner.set_output_compare_mode(channel, mode); }
    }

    pub fn set_center_aligned_mode(&mut self, cms: CenterAlignedMode) {
        unsafe { self.inner.set_center_aligned_mode(cms.into()); }
    }

    pub fn get_center_aligned_mode(&self) -> CenterAlignedMode {
        unsafe {
            self.inner.get_center_aligned_mode()
        }
    }
}

pub struct SimplePwmEx<'d, T> {
    inner: SimplePwm<'d, T>,
}

impl<'d, T: CaptureCompare16bitInstance> SimplePwmEx<'d, T> {
    pub fn new(simple_pwm: SimplePwm<'d, T>) -> Self {
        Self { inner: simple_pwm }
    }

    pub fn enable(&mut self, channel: Channel) {
        self.inner.enable(channel);
    }

    pub fn disable(&mut self, channel: Channel) {
        self.inner.disable(channel);
    }

    pub fn set_freq(&mut self, freq: Hertz) {
        let scale = self.get_scale();
        self.inner.set_freq(freq * scale);
    }

    pub fn get_max_duty(&self) -> u16 {
        self.inner.get_max_duty() / self.get_scale()
    }

    pub fn set_duty(&mut self, channel: Channel, duty: u16) {
        self.inner.set_duty(channel, duty);
    }

    pub fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode) {
        self.inner.set_output_compare_mode(channel, mode);
    }

    pub fn set_center_aligned_mode(&mut self, cms: CenterAlignedMode) {
        let prev_scale = self.get_scale();

        self.inner.tim_disable(); // Don't change cms with timer enabled
        self.inner.set_center_aligned_mode(cms);

        let new_scale = self.get_scale();

        if new_scale != prev_scale {
            let f = self.inner.get_freq() * new_scale / prev_scale;
            self.inner.set_freq(f);
        }

        self.inner.tim_enable();
    }

    fn get_scale(&self) -> u16 {
        let scale = match self.inner.get_center_aligned_mode() {
            CenterAlignedMode::EdgeAligned => 1,
            CenterAlignedMode::CenterAlignedMode1 => 2,
            CenterAlignedMode::CenterAlignedMode2 => 2,
            CenterAlignedMode::CenterAlignedMode3 => 2,
        };
        scale
    }
}
