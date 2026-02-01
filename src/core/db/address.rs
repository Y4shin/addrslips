use crate::core::db::{model::Point, street::Street};

#[derive(Debug, Clone)]
pub struct Address {
    pub id: i64,
    pub area_id: i64,
    pub house_number: String,
    pub position: Point,
    pub circle_radius: u32,
    pub confidence: f64,
    pub verified: bool,
    pub estimated_flats: Option<u16>,
    pub assigned_street_id: Option<i64>,
    pub(super) _guard: (),
}

#[derive(Debug, Clone)]
pub struct NewAddress {
    pub house_number: String,
    pub position: Point,
    pub confidence: f64,
    pub estimated_flats: Option<u16>,
    pub assigned_street_id: Option<i64>,
    pub circle_radius: u32,
}

#[derive(Debug, Clone, Default)]
pub struct AddressUpdate<'a> {
    pub house_number: Option<String>,
    pub circle_radius: Option<u32>,
    pub position: Option<Point>,
    pub confidence: Option<f64>,
    pub verified: Option<bool>,
    pub estimated_flats: Option<Option<u16>>,
    pub street: Option<Option<&'a Street>>,
}

pub trait AddressRepository {
    fn get_addresses(&self) -> impl Future<Output = anyhow::Result<Vec<Address>>>;
    fn get_address_by_id(&self, id: i64) -> impl Future<Output = anyhow::Result<Option<Address>>>;
    fn get_address_by_street(&self, street: &Street) -> impl Future<Output = anyhow::Result<Vec<Address>>>;
    fn add_address(&self, address: &NewAddress) -> impl Future<Output = anyhow::Result<Address>>;
    fn update_address(&self, address: &Address, update: &AddressUpdate) -> impl Future<Output = anyhow::Result<Address>>;
    fn delete_address(&self, address: Address) -> impl Future<Output = anyhow::Result<()>>;
}