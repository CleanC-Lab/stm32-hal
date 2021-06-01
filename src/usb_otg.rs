//! USB support, including for simulated COM ports. This module is a thin wrapper required to work with
//! the `usbd` crate.
//!
//! Requires the `usbotg_fs` or `usbotg_hs` features.
//! Used on F4, L4x5, L4x6, and H7. Others use the `usb` module.

// Based on `stm3h7xx-hal`

use crate::pac::{self, PWR, RCC};

pub use synopsys_usb_otg::UsbBus;
use synopsys_usb_otg::UsbPeripheral;

// #[cfg(not(feature = "h7455"))]
pub struct USB1 {
    pub usb_global: pac::OTG1_HS_GLOBAL,
    pub usb_device: pac::OTG1_HS_DEVICE,
    pub usb_pwrclk: pac::OTG1_HS_PWRCLK,
    // pub pin_dm: PB14<Alternate<AF12>>,
    // pub pin_dp: PB15<Alternate<AF12>>,
    // pub prec: rcc::rec::Usb1Otg,  todo
    pub hclk: u32,
}

// #[cfg(not(feature = "rm0455"))]
pub struct USB2 {
    pub usb_global: pac::OTG2_HS_GLOBAL,
    pub usb_device: pac::OTG2_HS_DEVICE,
    pub usb_pwrclk: pac::OTG2_HS_PWRCLK,
    // pub pin_dm: PA11<Alternate<AF10>>,
    // pub pin_dp: PA12<Alternate<AF10>>,
    // pub prec: rcc::rec::Usb2Otg,  todo
    pub hclk: u32,
}

macro_rules! usb_peripheral {
    ($USB:ident, $GLOBAL:ident, $en:ident, $rst:ident) => {
        unsafe impl Sync for $USB {}

        unsafe impl UsbPeripheral for $USB {
            const REGISTERS: *const () = stm32::$GLOBAL::ptr() as *const ();

            const HIGH_SPEED: bool = true;
            const FIFO_DEPTH_WORDS: usize = 1024;
            const ENDPOINT_COUNT: usize = 9;

            fn enable() {
                let pwr = unsafe { &*PWR::ptr() };
                let rcc = unsafe { &*RCC::ptr() };

                cortex_m::interrupt::free(|_| {
                    // USB Regulator in BYPASS mode
                    pwr.cr3.modify(|_, w| w.usb33den().set_bit());

                    // Enable USB peripheral
                    rcc.ahb1enr.modify(|_, w| w.$en().set_bit());

                    // Reset USB peripheral
                    rcc.ahb1rstr.modify(|_, w| w.$rst().set_bit());
                    rcc.ahb1rstr.modify(|_, w| w.$rst().clear_bit());
                });
            }

            fn ahb_frequency_hz(&self) -> u32 {
                // For correct operation, the AHB frequency should be higher
                // than 30MHz. See RM0433 Rev 7. Section 57.4.4. This is checked
                // by the UsbBus implementation in synopsys-usb-otg.

                self.hclk.0
            }
        }
    };
}

usb_peripheral! {
    USB1, OTG1_HS_GLOBAL, usb1otgen, usb1otgrst
}
pub type Usb1BusType = UsbBus<USB1>;

// #[cfg(not(feature = "rm0455"))]
usb_peripheral! {
    USB2, OTG2_HS_GLOBAL, usb2otgen, usb2otgrst
}
// #[cfg(not(feature = "rm0455"))]
pub type Usb2BusType = UsbBus<USB2>;

pub struct USB1_ULPI {
    pub usb_global: stm32::OTG1_HS_GLOBAL,
    pub usb_device: stm32::OTG1_HS_DEVICE,
    pub usb_pwrclk: stm32::OTG1_HS_PWRCLK,
    pub prec: rcc::rec::Usb1Otg,
    pub hclk: Hertz,
    pub ulpi_clk: PA5<Alternate<AF10>>,
    pub ulpi_dir: Usb1UlpiDirPin,
    pub ulpi_nxt: Usb1UlpiNxtPin,
    pub ulpi_stp: PC0<Alternate<AF10>>,
    pub ulpi_d0: PA3<Alternate<AF10>>,
    pub ulpi_d1: PB0<Alternate<AF10>>,
    pub ulpi_d2: PB1<Alternate<AF10>>,
    pub ulpi_d3: PB10<Alternate<AF10>>,
    pub ulpi_d4: PB11<Alternate<AF10>>,
    pub ulpi_d5: PB12<Alternate<AF10>>,
    pub ulpi_d6: PB13<Alternate<AF10>>,
    pub ulpi_d7: PB5<Alternate<AF10>>,
}

pub enum Usb1UlpiDirPin {
    PC2(PC2<Alternate<AF10>>),
    PI11(PI11<Alternate<AF10>>),
}

impl From<PI11<Alternate<AF10>>> for Usb1UlpiDirPin {
    fn from(v: PI11<Alternate<AF10>>) -> Self {
        Usb1UlpiDirPin::PI11(v)
    }
}

impl From<PC2<Alternate<AF10>>> for Usb1UlpiDirPin {
    fn from(v: PC2<Alternate<AF10>>) -> Self {
        Usb1UlpiDirPin::PC2(v)
    }
}

pub enum Usb1UlpiNxtPin {
    PC3(PC3<Alternate<AF10>>),
    PH4(PH4<Alternate<AF10>>),
}

impl From<PH4<Alternate<AF10>>> for Usb1UlpiNxtPin {
    fn from(v: PH4<Alternate<AF10>>) -> Self {
        Usb1UlpiNxtPin::PH4(v)
    }
}

impl From<PC3<Alternate<AF10>>> for Usb1UlpiNxtPin {
    fn from(v: PC3<Alternate<AF10>>) -> Self {
        Usb1UlpiNxtPin::PC3(v)
    }
}

unsafe impl Sync for USB1_ULPI {}

unsafe impl UsbPeripheral for USB1_ULPI {
    const REGISTERS: *const () = stm32::OTG1_HS_GLOBAL::ptr() as *const ();

    const HIGH_SPEED: bool = true;
    const FIFO_DEPTH_WORDS: usize = 1024;
    const ENDPOINT_COUNT: usize = 9;

    fn enable() {
        let rcc = unsafe { &*stm32::RCC::ptr() };

        cortex_m::interrupt::free(|_| {
            // Enable USB peripheral
            rcc.ahb1enr.modify(|_, w| w.usb1otgen().enabled());

            // Enable ULPI Clock
            rcc.ahb1enr.modify(|_, w| w.usb1ulpien().enabled());

            // Reset USB peripheral
            rcc.ahb1rstr.modify(|_, w| w.usb1otgrst().set_bit());
            rcc.ahb1rstr.modify(|_, w| w.usb1otgrst().clear_bit());
        });
    }

    fn ahb_frequency_hz(&self) -> u32 {
        self.hclk.0
    }

    fn phy_type(&self) -> synopsys_usb_otg::PhyType {
        synopsys_usb_otg::PhyType::ExternalHighSpeed
    }
}
pub type Usb1UlpiBusType = UsbBus<USB1_ULPI>;
