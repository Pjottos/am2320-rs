#![no_std]

use embedded_hal::{blocking::i2c, prelude::*, timer};

const DEVICE_ADDRESS: i2c::SevenBitAddress = 0xB8;

const FUNC_READ_REGISTERS: u8 = 0x03;

const REG_HUMIDITY_HIGH: u8 = 0;
const REG_HUMIDITY_LOW: u8 = 1;
const REG_TEMPERATURE_HIGH: u8 = 2;
const REG_TEMPERATURE_LOW: u8 = 3;
const MEASUREMENT_REG_COUNT: u8 = 4;

pub enum I2cError<I: i2c::WriteRead + i2c::Write> {
    Write(<I as i2c::Write>::Error),
    WriteRead(<I as i2c::WriteRead>::Error),
}

pub enum Error<I: i2c::WriteRead + i2c::Write> {
    SensorFailed,
    IncorrectCrc,
    I2cError(I2cError<I>),
}

pub fn measure<I, T>(i2c: &mut I, timer: &mut T) -> Result<Measurement, Error<I>>
where
    I: i2c::WriteRead + i2c::Write,
    T: timer::CountDown,
{
    const COMMAND: [u8; 3] = [
        FUNC_READ_REGISTERS,
        REG_HUMIDITY_HIGH,
        MEASUREMENT_REG_COUNT,
    ];

    // This write wakes up the sensor.
    i2c.write(DEVICE_ADDRESS, &[0x00])
        .map_err(|e| Error::I2cError(I2cError::Write(e)))?;

    let mut buf = [0; 8];
    // cannot use From impl because there is no way (that i know of)
    // to enforce that the associated error type is not our Error enum.
    // which is necessary because otherwise there is a conflicting trait impl.
    i2c.write_read(DEVICE_ADDRESS, &COMMAND, &mut buf)
        .map_err(|e| Error::I2cError(I2cError::WriteRead(e)))?;

    let func_code = buf[0];
    let read_count = buf[1];

    if func_code != FUNC_READ_REGISTERS || read_count != MEASUREMENT_REG_COUNT {
        return Err(Error::SensorFailed);
    }

    let crc = u16::from_be_bytes([buf[6], buf[7]]);
    if crc != crc16(&buf[2..6]) {
        return Err(Error::IncorrectCrc);
    }

    let raw_humidity = u16::from_be_bytes([
        buf[2 + REG_HUMIDITY_HIGH as usize],
        buf[2 + REG_HUMIDITY_LOW as usize],
    ]);
    let raw_temperature = u16::from_be_bytes([
        buf[2 + REG_TEMPERATURE_HIGH as usize],
        buf[2 + REG_TEMPERATURE_LOW as usize],
    ]);

    Ok(Measurement::from_raw(raw_temperature, raw_humidity))
}

#[derive(Debug, Copy, Clone)]
pub struct Measurement {
    temperature: i16,
    humidity: u16,
}

impl Measurement {
    fn from_raw(raw_temperature: u16, raw_humidity: u16) -> Self {
        let temperature = if raw_temperature & 0x8000 != 0 {
            (raw_temperature & 0x7FFF) as i16 * -1
        } else {
            raw_temperature as i16
        };

        Self {
            temperature,
            humidity: raw_humidity,
        }
    }

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
    pub fn humidity(&self) -> u16 {
        self.humidity
    }

    /// Returns the humidity as an f32.
    ///
    /// The value is Relative Humidity in range [0, 1].
    pub fn humidity_f32(&self) -> f32 {
        f32::from(self.humidity) * 0.001
    }
}

fn crc16(data: &[u8]) -> u16 {
    let mut crc = 0xFFFF;

    for value in data.iter().map(|&b| u16::from(b)) {
        crc ^= value;
        for _ in 0..8 {
            if crc & 0x01 != 0 {
                crc >>= 1;
                crc ^= 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }

    crc
}
