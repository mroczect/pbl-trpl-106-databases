#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::types::core::{
    ColumnDef, Constraint, ConstraintKind, DataType, Index, Table, TableChange,
};

#[test]
fn datatype_variants_exist() {
    let types = [
        DataType::Int,
        DataType::BigInt,
        DataType::Varchar(255),
        DataType::Text,
        DataType::Boolean,
        DataType::Timestamp,
    ];
    assert_eq!(types.len(), 6);
}

#[test]
fn column_def_builder() {
    let c = ColumnDef::new("id", DataType::BigInt)
        .primary_key()
        .auto_increment();
    assert!(c.primary_key);
    assert!(c.auto_increment);
    assert!(!c.nullable);
}

#[test]
fn column_def_nullable() {
    let c = ColumnDef::new("bio", DataType::Text).nullable(true);
    assert!(c.nullable);
}

#[test]
fn column_def_unique() {
    let c = ColumnDef::new("email", DataType::Varchar(255)).unique();
    assert!(c.unique);
}

#[test]
fn index_new_and_unique() {
    let i = Index::new("idx_name", vec!["name".into()]).unique();
    assert_eq!(i.name, "idx_name");
    assert_eq!(i.columns.len(), 1);
    assert!(i.unique);
}

#[test]
fn constraint_new_fk() {
    let c = Constraint::new(
        "fk_user",
        ConstraintKind::ForeignKey {
            table: "users".into(),
            column: "id".into(),
        },
    );
    assert_eq!(c.name, "fk_user");
}

#[test]
fn table_new_and_builder() {
    let cols = vec![ColumnDef::new("id", DataType::BigInt).primary_key()];
    let t = Table::new("users", cols)
        .with_index(Index::new("idx", vec!["id".into()]))
        .with_constraint(Constraint::new("pk", ConstraintKind::PrimaryKey));
    assert_eq!(t.name, "users");
    assert_eq!(t.columns.len(), 1);
    assert_eq!(t.indexes.len(), 1);
    assert_eq!(t.constraints.len(), 1);
}

#[test]
fn table_change_variants() {
    let variants = [
        TableChange::AddColumn(ColumnDef::new("x", DataType::Int)),
        TableChange::DropColumn("x".into()),
        TableChange::RenameColumn {
            from: "a".into(),
            to: "b".into(),
        },
        TableChange::AddIndex(Index::new("i", vec![])),
        TableChange::DropIndex("i".into()),
    ];
    assert_eq!(variants.len(), 5);
}
