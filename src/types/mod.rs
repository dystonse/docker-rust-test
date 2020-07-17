mod db_item;
mod default_curves;
mod delay_statistics;
mod event_type;
mod prediction_result;
mod route_data;
mod route_sections;
mod route_variant_data;
mod structured_map_serde;
mod time_slots;
mod curve_data;

pub use db_item::DbItem;
pub use default_curves::DefaultCurves;
pub use default_curves::DefaultCurveKey;
pub use delay_statistics::DelayStatistics;
pub use event_type::{EventType, EventPair, GetByEventType};
pub use prediction_result::PredictionResult;
pub use route_data::RouteData;
pub use route_sections::RouteSection;
pub use route_variant_data::RouteVariantData;
pub use time_slots::TimeSlot;
pub use curve_data::{CurveData, CurveSetData};

use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, Clone)]
pub struct PredictionBasis {
    pub stop_id: String,
    pub delay_departure: Option<i64>
}

// used to store where a prediction was generated from
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OriginType {
    Realtime,
    Schedule,
}

impl OriginType {
    pub fn to_int(&self) -> u8 {
        match self {
            Self::Realtime => 1,
            Self::Schedule => 2,
        }
    }
}

// Info about how precisely the base dataset matches the curve's purpose
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PrecisionType {
    Unknown,
    Specific, 
    SemiSpecific,
    General,
    FallbackGeneral, //TODO: come up with better names!
    SuperGeneral
}

impl PrecisionType {
    pub fn to_int(&self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::Specific => 1,
            Self::SemiSpecific => 2,
            Self::General => 3,
            Self::FallbackGeneral => 4,
            Self::SuperGeneral => 5,
        }
    }

    pub fn from_int(int: u8) -> Self {
        match int {
            1 => Self::Specific,
            2 => Self::SemiSpecific,
            3 => Self::General,
            4 => Self::FallbackGeneral,
            5 => Self::SuperGeneral,
            _ => Self::Unknown 
        }
    }
}


#[cfg(test)]
mod tests {

    use crate::FnResult;
    use super::DelayStatistics;
    use dystonse_curves::tree::{NodeData, SerdeFormat};

    #[test]
    fn test_load_save() -> FnResult<()> {
        println!("Read test file");
        let data = DelayStatistics::load_from_file("./data/test", "test_delay_statistics", &SerdeFormat::Json)?;
        println!("Save test file");
        data.save_to_file("./data/test", "test_delay_statistics_roundtrip", &SerdeFormat::Json)?;
        println!("Done with test file");

        Ok(())
    }
}