mod address;
mod area;
mod model;
mod project;
mod state;
mod street;
mod team;

use std::{ops::Deref, path::Path, sync::Arc};

use anyhow::Ok;
use image::DynamicImage;
use sqlx::Connection;
use state::ProjectState;
use time::OffsetDateTime;

pub use address::{Address, AddressRepository, AddressUpdate, NewAddress};
pub use area::{Area, AreaRepository, AreaState, AreaUpdate, BoundAreaRepository, NewArea};
pub use model::{Color, Point};
pub use project::{ProjectRepository, UpdateProjectSettings};
pub use street::{Street, StreetPolyline, StreetRepository, StreetUpdate};
pub use team::{Team, TeamAddress, TeamBounds, TeamRepository};

#[derive(Debug)]
pub struct ProjectDb {
    state: Arc<ProjectState>,
}

impl ProjectDb {
    pub async fn new<P: AsRef<Path>>(project_file: P) -> anyhow::Result<Self> {
        Ok(Self {
            state: Arc::new(ProjectState::new(project_file).await?),
        })
    }

    /// Explicitly save the project to disk.
    /// This is required when dropping in an async context (e.g., tests with #[tokio::test]).
    pub async fn save_project(&self) -> anyhow::Result<()> {
        self.state.save_project().await
    }
}

pub struct AreaDb {
    state: Arc<ProjectState>,
    area_id: i64,
    image: DynamicImage,
}

impl std::fmt::Debug for AreaDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AreaDb")
            .field("area_id", &self.area_id)
            .field("state", &self.state)
            .finish()
    }
}

impl ProjectRepository for ProjectDb {
    async fn get_project_name(&self) -> anyhow::Result<String> {
        let mut conn = self.state.conn().await?;
        let name = sqlx::query!(r#"SELECT value FROM project_metadata WHERE key = 'name'"#)
            .fetch_one(&mut **conn)
            .await?
            .value;
        Ok(name)
    }

    async fn get_project_created_at(&self) -> anyhow::Result<OffsetDateTime> {
        let mut conn = self.state.conn().await?;
        let created_at_str =
            sqlx::query!(r#"SELECT value FROM project_metadata WHERE key = 'created_at'"#)
                .fetch_one(&mut **conn)
                .await?
                .value;
        let created_at = OffsetDateTime::parse(
            &created_at_str,
            &time::format_description::well_known::Rfc3339,
        )?;
        Ok(created_at)
    }

    async fn get_target_address_count(&self) -> anyhow::Result<u64> {
        let mut conn = self.state.conn().await?;

        let value = sqlx::query!(
            r#"SELECT value FROM project_metadata WHERE key = 'target_address_count'"#
        )
        .fetch_one(&mut **conn)
        .await?
        .value
        .parse()?;
        Ok(value)
    }

    async fn set_project_settings(
        &self,
        settings: project::UpdateProjectSettings,
    ) -> anyhow::Result<()> {
        let mut conn = self.state.conn().await?;
        let mut items = vec![];
        if let Some(name) = settings.name {
            items.push(("name", name));
        }
        if let Some(target_address_count) = settings.target_address_count {
            items.push(("target_address_count", target_address_count.to_string()));
        }
        if let Some(created_at) = settings.created_at {
            items.push((
                "created_at",
                created_at.format(&time::format_description::well_known::Rfc3339)?,
            ));
        }
        for (key, value) in items {
            sqlx::query!(
                r#"INSERT INTO project_metadata (key, value) VALUES ($1, $2)
                ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value"#,
                key,
                value
            )
            .execute(&mut **conn)
            .await?;
        }
        Ok(())
    }
}

impl AreaRepository for ProjectDb {
    type Repository = AreaDb;

    fn get_area_repo(
        &self,
        id: i64,
    ) -> impl std::future::Future<Output = anyhow::Result<Self::Repository>> + 'static {
        let state = self.state.clone();
        async move {
            let mut conn = state.conn().await?;
            let image_fname = sqlx::query!("SELECT image_fname FROM area WHERE id = $1", id)
                .fetch_one(&mut **conn)
                .await?
                .image_fname;
            let image = state.load_area_image(&image_fname).await?;
            Ok(AreaDb {
                state: state.clone(),
                area_id: id,
                image,
            })
        }
    }

    fn add_area(
        &self,
        area: NewArea,
    ) -> impl std::future::Future<Output = anyhow::Result<Self::Repository>> + 'static {
        let state = self.state.clone();
        async move {
            let mut conn = state.conn().await?;
            let image_fname = state.store_area_image(&area.image_path).await?;
            let color_int = i64::from(area.color);
            let initial_state = i64::from(AreaState::Imported);
            let area_id = sqlx::query!(
                "INSERT INTO area (name, color, image_fname, state) VALUES ($1, $2, $3, $4) RETURNING id",
                area.name,
                color_int,
                image_fname,
                initial_state
            )
            .fetch_one(&mut **conn)
            .await?
            .id;
            let image = state.load_area_image(&image_fname).await?;
            Ok(AreaDb {
                state: state.clone(),
                area_id,
                image,
            })
        }
    }

