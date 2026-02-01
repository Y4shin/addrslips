use std::collections::HashMap;

use crate::core::db::{address::Address, model::Point};

#[derive(Debug, Clone)]
pub struct Team {
    pub id: i64,
    pub number: u16,
    pub(super) _guard: (),
}

#[derive(Debug, Clone)]
pub struct TeamBounds {
    pub boundary: Vec<Point>,
    pub(super) _guard: (),
}

#[derive(Debug, Clone)]
pub struct TeamAddress {
    pub address_id: i64,
    pub street_id: Option<i64>,
    pub street_name: Option<String>,
    pub house_number: String,
    pub(super) _guard: (),
}

pub trait TeamRepository {
    fn get_teams(&self) -> impl Future<Output = anyhow::Result<Vec<Team>>>;
    fn get_team_by_id(&self, id: i64) -> impl Future<Output = anyhow::Result<Option<Team>>>;
    fn add_team(&self) -> impl Future<Output = anyhow::Result<Team>>;
    fn add_address(
        &self,
        team: &Team,
        address: &Address,
    ) -> impl Future<Output = anyhow::Result<()>>;
    fn remove_address(
        &self,
        team: &Team,
        address: &Address,
    ) -> impl Future<Output = anyhow::Result<()>>;
    fn get_team_addresses(
        &self,
        team: &Team,
    ) -> impl Future<Output = anyhow::Result<Vec<TeamAddress>>>;
    fn get_team_addresses_all(
        &self,
    ) -> impl Future<Output = anyhow::Result<HashMap<i64, Vec<TeamAddress>>>>;
    fn set_team_bounds(
        &self,
        team: &Team,
        bounds: &[Point],
    ) -> impl Future<Output = anyhow::Result<TeamBounds>>;
    fn get_team_bounds(
        &self,
        team: &Team,
    ) -> impl Future<Output = anyhow::Result<Option<TeamBounds>>>;
    fn remove_team_bounds(&self, team: &Team) -> impl Future<Output = anyhow::Result<()>>;
}
