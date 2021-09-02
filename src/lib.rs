use embedded_hal::blocking::i2c;

const DEVICE_ADDRESS: i2c::SevenBitAddress = 0xB8;

pub struct TemperatureSensor {}

impl TemperatureSensor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn measure<I: i2c::Read>(&self, i2c: &mut I) -> Measurement {
        let raw_temperature = 0;
        let raw_humidity = 0;

        let temperature = if raw_temperature & 0x8000 != 0 {
            (raw_temperature & 0x7FFF) as i16 * -1
        } else {
            raw_temperature as i16
        };

        let humidity = if raw_humidity & 0x8000 != 0 {
            (raw_humidity & 0x7FFF) as i16 * -1
        } else {
            raw_humidity as i16
        };

        Measurement {
            temperature,
            humidity,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Measurement {
    temperature: i16,
    humidity: i16,
}

impl Measurement {
    /// Returns the integer representation of the temperature.
    ///
    /// This is a base 10 fixed point number with 1 digit behind the decimal point.
    /// The value is in degrees Celsius.
    pub fn temperature(&self) -> i16 {
        self.temperature
    }

    /// Returns the temperature as an f32.
    ///
    /// The value is in degrees Celsius.
    pub fn temperature_f32(&self) -> f32 {
        f32::from(self.temperature) * 0.1
    }

    /// Returns the integer representation of the humidity.
    ///
    /// This is a base 10 fixed point number with 1 digit behind the decimal point.
    /// The value is Relative Humidity in percent.
    pub fn humidity(&self) -> i16 {
        self.humidity
    }

    /// Returns the humidity as an f32.
    ///
    /// The value is Relative Humidity in range [0, 1].
    pub fn humidity_f32(&self) -> f32 {
        f32::from(self.humidity) * 0.001
    }
}
