use addrslips::core::db::AreaRepository;
use addrslips::core::db::ProjectDb;

#[tokio::test]
async fn test_simple() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    let path = dir.path().join("test.addrslips");
    let project: ProjectDb = ProjectDb::new(&path).await?;

    let areas = project.get_areas().await?;
    assert_eq!(areas.len(), 0);

    Ok(())
}
