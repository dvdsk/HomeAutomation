use rand::{RngCore, SeedableRng};
use stm32f4::stm32f401;
// use stm32f4xx_hal::{
//     adc::{
//         config::{AdcConfig, Resolution, SampleTime},
//         Adc,
//     },
//     pac,
//     prelude::*,
// };
//
// /// #Warning
// /// Should only be called before async tasks are spawned. This takes a lot of
// /// time to run and will disturb any async task running.
// pub fn generate_seed_once() -> u64 {
//     let p = pac::Peripherals::take().expect("function generate_once should only be called once");
//     let gpioa = p.GPIOA.split();
//     let adc_pin = gpioa.pa1.into_analog();
//
//     let adc_config = AdcConfig::default().resolution(Resolution::Twelve);
//
//     let mut adc = Adc::adc1(p.ADC1, true, adc_config);
//
//     let mut seed = [0u8; 16];
//     for byte in &mut seed {
//         let sample = adc.convert(&adc_pin, SampleTime::Cycles_3);
//         let most_noisy_byte = (sample & 0b0000_0000_1111_1111) as u8;
//         *byte = most_noisy_byte;
//     }
//
//     let mut rng = rand::rngs::SmallRng::from_seed(seed);
//     rng.next_u64()
// }

/// #Warning
/// Should only be called before async tasks are spawned. This takes a lot of
/// time to run and will disturb any async task running.
pub fn generate_seed_blocking() -> u64 {
    let p =
        stm32f401::Peripherals::take().expect("function generate_once should only be called once");

    // enable clock to the adc such that we can control it
    p.RCC.apb2enr.modify(|_, reg| reg.adc1en().enabled());

    p.ADC_COMMON.ccr.modify(|_, reg| {
        reg
            // disable vbat (needed to enable the temp sensor)
            .vbate()
            .disabled()
            // enable the temperature sensor
            .tsvrefe()
            .enabled()
    });

    // set resolution to 12bit
    p.ADC1.cr1.modify(|_, reg| reg.res().twelve_bit());
    // adc conversion sequence has only 1 value
    p.ADC1.sqr1.modify(|_, reg| reg.l().bits(0b0000));
    // setup channel 18 which is connected to the internal temperature sensor
    p.ADC1.sqr3.modify(|_, reg| reg.sq1().variant(18));
    // enable the end of conversion bit
    p.ADC1.cr2.modify(|_, reg| reg.eocs().each_conversion());
    // disable continues conversions
    p.ADC1.cr2.modify(|_, reg| reg.cont().clear_bit());

    // finally start the adc
    p.ADC1.cr2.modify(|_, reg| reg.adon().set_bit());

    let adc_read = || {
        // clear any overrun status first
        p.ADC1.sr.modify(|_, reg| reg.ovr().clear_bit());
        // start single conversion mode for a regular channel (18)
        p.ADC1.cr2.modify(|_, reg| reg.swstart().set_bit());

        // while not status register end of conversion bit is set
        while !p.ADC1.sr.read().eoc().bit_is_set() {}
        
        p.ADC1.dr.read().bits()
    };

    let mut seed = [0u8; 16];
    for byte in &mut seed {
        let sample = adc_read();
        let most_noisy_byte = (sample & 0b0000_0000_1111_1111) as u8;
        *byte = most_noisy_byte;
    }

    let mut rng = rand::rngs::SmallRng::from_seed(seed);
    rng.next_u64()
}
