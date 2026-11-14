#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::types::core::{Migration, MigrationState, MigrationStatus};

#[test]
fn migration_new() {
    let m = Migration::new(1, "init", "CREATE TABLE t (id INT)");
    assert_eq!(m.version, 1);
    assert_eq!(m.name, "init");
    assert_eq!(m.up_sql, "CREATE TABLE t (id INT)");
    assert!(m.down_sql.is_none());
}

#[test]
fn migration_with_down() {
    let m = Migration::new(1, "init", "UP").with_down("DOWN");
    assert_eq!(m.down_sql.as_deref(), Some("DOWN"));
}

#[test]
fn migration_state_variants() {
    assert_ne!(MigrationState::Pending, MigrationState::Applied);
    assert_ne!(MigrationState::Applied, MigrationState::Failed);
}

#[test]
fn migration_status_new() {
    let m = Migration::new(1, "init", "SQL");
    let s = MigrationStatus::new(m, MigrationState::Pending, None);
    assert_eq!(s.state, MigrationState::Pending);
    assert!(s.applied_at.is_none());
}
