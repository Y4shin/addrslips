use std::{future::Future, path::PathBuf, sync::Arc};

use image::DynamicImage;

use crate::core::db::{address::AddressRepository, model::Color, street::StreetRepository, team::TeamRepository};

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct Area {
    pub id: i64,
    pub name: String,
    pub color: Color,
    pub state: AreaState,
    pub(super) _guard: (),
}

#[derive(Debug, Clone)]
pub struct NewArea {
    pub name: String,
    pub color: Color,
    pub image_path: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct AreaUpdate {
    pub name: Option<String>,
    pub color: Option<Color>,
    pub state: Option<AreaState>,
}

pub trait BoundAreaRepository: TeamRepository + StreetRepository + AddressRepository {
    fn get_area(&self) -> impl Future<Output = anyhow::Result<Area>>;
    fn update_area(&self, update: &AreaUpdate) -> impl Future<Output = anyhow::Result<Area>>;
    fn get_image(&self) -> &DynamicImage;
    fn delete(self) -> impl Future<Output = anyhow::Result<()>>;
}

pub trait AreaRepository: 'static {
    type Repository: BoundAreaRepository where Self: 'static;
    fn get_area_repo(&self, id: i64) -> impl Future<Output = anyhow::Result<Self::Repository>> + 'static;
    fn add_area(&self, area: NewArea) -> impl Future<Output = anyhow::Result<Self::Repository>>;
    fn get_areas(&self) -> impl Future<Output = anyhow::Result<Vec<Area>>>;
}

impl TryFrom<i64> for AreaState {
    type Error = anyhow::Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AreaState::Imported),
            1 => Ok(AreaState::AddressesDetected),
            2 => Ok(AreaState::AddressesCorrected),
            3 => Ok(AreaState::StreetsDetected),
            4 => Ok(AreaState::StreetsCorrected),
            5 => Ok(AreaState::AddressesAssigned),
            6 => Ok(AreaState::FlatsEstimated),
            7 => Ok(AreaState::TeamsAssigned),
            8 => Ok(AreaState::Complete),
            _ => Err(anyhow::anyhow!("Invalid AreaState value: {}", value)),
        }
    }
}

impl From<AreaState> for i64 {
    fn from(state: AreaState) -> Self {
        match state {
            AreaState::Imported => 0,
            AreaState::AddressesDetected => 1,
            AreaState::AddressesCorrected => 2,
            AreaState::StreetsDetected => 3,
            AreaState::StreetsCorrected => 4,
            AreaState::AddressesAssigned => 5,
            AreaState::FlatsEstimated => 6,
            AreaState::TeamsAssigned => 7,
            AreaState::Complete => 8,
        }
    }
}