    async fn get_areas(&self) -> anyhow::Result<Vec<Area>> {
        let mut conn = self.state.conn().await?;
        sqlx::query!(r#"SELECT id as "id!: i64", name, color, state FROM area ORDER BY id ASC;"#)
            .fetch_all(&mut **conn)
            .await?
            .into_iter()
            .map(|record| {
                let color = Color::try_from(record.color)?;
                let state = AreaState::try_from(record.state)?;
                Ok(Area {
                    id: record.id,
                    name: record.name,
                    color,
                    state,
                    _guard: (),
                })
            })
            .collect()
    }
}

impl TeamRepository for AreaDb {
    async fn get_teams(&self) -> anyhow::Result<Vec<Team>> {
        let mut conn = self.state.conn().await?;
        Ok(sqlx::query!(
            r#"SELECT id as "id!: i64", num FROM team WHERE area_id = $1 ORDER BY id ASC"#,
            self.area_id
        )
        .fetch_all(&mut **conn)
        .await?
        .into_iter()
        .map(|record| Team {
            id: record.id,
            number: record.num as u16,
            _guard: (),
        })
        .collect())
    }

    async fn get_team_by_id(&self, id: i64) -> anyhow::Result<Option<Team>> {
        let mut conn = self.state.conn().await?;
        if let Some(record) = sqlx::query!(
            r#"SELECT id as "id!: i64", num FROM team WHERE area_id = $1 AND id = $2"#,
            self.area_id,
            id
        )
        .fetch_optional(&mut **conn)
        .await?
        {
            Ok(Some(Team {
                id: record.id,
                number: record.num as u16,
                _guard: (),
            }))
        } else {
            Ok(None)
        }
    }

