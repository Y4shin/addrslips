use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Clone, Copy)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Clone)]
pub struct Address {
    pub id: Uuid,
    pub house_number: String,
    pub position: Point,
    pub confidence: u8,
    pub verified: bool,
    pub estimated_flats: Option<u16>,
    pub assigned_street_id: Option<Uuid>,
}

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Clone)]
pub struct Street {
    pub id: Uuid,
    pub name: Option<String>,
    pub polyline: Vec<Point>,
    pub verified: bool,
    pub area_id: Uuid,
}

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Clone)]
pub struct Team {
    pub id: Uuid,
    pub number: u16,
    pub assigned_addresses: Vec<Uuid>,
    pub total_flats: u32,
    pub boundary: Option<Vec<Point>>,
}

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Clone)]
pub struct Area {
    pub id: Uuid,
    pub name: String,
    pub color: AreaColor,
    pub state: AreaState,
}

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Clone)]
pub struct AreaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Clone, Copy)]
pub enum AreaState {
    Imported,
    AddressesDetected,
    AddressesCorrected,
    StreetsDetected,
    StreetsCorrected,
    AddressesAssigned,
    FlatsEstimated,
    TeamsAssigned,
    Complete,
}