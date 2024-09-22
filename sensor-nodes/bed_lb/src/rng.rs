use embassy_stm32::peripherals::ADC1;
use embassy_time::Timer;
use rand::{RngCore, SeedableRng};

use embassy_stm32::adc::{Adc, Temperature, VrefInt};
/// #Warning
/// Should only be called before async tasks are spawned. This takes a lot of
/// time to run and will disturb any async task running.
pub async fn generate_seed_blocking(adc: ADC1) -> u64 {
    let mut adc = Adc::new(adc);

    let mut vrefint = adc.enable_vrefint();
    let mut temp = adc.enable_temperature();

    // Startup delay can be combined to the maximum of either
    Timer::after_micros(Temperature::start_time_us().max(VrefInt::start_time_us()) as u64).await;

    let mut seed = [0u8; 16];
    let mut bytes = seed.chunks_exact_mut(2);
    while let Some([byte1, byte2]) = bytes.next() {
        let sample = adc.blocking_read(&mut temp);
        let most_noisy_byte = (sample & 0b0000_0000_1111_1111) as u8;
        *byte1 = most_noisy_byte;
        let sample = adc.blocking_read(&mut vrefint);
        let most_noisy_byte = (sample & 0b0000_0000_1111_1111) as u8;
        *byte2 = most_noisy_byte;
    }

    let mut rng = rand::rngs::SmallRng::from_seed(seed);
    rng.next_u64()
}