    async fn add_team(&self) -> anyhow::Result<Team> {
        let mut conn = self.state.conn().await?;
        let record = sqlx::query!(
            r#"INSERT INTO team (area_id, num) VALUES ($1, (
                SELECT COALESCE(MAX(num), -1) + 1 FROM team WHERE area_id = $1
            )) RETURNING id as "id!: i64", num"#,
            self.area_id
        )
        .fetch_one(&mut **conn)
        .await?;
        Ok(Team {
            id: record.id,
            number: record.num as u16,
            _guard: (),
        })
    }

    async fn add_address(&self, team: &Team, address: &Address) -> anyhow::Result<()> {
        let mut conn = self.state.conn().await?;
        sqlx::query!(
            r#"INSERT INTO team_assignment (team_id, address_id, area_id) VALUES ($1, $2, $3)"#,
            team.id,
            address.id,
            self.area_id
        )
        .execute(&mut **conn)
        .await?;
        Ok(())
    }

    async fn remove_address(&self, team: &Team, address: &Address) -> anyhow::Result<()> {
        let mut conn = self.state.conn().await?;
        sqlx::query!(
            r#"DELETE FROM team_assignment WHERE team_id = $1 AND address_id = $2 AND area_id = $3"#,
            team.id,
            address.id,
            self.area_id
        )
        .execute(&mut **conn)
        .await?;
        Ok(())
    }

    async fn get_team_addresses(&self, team: &Team) -> anyhow::Result<Vec<team::TeamAddress>> {
        let mut conn = self.state.conn().await?;
        Ok(sqlx::query!(
            r#"SELECT
                a.id as "address_id!: i64",
                s.id as "street_id",
                s.name as "street_name?",
                a.house_number
            FROM team_assignment ta
            JOIN address a ON ta.address_id = a.id
            LEFT JOIN street s ON a.street_id = s.id
            WHERE ta.team_id = $1
            AND a.area_id = $2
            ORDER BY a.id ASC"#,
            team.id,
            self.area_id
        )
        .fetch_all(&mut **conn)
        .await?
        .into_iter()
        .map(|record| team::TeamAddress {
            address_id: record.address_id,
            street_id: record.street_id,
            street_name: record.street_name,
            house_number: record.house_number,
            _guard: (),
        })
        .collect())
    }

    async fn get_team_addresses_all(
        &self,
    ) -> anyhow::Result<std::collections::HashMap<i64, Vec<team::TeamAddress>>> {
        let mut conn = self.state.conn().await?;
        let records = sqlx::query!(
            r#"SELECT
                ta.team_id as "team_id!: i64",
                a.id as "address_id!: i64",
                s.id as "street_id?",
                s.name as "street_name?",
                a.house_number
            FROM team_assignment ta
            JOIN address a ON ta.address_id = a.id
            LEFT JOIN street s ON a.street_id = s.id
            WHERE a.area_id = $1
            ORDER BY ta.team_id ASC, a.id ASC"#,
            self.area_id
        )
        .fetch_all(&mut **conn)
        .await?;
        let mut map: std::collections::HashMap<i64, Vec<team::TeamAddress>> =
            std::collections::HashMap::new();
        for record in records {
            let entry = map.entry(record.team_id).or_default();
            entry.push(team::TeamAddress {
                address_id: record.address_id,
                street_id: record.street_id,
                street_name: record.street_name,
                house_number: record.house_number,
                _guard: (),
            });
        }
        Ok(map)
    }

    async fn set_team_bounds(&self, team: &Team, bounds: &[Point]) -> anyhow::Result<TeamBounds> {
        let mut conn = self.state.conn().await?;
        let mut tx = conn.begin().await?;
        sqlx::query!(
            r#"DELETE FROM team_bounds_vertices WHERE team_id = $1"#,
            team.id
        )
        .execute(&mut *tx)
        .await?;
        for (position, point) in bounds.iter().enumerate() {
            let position = position as i64;
            sqlx::query!(
                r#"INSERT INTO team_bounds_vertices (team_id, position, x, y) VALUES ($1, $2, $3, $4)"#,
                team.id,
                position,
                point.x,
                point.y
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(TeamBounds {
            boundary: bounds.to_vec(),
            _guard: (),
        })
    }

    async fn get_team_bounds(&self, team: &Team) -> anyhow::Result<Option<TeamBounds>> {
        let mut conn = self.state.conn().await?;
        let records = sqlx::query!(
            r#"SELECT position, x, y FROM team_bounds_vertices
            WHERE team_id = $1
            ORDER BY position ASC"#,
            team.id
        )
        .fetch_all(&mut **conn)
        .await?;
        if records.is_empty() {
            Ok(None)
        } else {
            let points = records
                .into_iter()
                .map(|record| Point {
                    x: record
                        .x
                        .try_into()
                        .expect("x coordinate bounded by database constraint"),
                    y: record
                        .y
                        .try_into()
                        .expect("y coordinate bounded by database constraint"),
                })
                .collect();
            Ok(Some(TeamBounds {
                boundary: points,
                _guard: (),
            }))
        }
    }

    async fn remove_team_bounds(&self, team: &Team) -> anyhow::Result<()> {
        let mut conn = self.state.conn().await?;
        sqlx::query!(
            r#"DELETE FROM team_bounds_vertices WHERE team_id = $1"#,
            team.id
        )
        .execute(&mut **conn)
        .await?;
        Ok(())
    }
}

impl AddressRepository for AreaDb {
    async fn get_addresses(&self) -> anyhow::Result<Vec<Address>> {
        let mut conn = self.state.conn().await?;
        Ok(sqlx::query!(
            r#"SELECT
                id as "id!: i64",
                area_id as "area_id!: i64",
                house_number,
                circle_radius as "circle_radius!: u32",
                x,
                y,
                confidence,
                verified,
                estimated_flats,
                street_id as "assigned_street_id"
            FROM address
            WHERE area_id = $1
            ORDER BY id ASC"#,
            self.area_id
        )
        .fetch_all(&mut **conn)
        .await?
        .into_iter()
        .map(|record| Address {
            id: record.id,
            area_id: record.area_id,
            house_number: record.house_number,
            circle_radius: record.circle_radius,
            position: Point {
                x: record
                    .x
                    .try_into()
                    .expect("x coordinate bounded by database constraint"),
                y: record
                    .y
                    .try_into()
                    .expect("y coordinate bounded by database constraint"),
            },
            confidence: record.confidence,
            verified: record.verified != 0,
            estimated_flats: record.estimated_flats.map(|v| v as u16),
            assigned_street_id: record.assigned_street_id,
            _guard: (),
        })
        .collect())
    }

    async fn get_address_by_id(&self, id: i64) -> anyhow::Result<Option<Address>> {
        let mut conn = self.state.conn().await?;
        if let Some(record) = sqlx::query!(
            r#"SELECT
                id as "id!: i64",
                area_id as "area_id!: i64",
                house_number,
                x,
                y,
                confidence,
                verified,
                estimated_flats,
                circle_radius as "circle_radius!: u32",
                street_id as "assigned_street_id"
            FROM address
            WHERE area_id = $1 AND id = $2"#,
            self.area_id,
            id
        )
        .fetch_optional(&mut **conn)
        .await?
        {
            Ok(Some(Address {
                id: record.id,
                area_id: record.area_id,
                house_number: record.house_number,
                position: Point {
                    x: record
                        .x
                        .try_into()
                        .expect("x coordinate bounded by database constraint"),
                    y: record
                        .y
                        .try_into()
                        .expect("y coordinate bounded by database constraint"),
                },
                confidence: record.confidence,
                verified: record.verified != 0,
                estimated_flats: record.estimated_flats.map(|v| v as u16),
                circle_radius: record.circle_radius,
                assigned_street_id: record.assigned_street_id,
                _guard: (),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_address_by_street(&self, street: &Street) -> anyhow::Result<Vec<Address>> {
        let mut conn = self.state.conn().await?;
        Ok(sqlx::query!(
            r#"SELECT
                id as "id!: i64",
                area_id as "area_id!: i64",
                house_number,
                x,
                y,
                confidence,
                verified,
                estimated_flats,
                circle_radius as "circle_radius!: u32",
                street_id as "assigned_street_id"
            FROM address
            WHERE area_id = $1 AND street_id = $2
            ORDER BY id ASC"#,
            self.area_id,
            street.id
        )
        .fetch_all(&mut **conn)
        .await?
        .into_iter()
        .map(|record| Address {
            id: record.id,
            area_id: record.area_id,
            house_number: record.house_number,
            position: Point {
                x: record
                    .x
                    .try_into()
                    .expect("x coordinate bounded by database constraint"),
                y: record
                    .y
                    .try_into()
                    .expect("y coordinate bounded by database constraint"),
            },
            confidence: record.confidence,
            verified: record.verified != 0,
            estimated_flats: record.estimated_flats.map(|v| v as u16),
            circle_radius: record.circle_radius,
            assigned_street_id: record.assigned_street_id,
            _guard: (),
        })
        .collect())
    }

    async fn add_address(&self, address: &address::NewAddress) -> anyhow::Result<Address> {
        let mut conn = self.state.conn().await?;
        let estimated_flats = address.estimated_flats.map(|v| v as i64);
        let record = sqlx::query!(
            r#"INSERT INTO address
            (area_id, house_number, x, y, confidence, circle_radius, estimated_flats, street_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id as "id!: i64",
                area_id as "area_id!: i64",
                circle_radius as "circle_radius!: u32",
                house_number,
                x,
                y,
                confidence,
                verified,
                estimated_flats,
                street_id as "assigned_street_id""#,
            self.area_id,
            address.house_number,
            address.position.x,
            address.position.y,
            address.confidence,
            address.circle_radius,
            estimated_flats,
            address.assigned_street_id
        )
        .fetch_one(&mut **conn)
        .await?;
        Ok(Address {
            id: record.id,
            area_id: record.area_id,
            house_number: record.house_number,
            position: Point {
                x: record
                    .x
                    .try_into()
                    .expect("x coordinate bounded by database constraint"),
                y: record
                    .y
                    .try_into()
                    .expect("y coordinate bounded by database constraint"),
            },
            confidence: record.confidence,
            verified: record.verified != 0,
            estimated_flats: record.estimated_flats.map(|v| v as u16),
            assigned_street_id: record.assigned_street_id,
            circle_radius: record.circle_radius,
            _guard: (),
        })
    }

    async fn update_address(
        &self,
        address: &Address,
        update: &address::AddressUpdate<'_>,
    ) -> anyhow::Result<Address> {
        let mut conn = self.state.conn().await?;
        let estimated_flats = match update.estimated_flats {
            Some(Some(v)) => Some(v as i64),
            Some(None) => None,
            None => address.estimated_flats.map(|v| v as i64),
        };
        let assigned_street_id = match update.street {
            Some(x) => x.map(|s| s.id),
            None => address.assigned_street_id,
        };
        let x = update.position.as_ref().map(|p| p.x);
        let y = update.position.as_ref().map(|p| p.y);
        let record = sqlx::query!(
            r#"UPDATE address SET
                house_number = COALESCE($1, house_number),
                x = COALESCE($2, x),
                y = COALESCE($3, y),
                confidence = COALESCE($4, confidence),
                verified = COALESCE($5, verified),
                circle_radius = COALESCE($10, circle_radius),
                estimated_flats = $6,
                street_id = $7
            WHERE id = $8 AND area_id = $9
            RETURNING
                id as "id!: i64",
                area_id as "area_id!: i64",
                house_number,
                x,
                y,
                confidence,
                verified,
                estimated_flats,
                street_id as "assigned_street_id",
                circle_radius as "circle_radius!: u32""#,
            update.house_number,
            x,
            y,
            update.confidence,
            update.verified,
            estimated_flats,
            assigned_street_id,
            address.id,
            self.area_id,
            update.circle_radius,
        )
        .fetch_one(&mut **conn)
        .await?;
        Ok(Address {
            id: record.id,
            area_id: record.area_id,
            house_number: record.house_number,
            position: Point {
                x: record
                    .x
                    .try_into()
                    .expect("x coordinate bounded by database constraint"),
                y: record
                    .y
                    .try_into()
                    .expect("y coordinate bounded by database constraint"),
            },
            confidence: record.confidence,
            verified: record.verified != 0,
            estimated_flats: record.estimated_flats.map(|v| v as u16),
            assigned_street_id: record.assigned_street_id,
            circle_radius: record.circle_radius,
            _guard: (),
        })
    }

    async fn delete_address(&self, address: Address) -> anyhow::Result<()> {
        let mut conn = self.state.conn().await?;
        sqlx::query!(
            r#"DELETE FROM address WHERE id = $1 AND area_id = $2"#,
            address.id,
            self.area_id
        )
        .execute(&mut **conn)
        .await?;
        Ok(())
    }
}

impl StreetRepository for AreaDb {
    async fn get_streets(&self) -> anyhow::Result<Vec<Street>> {
        let mut conn = self.state.conn().await?;
        Ok(sqlx::query!(
            r#"SELECT id as "id!: i64", name, verified FROM street
            WHERE area_id = $1
            ORDER BY id ASC"#,
            self.area_id
        )
        .fetch_all(&mut **conn)
        .await?
        .into_iter()
        .map(|record| Street {
            id: record.id,
            name: record.name,
            verified: record.verified != 0,
            _guard: (),
        })
        .collect())
    }

    async fn get_street_by_id(&self, id: i64) -> anyhow::Result<Option<Street>> {
        let mut conn = self.state.conn().await?;
        if let Some(record) = sqlx::query!(
            r#"SELECT id as "id!: i64", name, verified FROM street
            WHERE area_id = $1 AND id = $2"#,
            self.area_id,
            id
        )
        .fetch_optional(&mut **conn)
        .await?
        {
            Ok(Some(Street {
                id: record.id,
                name: record.name,
                verified: record.verified != 0,
                _guard: (),
            }))
        } else {
            Ok(None)
        }
    }

    async fn add_street(&self) -> anyhow::Result<Street> {
        let mut conn = self.state.conn().await?;
        let record = sqlx::query!(
            r#"INSERT INTO street (area_id) VALUES ($1)
            RETURNING id as "id!: i64", name, verified"#,
            self.area_id
        )
        .fetch_one(&mut **conn)
        .await?;
        Ok(Street {
            id: record.id,
            name: record.name,
            verified: record.verified != 0,
            _guard: (),
        })
    }

    async fn draw_street_polyline(
        &self,
        street: &Street,
        polyline: &[Point],
    ) -> anyhow::Result<()> {
        let mut conn = self.state.conn().await?;
        let mut tx = conn.begin().await?;
        sqlx::query!(
            r#"DELETE FROM street_polyline_vertices WHERE street_id = $1"#,
            street.id
        )
        .execute(&mut *tx)
        .await?;
        for (position, point) in polyline.iter().enumerate() {
            let position = position as i64;
            sqlx::query!(
                r#"INSERT INTO street_polyline_vertices (street_id, position, x, y) VALUES ($1, $2, $3, $4)"#,
                street.id,
                position,
                point.x,
                point.y
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn get_street_polyline(&self, street: &Street) -> anyhow::Result<Option<StreetPolyline>> {
        let mut conn = self.state.conn().await?;
        let records = sqlx::query!(
            r#"SELECT position, x, y FROM street_polyline_vertices
            WHERE street_id = $1
            ORDER BY position ASC"#,
            street.id
        )
        .fetch_all(&mut **conn)
        .await?;
        if records.is_empty() {
            Ok(None)
        } else {
            let points = records
                .into_iter()
                .map(|record| Point {
                    x: record
                        .x
                        .try_into()
                        .expect("x coordinate bounded by database constraint"),
                    y: record
                        .y
                        .try_into()
                        .expect("y coordinate bounded by database constraint"),
                })
                .collect();
            Ok(Some(StreetPolyline { points, _guard: () }))
        }
    }

    async fn remove_street_polyline(&self, street: &Street) -> anyhow::Result<()> {
        let mut conn = self.state.conn().await?;
        sqlx::query!(
            r#"DELETE FROM street_polyline_vertices WHERE street_id = $1"#,
            street.id
        )
        .execute(&mut **conn)
        .await?;
        Ok(())
    }

    async fn update_street(
        &self,
        street: &Street,
        update: &StreetUpdate,
    ) -> anyhow::Result<Street> {
        let mut conn = self.state.conn().await?;
        let record = sqlx::query!(
            r#"UPDATE street SET
                name = COALESCE($1, name),
                verified = COALESCE($2, verified)
            WHERE id = $3 AND area_id = $4
            RETURNING id as "id!: i64", name, verified"#,
            update.name,
            update.verified,
            street.id,
            self.area_id
        )
        .fetch_one(&mut **conn)
        .await?;
        Ok(Street {
            id: record.id,
            name: record.name,
            verified: record.verified != 0,
            _guard: (),
        })
    }

    async fn delete_street(&self, street: Street) -> anyhow::Result<()> {
        let mut conn = self.state.conn().await?;
        sqlx::query!(
            r#"DELETE FROM street WHERE id = $1 AND area_id = $2"#,
            street.id,
            self.area_id
        )
        .execute(&mut **conn)
        .await?;
        Ok(())
    }
}

impl BoundAreaRepository for AreaDb {
    async fn get_area(&self) -> anyhow::Result<Area> {
        let mut conn = self.state.conn().await?;
        if let Some(record) = sqlx::query!(
            r#"SELECT id as "id!: i64", name, color, state FROM area WHERE id = $1"#,
            self.area_id
        )
        .fetch_optional(&mut **conn)
        .await?
        {
            let color = Color::try_from(record.color)?;
            let state = AreaState::try_from(record.state)?;
            Ok(Area {
                id: record.id,
                name: record.name,
                color,
                state,
                _guard: (),
            })
        } else {
            Err(anyhow::anyhow!("Area with id {} not found", self.area_id))
        }
    }

    async fn update_area(&self, update: &area::AreaUpdate) -> anyhow::Result<Area> {
        let mut conn = self.state.conn().await?;
        let color = update.color.map(i64::from);
        let state = update.state.map(i64::from);
        let record = sqlx::query!(
            r#"UPDATE area SET
                name = COALESCE($1, name),
                color = COALESCE($2, color),
                state = COALESCE($3, state)
            WHERE id = $4
            RETURNING id as "id!: i64", name, color, state"#,
            update.name,
            color,
            state,
            self.area_id
        )
        .fetch_one(&mut **conn)
        .await?;
        let color = Color::try_from(record.color)?;
        let state = AreaState::try_from(record.state)?;
        Ok(Area {
            id: record.id,
            name: record.name,
            color,
            state,
            _guard: (),
        })
    }

    fn get_image(&self) -> &DynamicImage {
        &self.image
    }

    async fn delete(self) -> anyhow::Result<()> {
        let mut conn = self.state.conn().await?;
        sqlx::query!(r#"DELETE FROM area WHERE id = $1"#, self.area_id)
            .execute(&mut **conn)
            .await?;
        Ok(())
    }
}
