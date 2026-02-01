//! Integration tests for Address CRUD operations.
//!
//! Tests cover:
//! - Adding addresses with and without street assignments
//! - Querying addresses by ID and by street
//! - Updating address fields (verified flag, estimated flats)
//! - Deleting addresses

mod common;

use common::*;

#[tokio::test]
async fn test_add_address_without_street() -> anyhow::Result<()> {
    // 1. Create area
    let (project, _temp_dir) = create_test_project().await;
    let (new_area, _img_file) = make_new_area("Test Area", TEST_RED);
    let area_repo = project.add_area(new_area).await?;

    // 2. Add address with assigned_street_id = None
    let new_address = make_test_address("42", 100, 200);
    let address = AddressRepository::add_address(&area_repo, &new_address).await?;

    // 3. Verify address created with correct properties
    assert!(address.id > 0);
    assert_eq!(address.house_number, "42");
    assert_eq!(address.position.x, 100);
    assert_eq!(address.position.y, 200);
    assert_eq!(address.confidence, 0.95);
    assert_eq!(address.verified, false);
    assert_eq!(address.estimated_flats, Some(4));
    assert_eq!(address.assigned_street_id, None);

    Ok(())
}

#[tokio::test]
async fn test_add_address_with_street() -> anyhow::Result<()> {
    // 1. Create area and street
    let (project, _temp_dir) = create_test_project().await;
    let (new_area, _img_file) = make_new_area("Test Area", TEST_BLUE);
    let area_repo = project.add_area(new_area).await?;
    let street = area_repo.add_street().await?;

    // 2. Add address with assigned_street_id = Some(street.id)
    let mut new_address = make_test_address("123", 300, 400);
    new_address.assigned_street_id = Some(street.id);
    let address = AddressRepository::add_address(&area_repo, &new_address).await?;

    // 3. Verify foreign key accepted
    assert_eq!(address.assigned_street_id, Some(street.id));

    // 4. Query addresses by street, verify list contains address
    let street_addresses = area_repo.get_address_by_street(&street).await?;
    assert_eq!(street_addresses.len(), 1);
    assert_eq!(street_addresses[0].id, address.id);
    assert_eq!(street_addresses[0].house_number, "123");

    Ok(())
}

#[tokio::test]
async fn test_update_address_verified_flag() -> anyhow::Result<()> {
    // 1. Create address with verified = false
    let (project, _temp_dir) = create_test_project().await;
    let (new_area, _img_file) = make_new_area("Test Area", TEST_GREEN);
    let area_repo = project.add_area(new_area).await?;
    let new_address = make_test_address("99", 500, 600);
    let address = AddressRepository::add_address(&area_repo, &new_address).await?;

    assert_eq!(address.verified, false);

    // 2. Update with AddressUpdate { verified: Some(true), .. }
    let update = AddressUpdate {
        verified: Some(true),
        ..Default::default()
    };
    let updated: Address = area_repo.update_address(&address, &update).await?;

    // 3. Verify update returned updated Address
    assert_eq!(updated.verified, true);
    assert_eq!(updated.id, address.id);
    assert_eq!(updated.house_number, "99"); // Other fields unchanged

    // 4. Get address by ID, verify verified = true persisted
    let reloaded: Option<Address> = area_repo.get_address_by_id(address.id).await?;
    let reloaded = reloaded.expect("Address should exist");
    assert_eq!(reloaded.verified, true);

    Ok(())
}

#[tokio::test]
async fn test_delete_address() -> anyhow::Result<()> {
    // 1. Add address
    let (project, _temp_dir): (ProjectDb, _) = create_test_project().await;
    let (new_area, _img_file) = make_new_area("Test Area", TEST_RED);
    let area_repo: AreaDb = project.add_area(new_area).await?;
    let new_address = make_test_address("77", 700, 800);
    let address: Address = AddressRepository::add_address(&area_repo, &new_address).await?;
    let address_id = address.id;

    // Verify address exists
    let all_addresses: Vec<Address> = area_repo.get_addresses().await?;
    assert_eq!(all_addresses.len(), 1);

    // 2. Delete via delete_address()
    area_repo.delete_address(address).await?;

    // 3. Verify get_address_by_id returns None
    let result: Option<Address> = area_repo.get_address_by_id(address_id).await?;
    assert!(result.is_none(), "Address should no longer exist");

    let all_addresses: Vec<Address> = area_repo.get_addresses().await?;
    assert_eq!(all_addresses.len(), 0);

    Ok(())
}
