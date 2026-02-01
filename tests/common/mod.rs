mod fixtures;
pub use fixtures::*;

// Re-export commonly used types from addrslips for tests
pub use addrslips::core::db::{
    Address, AddressRepository, AddressUpdate, Area, AreaDb, AreaRepository, AreaState, AreaUpdate,
    BoundAreaRepository, Color, NewAddress, NewArea, Point, ProjectDb, Street, StreetPolyline,
    StreetRepository, StreetUpdate, Team, TeamAddress, TeamBounds, TeamRepository,
};
