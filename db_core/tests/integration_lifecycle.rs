#![allow(
    unused_crate_dependencies,
    clippy::unwrap_used,
    clippy::expect_used,
    unused_results
)]

use db_core::Database;
use db_handler::{
    json::from_json, Connection, DatabaseInfo, DatabaseInfoProvider, HealthCheck, HealthDetails,
    HealthStatus, Json,
};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{Executor, MySqlPool};

const TEST_DB: &str = "pbl_test_lifecycle";
const ROOT_URL: &str = "mysql://root:root@127.0.0.1:3307";

fn test_db_url() -> String {
    format!("{ROOT_URL}/{TEST_DB}")
}

async fn admin_pool() -> MySqlPool {
    MySqlPoolOptions::new()
        .max_connections(1)
        .connect(ROOT_URL)
        .await
        .expect("admin connect")
}

async fn db_exists(admin: &MySqlPool, name: &str) -> bool {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM information_schema.schemata WHERE schema_name = ?")
            .bind(name)
            .fetch_one(admin)
            .await
            .expect("info_schema schemata");
    row.0 == 1
}

async fn table_exists(admin: &MySqlPool, db: &str, table: &str) -> bool {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables \
         WHERE table_schema = ? AND table_name = ?",
    )
    .bind(db)
    .bind(table)
    .fetch_one(admin)
    .await
    .expect("info_schema tables");
    row.0 == 1
}

#[tokio::test]
async fn full_lifecycle_with_traits() {
    let admin = admin_pool().await;

    // [0] pre-cleanup
    admin
        .execute(format!("DROP DATABASE IF EXISTS {TEST_DB}").as_str())
        .await
        .expect("pre-cleanup");
    assert!(!db_exists(&admin, TEST_DB).await);
    println!("[0] pre-cleanup OK");

    // [1] CREATE DATABASE
    admin
        .execute(format!("CREATE DATABASE {TEST_DB}").as_str())
        .await
        .expect("create db");
    assert!(db_exists(&admin, TEST_DB).await);
    println!("[1] CREATE DATABASE OK");

    // [2] Database::connect + Connection::ping
    let db = Database::connect(&test_db_url()).await.expect("connect");
    db.ping().await.expect("ping");
    println!("[2] Database::connect + Connection::ping OK");

    // [3] CREATE TABLE
    sqlx::query(
        r"
        CREATE TABLE users (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL UNIQUE
        )
        ",
    )
    .execute(db.pool())
    .await
    .expect("create table");
    assert!(table_exists(&admin, TEST_DB, "users").await);
    println!("[3] CREATE TABLE OK");

    // [4] HealthCheck::health
    let health: HealthDetails = from_json(&db.health().await.expect("health")).expect("parse");
    assert_eq!(health.status, HealthStatus::Healthy);
    assert!(!health.server_version.is_empty());
    println!(
        "[4] HealthCheck::health OK — latency={} ms, version={}",
        health.latency_ms, health.server_version
    );

    // [5] DatabaseInfoProvider::info
    let info: DatabaseInfo = from_json(&db.info().await.expect("info")).expect("parse");
    assert_eq!(info.database_name, TEST_DB);
    assert!(!info.server_version.is_empty());
    println!(
        "[5] DatabaseInfoProvider::info OK — db={}, pool_size={}, idle={}",
        info.database_name, info.pool_size, info.pool_idle
    );

    // [6] Connection::version
    let version_json = db.version().await.expect("version");
    let version = version_json
        .get("version")
        .and_then(Json::as_str)
        .expect("version field");
    assert!(!version.is_empty());
    println!("[6] Connection::version OK — {version}");

    // [7] Transaction::rollback
    db.begin()
        .await
        .expect("begin")
        .rollback()
        .await
        .expect("rollback");
    println!("[7] Transaction::rollback OK");

    // [8] Transaction::commit
    db.begin()
        .await
        .expect("begin")
        .commit()
        .await
        .expect("commit");
    println!("[8] Transaction::commit OK");

    // [9] INSERT + COUNT
    sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
        .bind("Alice")
        .bind("alice@example.com")
        .execute(db.pool())
        .await
        .expect("insert");
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(db.pool())
        .await
        .expect("count");
    assert_eq!(count, 1);
    println!("[9] INSERT + COUNT OK — {count}");

    // [10] UNIQUE constraint
    let dup = sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
        .bind("Bob")
        .bind("alice@example.com")
        .execute(db.pool())
        .await;
    assert!(dup.is_err());
    println!("[10] UNIQUE constraint OK");

    // [11] DROP TABLE
    sqlx::query("DROP TABLE users")
        .execute(db.pool())
        .await
        .expect("drop table");
    assert!(!table_exists(&admin, TEST_DB, "users").await);
    println!("[11] DROP TABLE OK");

    // [12] Close pool sebelum DROP DATABASE (wajib — MySQL menolak drop jika masih ada koneksi aktif)
    db.close().await;
    println!("[12] Database closed OK");

    // [13] DROP DATABASE
    admin
        .execute(format!("DROP DATABASE {TEST_DB}").as_str())
        .await
        .expect("drop db");
    assert!(!db_exists(&admin, TEST_DB).await);
    println!("[13] DROP DATABASE OK");

    // [14] Close admin pool
    admin.close().await;
    println!("[14] admin closed OK");
}
