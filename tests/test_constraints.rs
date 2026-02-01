//! Integration tests for database constraint enforcement.
//!
//! Tests cover:
//! - Team and address must be in same area (foreign key constraint)
//! - Each address can only belong to one team (unique constraint)

mod common;

// Import traits to bring methods into scope
use addrslips::core::db::{AddressRepository, AreaRepository, StreetRepository, TeamRepository};

use common::*;

#[tokio::test]
async fn test_team_assignment_enforces_same_area() -> anyhow::Result<()> {
    // 1. Create area1 with street1 and address1
    let (project, _temp_dir) = create_test_project().await;
    let (new_area1, _img_file1) = make_new_area("Area 1", TEST_RED);
    let area1_repo = project.add_area(new_area1).await?;
    let street1 = area1_repo.add_street().await?;

    let mut new_address1 = make_test_address("42", 100, 200);
    new_address1.assigned_street_id = Some(street1.id);
    let address1: Address = AddressRepository::add_address(&area1_repo, &new_address1).await?;

    // 2. Create area2 with team2
    let (new_area2, _img_file2) = make_new_area("Area 2", TEST_BLUE);
    let area2_repo = project.add_area(new_area2).await?;
    let team2 = area2_repo.add_team().await?;

    // 3. Attempt to assign address1 (from area1) to team2 (from area2)
    let result: anyhow::Result<()> =
        TeamRepository::add_address(&area2_repo, &team2, &address1).await;

    // 4. Assert error contains "FOREIGN KEY constraint failed"
    assert!(
        result.is_err(),
        "Should fail to assign address from different area"
    );
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("FOREIGN KEY constraint failed")
            || error_msg.contains("FOREIGN KEY")
            || error_msg.contains("foreign key"),
        "Error should mention foreign key constraint, got: {}",
        error_msg
    );

    Ok(())
}

#[tokio::test]
async fn test_address_can_only_belong_to_one_team() -> anyhow::Result<()> {
    // 1. Create area, street, and address
    let (project, _temp_dir): (ProjectDb, _) = create_test_project().await;
    let (new_area, _img_file) = make_new_area("Test Area", TEST_GREEN);
    let area_repo: AreaDb = project.add_area(new_area).await?;
    let street: Street = area_repo.add_street().await?;

    let mut new_address = make_test_address("99", 300, 400);
    new_address.assigned_street_id = Some(street.id);
    let address: Address = AddressRepository::add_address(&area_repo, &new_address).await?;

    // 2. Create team1 and assign address
    let team1: Team = area_repo.add_team().await?;
    TeamRepository::add_address(&area_repo, &team1, &address).await?;

    // Verify assignment succeeded
    let team1_addresses: Vec<TeamAddress> = area_repo.get_team_addresses(&team1).await?;
    assert_eq!(team1_addresses.len(), 1);

    // 3. Create team2 and attempt to assign same address
    let team2: Team = area_repo.add_team().await?;
    let result: anyhow::Result<()> =
        TeamRepository::add_address(&area_repo, &team2, &address).await;

    // 4. Assert unique constraint violation error
    assert!(
        result.is_err(),
        "Should fail to assign address to second team"
    );
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("UNIQUE")
            || error_msg.contains("unique")
            || error_msg.contains("constraint"),
        "Error should mention unique constraint, got: {}",
        error_msg
    );

    Ok(())
}
