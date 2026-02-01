use time::OffsetDateTime;

use crate::core::db::AreaRepository;

pub struct UpdateProjectSettings {
    pub name: Option<String>,
    pub target_address_count: Option<u64>,
    pub created_at: Option<OffsetDateTime>,
}

pub trait ProjectRepository: AreaRepository {
    fn get_project_name(&self) -> impl Future<Output = anyhow::Result<String>>;
    fn get_project_created_at(&self) -> impl Future<Output = anyhow::Result<OffsetDateTime>>;
    fn get_target_address_count(&self) -> impl Future<Output = anyhow::Result<u64>>;
    fn set_project_settings(&self, settings: UpdateProjectSettings) -> impl Future<Output = anyhow::Result<()>>;
}