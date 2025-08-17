use super::{header::HistoryHeader, record::DataRecord};
use crate::sensor::protocol::LogParameter;
use chrono::Local;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct HistoryRequest {
    pub parameter: LogParameter,
    pub first_index: u16,
}

impl HistoryRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut data: Vec<u8> = vec![0x61];
        data.extend(bincode::serialize(self).unwrap());
        data
    }
}

/// Metadata about a [`HistoryReadings`]
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct HistoryInformation {
    pub interval: chrono::Duration,
    pub beginning: chrono::DateTime<Local>,
}

impl From<HistoryHeader> for HistoryInformation {
    fn from(header: HistoryHeader) -> Self {
        let interval = chrono::Duration::seconds(header.interval.into());
        let beginning = header.get_data_start().unwrap_or_else(chrono::Local::now);
        Self {
            interval,
            beginning,
        }
    }
}

/// Historical Readings from Sensor
#[derive(Debug, Clone)]
pub struct HistoryReadings {
    pub information: HistoryInformation,
    pub temperature: Vec<f32>,
    pub humidity: Vec<u8>,
    pub co2: Vec<u16>,
    pub pressure: Vec<f32>,
}

impl HistoryReadings {
    /// Get a view of the data as a vector of [`DataRecord`]
    pub fn as_records<'a>(&'a self) -> impl Iterator<Item = DataRecord> + 'a {
        self.temperature
            .iter()
            .zip(self.humidity.iter())
            .zip(self.co2.iter())
            .zip(self.pressure.iter())
            .map(|tup| {
                let (((temperature, humidity), co2), pressure) = tup;
                DataRecord {
                    temperature: *temperature,
                    humidity: *humidity,
                    pressure: *pressure,
                    co2: *co2,
                }
            })
    }

    pub fn as_records_with_datetime<'a>(
        &'a self,
    ) -> impl Iterator<Item = (chrono::DateTime<Local>, DataRecord)> + 'a {
        self.temperature
            .iter()
            .zip(self.humidity.iter())
            .zip(self.co2.iter())
            .zip(self.pressure.iter())
            .enumerate()
            .map(|(index, (((temperature, humidity), co2), pressure))| {
                let HistoryInformation {
                    interval,
                    beginning,
                } = self.information;
                let datetime = beginning + (interval * index as i32);
                let record = DataRecord {
                    temperature: *temperature,
                    humidity: *humidity,
                    pressure: *pressure,
                    co2: *co2,
                };
                (datetime, record)
            })
    }
}
