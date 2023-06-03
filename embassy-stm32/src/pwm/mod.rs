pub mod complementary_pwm;
pub mod simple_pwm;

use stm32_metapac::timer::vals::Ckd;

#[cfg(feature = "unstable-pac")]
pub mod low_level {
    pub use super::sealed::*;
}

#[derive(Clone, Copy)]
pub enum Channel {
    Ch1,
    Ch2,
    Ch3,
    Ch4,
}

impl Channel {
    pub fn raw(&self) -> usize {
        match self {
            Channel::Ch1 => 0,
            Channel::Ch2 => 1,
            Channel::Ch3 => 2,
            Channel::Ch4 => 3,
        }
    }
}

#[derive(Clone, Copy)]
pub enum OutputCompareMode {
    Frozen,
    ActiveOnMatch,
    InactiveOnMatch,
    Toggle,
    ForceInactive,
    ForceActive,
    PwmMode1,
    PwmMode2,
}

impl From<OutputCompareMode> for stm32_metapac::timer::vals::Ocm {
    fn from(mode: OutputCompareMode) -> Self {
        match mode {
            OutputCompareMode::Frozen => stm32_metapac::timer::vals::Ocm::FROZEN,
            OutputCompareMode::ActiveOnMatch => stm32_metapac::timer::vals::Ocm::ACTIVEONMATCH,
            OutputCompareMode::InactiveOnMatch => stm32_metapac::timer::vals::Ocm::INACTIVEONMATCH,
            OutputCompareMode::Toggle => stm32_metapac::timer::vals::Ocm::TOGGLE,
            OutputCompareMode::ForceInactive => stm32_metapac::timer::vals::Ocm::FORCEINACTIVE,
            OutputCompareMode::ForceActive => stm32_metapac::timer::vals::Ocm::FORCEACTIVE,
            OutputCompareMode::PwmMode1 => stm32_metapac::timer::vals::Ocm::PWMMODE1,
            OutputCompareMode::PwmMode2 => stm32_metapac::timer::vals::Ocm::PWMMODE2,
        }
    }
}

pub enum CenterAlignedMode {
    EdgeAligned,
    CenterAlignedMode1,
    CenterAlignedMode2,
    CenterAlignedMode3,
}

impl From<CenterAlignedMode> for stm32_metapac::timer::vals::Cms {
    fn from(mode: CenterAlignedMode) -> Self {
        match mode {
            CenterAlignedMode::EdgeAligned => stm32_metapac::timer::vals::Cms::EDGEALIGNED,
            CenterAlignedMode::CenterAlignedMode1 => stm32_metapac::timer::vals::Cms::CENTERALIGNED1,
            CenterAlignedMode::CenterAlignedMode2 => stm32_metapac::timer::vals::Cms::CENTERALIGNED2,
            CenterAlignedMode::CenterAlignedMode3 => stm32_metapac::timer::vals::Cms::CENTERALIGNED3,
        }
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait CaptureCompare16bitInstance: crate::timer::sealed::GeneralPurpose16bitInstance {
        /// Global output enable. Does not do anything on non-advanced timers.
        unsafe fn enable_outputs(&mut self, enable: bool);

        unsafe fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode);

        unsafe fn enable_channel(&mut self, channel: Channel, enable: bool);

        unsafe fn set_compare_value(&mut self, channel: Channel, value: u16);

        unsafe fn get_max_compare_value(&self) -> u16;

        unsafe fn is_tim_enabled(&mut self) -> bool;

        unsafe fn set_center_aligned_mode(&mut self, cms: CenterAlignedMode);

        unsafe fn get_center_aligned_mode(&self) -> CenterAlignedMode;
    }

    pub trait ComplementaryCaptureCompare16bitInstance: CaptureCompare16bitInstance {
        unsafe fn set_dead_time_clock_division(&mut self, value: Ckd);

        unsafe fn set_dead_time_value(&mut self, value: u8);

        unsafe fn enable_complementary_channel(&mut self, channel: Channel, enable: bool);
    }

    pub trait CaptureCompare32bitInstance: crate::timer::sealed::GeneralPurpose32bitInstance {
        unsafe fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode);

        unsafe fn enable_channel(&mut self, channel: Channel, enable: bool);

        unsafe fn set_compare_value(&mut self, channel: Channel, value: u32);

        unsafe fn get_max_compare_value(&self) -> u32;
    }
}

pub trait CaptureCompare16bitInstance:
    sealed::CaptureCompare16bitInstance + crate::timer::GeneralPurpose16bitInstance + 'static
{
}

pub trait ComplementaryCaptureCompare16bitInstance:
    sealed::ComplementaryCaptureCompare16bitInstance + crate::timer::AdvancedControlInstance + 'static
{
}

pub trait CaptureCompare32bitInstance:
    sealed::CaptureCompare32bitInstance + CaptureCompare16bitInstance + crate::timer::GeneralPurpose32bitInstance + 'static
{
}

