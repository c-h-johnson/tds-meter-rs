#![no_std]

use core::marker::PhantomData;

use embedded_hal::{
    adc::{Channel, OneShot},
    blocking::delay::DelayMs,
};
// for f32::powi
use num_traits::float::FloatCore;

const NO_SAMPLES: usize = 30;
const SAMPLE_INTERVAL_MS: u16 = 40;

pub enum Error<AdcError> {
    ReadError(AdcError),
}

fn median(mut data: [u16; NO_SAMPLES]) -> u16 {
    data.sort_unstable();
    return data[data.len() / 2];
}

pub struct TdsMeter<OneShotReader, Adc, Word, PinData, Delay>
where
    OneShotReader: OneShot<Adc, Word, PinData>,
    PinData: Channel<Adc>,
    Delay: DelayMs<u16>,
{
    adc: OneShotReader,
    adc_range: u16,
    adc_vref: f32,
    pin_data: PinData,
    _unused: PhantomData<Adc>,
    _unused2: PhantomData<Word>,
    _unused3: PhantomData<Delay>,
}

impl<OneShotReader, Adc, Word, PinData, Delay> TdsMeter<OneShotReader, Adc, Word, PinData, Delay>
where
    Word: Into<u16>,
    OneShotReader: OneShot<Adc, Word, PinData>,
    PinData: Channel<Adc>,
    Delay: DelayMs<u16>,
{
    pub fn new(adc: OneShotReader, adc_range: u16, adc_vref: f32, pin_data: PinData) -> Self {
        Self {
            adc,
            adc_range,
            adc_vref,
            pin_data,
            _unused: PhantomData,
            _unused2: PhantomData,
            _unused3: PhantomData,
        }
    }

    /// Output TDS value in parts per million
    ///
    /// Set temperature to the temperature of the water in °C or 25 if unsure.
    pub fn measure(
        &mut self,
        temperature: f32,
        delay: &mut Delay,
    ) -> Result<f32, Error<OneShotReader::Error>> {
        let mut data: [u16; NO_SAMPLES] = [0; NO_SAMPLES];
        for i in 0..NO_SAMPLES {
            loop {
                let read_result = self.adc.read(&mut self.pin_data);

                match read_result {
                    Ok(word) => {
                        data[i] = word.into();
                        break;
                    }
                    Err(nb::Error::Other(failed)) => {
                        return Err(Error::ReadError(failed));
                    }
                    Err(nb::Error::WouldBlock) => continue,
                };
            }

            delay.delay_ms(SAMPLE_INTERVAL_MS);
        }

        let average_data = median(data);
        let voltage = (average_data as f32 / self.adc_range as f32) * self.adc_vref;

        // temperature compensation
        let compensation_coefficient = 1.0 + 0.02 * (temperature - 25.0);
        let compensated_voltage = voltage / compensation_coefficient;

        Ok((66.71 * compensated_voltage.powi(3))
            + (-112.93 * compensated_voltage.powi(2))
            + (428.695 * compensated_voltage))
    }
}
