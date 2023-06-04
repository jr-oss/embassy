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

mod cms_sealed {
    use crate::pwm::CenterAlignedMode;

    pub trait CmsAlignMode {
        const FREQ_FACTOR: u8;
        const MODE: CenterAlignedMode;
    }
}
pub trait CmsAlignMode: cms_sealed::CmsAlignMode {}
impl<T: cms_sealed::CmsAlignMode> CmsAlignMode for T {}

pub struct CmsEdgeAlignedMode;
pub struct CmsCenterAlignedMode1;
pub struct CmsCenterAlignedMode2;
pub struct CmsCenterAlignedMode3;

impl cms_sealed::CmsAlignMode for CmsEdgeAlignedMode {
    const FREQ_FACTOR: u8 = 1;
    const MODE: CenterAlignedMode = CenterAlignedMode::EdgeAligned;
}

impl cms_sealed::CmsAlignMode for CmsCenterAlignedMode1 {
    const FREQ_FACTOR: u8 = 2;
    const MODE: CenterAlignedMode = CenterAlignedMode::CenterAlignedMode1;
}

impl cms_sealed::CmsAlignMode for CmsCenterAlignedMode2 {
    const FREQ_FACTOR: u8 = 2;
    const MODE: CenterAlignedMode = CenterAlignedMode::CenterAlignedMode2;
}

impl cms_sealed::CmsAlignMode for CmsCenterAlignedMode3 {
    const FREQ_FACTOR: u8 = 2;
    const MODE: CenterAlignedMode = CenterAlignedMode::CenterAlignedMode3;
}

pub struct SimplePwm<'d, T, CMS = CmsEdgeAlignedMode> {
    inner: PeripheralRef<'d, T>,
    _cms: PhantomData<CMS>
}

impl<'d, T: CaptureCompare16bitInstance, CMS: CmsAlignMode> SimplePwm<'d, T, CMS> {
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

        let mut this = Self { inner: tim, _cms: PhantomData };

        // SAFETY: Center mode select cn be changed because timer is disabled
        unsafe { this.inner.set_center_aligned_mode(CMS::MODE); }

        this.set_freq(freq);
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
        self.inner.set_frequency(freq * CMS::FREQ_FACTOR);
    }

    pub fn get_max_duty(&self) -> u16 {
        unsafe { self.inner.get_max_compare_value() }
    }

    pub fn set_duty(&mut self, channel: Channel, duty: u16) {
        assert!(duty < self.get_max_duty());
        unsafe { self.inner.set_compare_value(channel, duty) }
    }

    pub fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode) {
        unsafe {
            self.inner.set_output_compare_mode(channel, mode);
        }
    }

    pub fn set_center_aligned_mode(&mut self, cms: CenterAlignedMode) {
        unsafe {
            self.inner.set_center_aligned_mode(cms.into());
        }
    }
}