#[allow(unused)]
macro_rules! impl_compare_capable_16bit {
    ($inst:ident) => {
        impl crate::pwm::sealed::CaptureCompare16bitInstance for crate::peripherals::$inst {
            unsafe fn enable_outputs(&mut self, _enable: bool) {}

            unsafe fn set_output_compare_mode(&mut self, channel: crate::pwm::Channel, mode: OutputCompareMode) {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                let r = Self::regs_gp16();
                let raw_channel: usize = channel.raw();
                r.ccmr_output(raw_channel / 2)
                    .modify(|w| w.set_ocm(raw_channel % 2, mode.into()));
            }

            unsafe fn enable_channel(&mut self, channel: Channel, enable: bool) {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                Self::regs_gp16()
                    .ccer()
                    .modify(|w| w.set_cce(channel.raw(), enable));
            }

            unsafe fn set_compare_value(&mut self, channel: Channel, value: u16) {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                Self::regs_gp16().ccr(channel.raw()).modify(|w| w.set_ccr(value));
            }

            unsafe fn get_max_compare_value(&self) -> u16 {
                use crate::timer::sealed::GeneralPurpose16bitInstance;
                Self::regs_gp16().arr().read().arr()
            }

            unsafe fn is_tim_enabled(&mut self) -> bool {
                <Self as crate::timer::sealed::GeneralPurpose16bitInstance>::regs_gp16()
                    .cr1()
                    .read()
                    .cen()
            }

            unsafe fn set_center_aligned_mode(&mut self, cms: CenterAlignedMode) {
                <Self as crate::timer::sealed::GeneralPurpose16bitInstance>::regs_gp16()
                    .cr1()
                    .modify(|w| w.set_cms(cms.into()));
            }

            unsafe fn get_center_aligned_mode(&self) -> CenterAlignedMode {
                let cms = unsafe {
                    <Self as crate::timer::sealed::GeneralPurpose16bitInstance>::regs_gp16()
                        .cr1()
                        .read()
                        .cms()
                };
                let center_aligned_mode = match cms {
                    stm32_metapac::timer::vals::Cms::EDGEALIGNED => CenterAlignedMode::EdgeAligned,
                    stm32_metapac::timer::vals::Cms::CENTERALIGNED1 => CenterAlignedMode::CenterAlignedMode1,
                    stm32_metapac::timer::vals::Cms::CENTERALIGNED2 => CenterAlignedMode::CenterAlignedMode2,
                    stm32_metapac::timer::vals::Cms::CENTERALIGNED3 => CenterAlignedMode::CenterAlignedMode3,
                    _ => unreachable!(),
                };
                center_aligned_mode
            }
        }
    };
}

