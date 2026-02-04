use std::future::Future;

use crate::core::db::model::Point;

#[derive(Debug, Clone)]
pub struct Street {
    pub id: i64,
    pub name: Option<String>,
    pub verified: bool,
    pub(super) _guard: (),
}

#[derive(Debug, Clone, Default)]
pub struct StreetUpdate {
    pub name: Option<String>,
    pub verified: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct StreetPolyline {
    pub points: Vec<Point>,
    pub(super) _guard: (),
}

pub trait StreetRepository {
    fn get_streets(&self) -> impl Future<Output = anyhow::Result<Vec<Street>>>;
    fn get_street_by_id(&self, id: i64) -> impl Future<Output = anyhow::Result<Option<Street>>>;
    fn add_street(&self) -> impl Future<Output = anyhow::Result<Street>>;
    fn draw_street_polyline(&self, street: &Street, polyline: &[Point]) -> impl Future<Output = anyhow::Result<()>>;
    fn get_street_polyline(&self, street: &Street) -> impl Future<Output = anyhow::Result<Option<StreetPolyline>>>;
    fn remove_street_polyline(&self, street: &Street) -> impl Future<Output = anyhow::Result<()>>;
    fn update_street(&self, street: &Street, update: &StreetUpdate) -> impl Future<Output = anyhow::Result<Street>>;
    fn delete_street(&self, street: Street) -> impl Future<Output = anyhow::Result<()>>;
}