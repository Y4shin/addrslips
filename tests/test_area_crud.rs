//! Integration tests for Area CRUD operations.
//!
//! Tests cover:
//! - Creating areas with images
//! - Retrieving areas by ID and listing all areas
//! - Updating area metadata (state)
//! - Deleting areas
//! - Area persistence through save/load cycles

mod common;

// Import traits to bring methods into scope
use addrslips::core::db::{AreaRepository, BoundAreaRepository};

use common::*;

#[tokio::test]
async fn test_create_and_retrieve_area() -> anyhow::Result<()> {
    // 1. Create test project
    let (project, _temp_dir) = create_test_project().await;

    // 2. Create test image and area
    let (new_area, _img_file) = make_new_area("Test Area", TEST_RED);

    // 3. Add area to project
    let area_repo = project.add_area(new_area).await?;
    let area = area_repo.get_area().await?;

    // 4. Verify area has correct properties
    assert!(area.id > 0, "Area should have positive ID");
    assert_eq!(area.name, "Test Area");
    assert_eq!(area.color.r, 255);
    assert_eq!(area.color.g, 0);
    assert_eq!(area.color.b, 0);
    assert!(matches!(area.state, AreaState::Imported));

    // 5. Get all areas and verify count
    let areas: Vec<Area> = project.get_areas().await?;
    assert_eq!(areas.len(), 1);
    assert_eq!(areas[0].name, "Test Area");

    Ok(())
}

#[tokio::test]
async fn test_update_area_state() -> anyhow::Result<()> {
    // 1. Create area in Imported state
    let (project, _temp_dir) = create_test_project().await;
    let (new_area, _img_file) = make_new_area("Test Area", TEST_BLUE);
    let area_repo = project.add_area(new_area).await?;

    let area_before = area_repo.get_area().await?;
    assert!(matches!(area_before.state, AreaState::Imported));

    // 2. Update state to AddressesDetected
    let update = AreaUpdate {
        name: None,
        color: None,
        state: Some(AreaState::AddressesDetected),
    };
    let updated_area = area_repo.update_area(&update).await?;

    // 3. Verify state changed
    assert!(matches!(updated_area.state, AreaState::AddressesDetected));
    assert_eq!(updated_area.name, "Test Area"); // Other fields unchanged
    assert_eq!(updated_area.id, area_before.id);

    Ok(())
}

#[tokio::test]
async fn test_delete_area() -> anyhow::Result<()> {
    // 1. Create area
    let (project, _temp_dir) = create_test_project().await;
    let (new_area, _img_file) = make_new_area("Area to Delete", TEST_GREEN);
    let area_repo = project.add_area(new_area).await?;
    let area_id = area_repo.get_area().await?.id;

    // Verify area exists
    let areas_before: Vec<Area> = project.get_areas().await?;
    assert_eq!(areas_before.len(), 1);

    // 2. Delete area
    area_repo.delete().await?;

    // 3. Verify area no longer in list
    let areas_after: Vec<Area> = project.get_areas().await?;
    assert_eq!(areas_after.len(), 0);

    // Attempting to get deleted area should fail
    let result: anyhow::Result<AreaDb> = project.get_area_repo(area_id).await;
    assert!(result.is_err(), "Getting deleted area should fail");

    Ok(())
}

#[tokio::test]
async fn test_area_persists_after_save() -> anyhow::Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let project_path = temp_dir.path().join("persist_test.addrslips");

    // 1. Create project and add area
    {
        let project: ProjectDb = ProjectDb::new(&project_path).await?;
        let (new_area, _img_file) = make_new_area("Persistent Area", TEST_RED);
        let _area_repo: AreaDb = project.add_area(new_area).await?;

        // Verify area exists
        let areas: Vec<Area> = project.get_areas().await?;
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].name, "Persistent Area");

        // Explicitly save before dropping (required in async context)
        project.save_project().await?;
    } // Drop project

    // 2. Reopen project from same path
    {
        let project: ProjectDb = ProjectDb::new(&project_path).await?;

        // 3. Verify area still exists with correct data
        let areas: Vec<Area> = project.get_areas().await?;
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].name, "Persistent Area");
        assert_eq!(areas[0].color.r, 255);
        assert!(matches!(areas[0].state, AreaState::Imported));

        // 4. Verify image is still accessible
        let area_repo: AreaDb = project.get_area_repo(areas[0].id).await?;
        let image = area_repo.get_image();
        assert_eq!(image.width(), 100);
        assert_eq!(image.height(), 100);
    }

    Ok(())
}