foreach_interrupt! {
    ($inst:ident, timer, TIM_GP16, UP, $irq:ident) => {
        impl_compare_capable_16bit!($inst);

        impl CaptureCompare16bitInstance for crate::peripherals::$inst {

        }
    };

    ($inst:ident, timer, TIM_GP32, UP, $irq:ident) => {
        impl_compare_capable_16bit!($inst);
        impl crate::pwm::sealed::CaptureCompare32bitInstance for crate::peripherals::$inst {
            unsafe fn set_output_compare_mode(
                &mut self,
                channel: crate::pwm::Channel,
                mode: OutputCompareMode,
            ) {
                use crate::timer::sealed::GeneralPurpose32bitInstance;
                let raw_channel = channel.raw();
                Self::regs_gp32().ccmr_output(raw_channel / 2).modify(|w| w.set_ocm(raw_channel % 2, mode.into()));
            }

            unsafe fn enable_channel(&mut self, channel: Channel, enable: bool) {
                use crate::timer::sealed::GeneralPurpose32bitInstance;
                Self::regs_gp32().ccer().modify(|w| w.set_cce(channel.raw(), enable));
            }

            unsafe fn set_compare_value(&mut self, channel: Channel, value: u32) {
                use crate::timer::sealed::GeneralPurpose32bitInstance;
                Self::regs_gp32().ccr(channel.raw()).modify(|w| w.set_ccr(value));
            }

            unsafe fn get_max_compare_value(&self) -> u32 {
                use crate::timer::sealed::GeneralPurpose32bitInstance;
                Self::regs_gp32().arr().read().arr() as u32
            }
        }
        impl CaptureCompare16bitInstance for crate::peripherals::$inst {

        }
        impl CaptureCompare32bitInstance for crate::peripherals::$inst {

        }
    };

    ($inst:ident, timer, TIM_ADV, UP, $irq:ident) => {
        impl crate::pwm::sealed::CaptureCompare16bitInstance for crate::peripherals::$inst {
            unsafe fn enable_outputs(&mut self, enable: bool) {
                use crate::timer::sealed::AdvancedControlInstance;
                let r = Self::regs_advanced();
                r.bdtr().modify(|w| w.set_moe(enable));
            }

            unsafe fn set_output_compare_mode(
                &mut self,
                channel: crate::pwm::Channel,
                mode: OutputCompareMode,
            ) {
                use crate::timer::sealed::AdvancedControlInstance;
                let r = Self::regs_advanced();
                let raw_channel: usize = channel.raw();
                r.ccmr_output(raw_channel / 2)
                    .modify(|w| w.set_ocm(raw_channel % 2, mode.into()));
            }

            unsafe fn enable_channel(&mut self, channel: Channel, enable: bool) {
                use crate::timer::sealed::AdvancedControlInstance;
                Self::regs_advanced()
                    .ccer()
                    .modify(|w| w.set_cce(channel.raw(), enable));
            }

            unsafe fn set_compare_value(&mut self, channel: Channel, value: u16) {
                use crate::timer::sealed::AdvancedControlInstance;
                Self::regs_advanced()
                    .ccr(channel.raw())
                    .modify(|w| w.set_ccr(value));
            }

            unsafe fn get_max_compare_value(&self) -> u16 {
                use crate::timer::sealed::AdvancedControlInstance;
                Self::regs_advanced().arr().read().arr()
            }

            unsafe fn is_tim_enabled(&mut self) -> bool {
                use crate::timer::sealed::AdvancedControlInstance;
                Self::regs_advanced()
                    .cr1()
                    .read()
                    .cen()
            }

            unsafe fn set_center_aligned_mode(&mut self, cms: CenterAlignedMode) {
                use crate::timer::sealed::AdvancedControlInstance;
                Self::regs_advanced()
                    .cr1()
                    .modify(|w| w.set_cms(cms.into()));
            }

            unsafe fn get_center_aligned_mode(&self) -> CenterAlignedMode {
                use crate::timer::sealed::AdvancedControlInstance;
                let cms = unsafe {
                    Self::regs_advanced()
                    .cr1()
                    .read()
                    .cms()
                };
                let center_aligned_mode = match cms {
                    stm32_metapac::timer::vals::Cms::EDGEALIGNED => CenterAlignedMode::EdgeAligned,
                    stm32_metapac::timer::vals::Cms::CENTERALIGNED1 => CenterAlignedMode::CenterAlignedMode1,
                    stm32_metapac::timer::vals::Cms::CENTERALIGNED2 => CenterAlignedMode::CenterAlignedMode2,
                    stm32_metapac::timer::vals::Cms::CENTERALIGNED3 => CenterAlignedMode::CenterAlignedMode3,
                    _ => unreachable!(),
                };
                center_aligned_mode
            }
        }

        impl CaptureCompare16bitInstance for crate::peripherals::$inst {

        }

        impl crate::pwm::sealed::ComplementaryCaptureCompare16bitInstance for crate::peripherals::$inst {
            unsafe fn set_dead_time_clock_division(&mut self, value: Ckd) {
                use crate::timer::sealed::AdvancedControlInstance;
                Self::regs_advanced().cr1().modify(|w| w.set_ckd(value));
            }

            unsafe fn set_dead_time_value(&mut self, value: u8) {
                use crate::timer::sealed::AdvancedControlInstance;
                Self::regs_advanced().bdtr().modify(|w| w.set_dtg(value));
            }

            unsafe fn enable_complementary_channel(&mut self, channel: Channel, enable: bool) {
                use crate::timer::sealed::AdvancedControlInstance;
                Self::regs_advanced()
                    .ccer()
                    .modify(|w| w.set_ccne(channel.raw(), enable));
            }
        }

        impl ComplementaryCaptureCompare16bitInstance for crate::peripherals::$inst {

        }
    };
}

pin_trait!(Channel1Pin, CaptureCompare16bitInstance);
pin_trait!(Channel1ComplementaryPin, CaptureCompare16bitInstance);
pin_trait!(Channel2Pin, CaptureCompare16bitInstance);
pin_trait!(Channel2ComplementaryPin, CaptureCompare16bitInstance);
pin_trait!(Channel3Pin, CaptureCompare16bitInstance);
pin_trait!(Channel3ComplementaryPin, CaptureCompare16bitInstance);
pin_trait!(Channel4Pin, CaptureCompare16bitInstance);
pin_trait!(Channel4ComplementaryPin, CaptureCompare16bitInstance);
pin_trait!(ExternalTriggerPin, CaptureCompare16bitInstance);
pin_trait!(BreakInputPin, CaptureCompare16bitInstance);
pin_trait!(BreakInputComparator1Pin, CaptureCompare16bitInstance);
pin_trait!(BreakInputComparator2Pin, CaptureCompare16bitInstance);
pin_trait!(BreakInput2Pin, CaptureCompare16bitInstance);
pin_trait!(BreakInput2Comparator1Pin, CaptureCompare16bitInstance);
pin_trait!(BreakInput2Comparator2Pin, CaptureCompare16bitInstance);
