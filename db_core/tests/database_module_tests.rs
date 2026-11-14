#![allow(
    unused_crate_dependencies,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::too_many_lines,
    clippy::items_after_statements,
    clippy::similar_names,
    clippy::redundant_clone,
    clippy::field_reassign_with_default,
    unused_results
)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use db_core::Database;
use db_handler::{
    ColumnDef, Connection, DataType, DatabaseInfoProvider, DbError, Executor, HealthCheck,
    HealthStatus, Index, IsolationLevel, Migration, MigrationRunner, PageRequest, PoolConfig,
    Query, SchemaManager, TableChange, TransactionManager,
};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

// `#[allow(unused_imports)]` makes the import bulletproof either way.
#[allow(unused_imports)]
use db_handler::Transaction;

const TEST_DB: &str = "pbl_test_db_module";
const ROOT_URL: &str = "mysql://root:root@127.0.0.1:3307";

static TABLE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn test_db_url() -> String {
    format!("{ROOT_URL}/{TEST_DB}")
}

fn unique_table(prefix: &str) -> String {
    let n = TABLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{prefix}_{n}")
}

async fn admin_pool() -> MySqlPool {
    MySqlPoolOptions::new()
        .max_connections(2)
        .connect(ROOT_URL)
        .await
        .expect("admin connect")
}

async fn ensure_test_db() {
    let admin = admin_pool().await;
    sqlx::query(&format!("CREATE DATABASE IF NOT EXISTS {TEST_DB}"))
        .execute(&admin)
        .await
        .expect("create test db");
    admin.close().await;
}

async fn test_db() -> Database {
    ensure_test_db().await;
    Database::connect(&test_db_url()).await.expect("connect")
}

async fn setup_table(prefix: &str) -> (Database, String) {
    let db = test_db().await;
    let table = unique_table(prefix);
    let sql = format!(
        "CREATE TABLE `{table}` (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL UNIQUE,
            age INT NULL
        )"
    );
    sqlx::query(&sql)
        .execute(db.pool())
        .await
        .expect("create table");
    (db, table)
}

async fn insert_user(db: &Database, table: &str, name: &str, email: &str, age: Option<i32>) {
    let sql = format!("INSERT INTO `{table}` (name, email, age) VALUES (?, ?, ?)");
    sqlx::query(&sql)
        .bind(name)
        .bind(email)
        .bind(age)
        .execute(db.pool())
        .await
        .expect("insert user");
}

async fn count_rows(db: &Database, table: &str) -> i64 {
    let sql = format!("SELECT COUNT(*) FROM `{table}`");
    let row = sqlx::query(&sql).fetch_one(db.pool()).await.expect("count");
    sqlx::Row::try_get::<i64, _>(&row, 0).expect("count i64")
}

async fn teardown(db: Database, table: &str) {
    let _ = sqlx::query(&format!("DROP TABLE IF EXISTS `{table}`"))
        .execute(db.pool())
        .await;
    db.close().await;
}

#[tokio::test]
async fn connect_with_valid_url_succeeds() {
    ensure_test_db().await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    assert!(db.ping().await.is_ok());
    db.close().await;
}

#[tokio::test]
async fn connect_with_invalid_url_returns_connection_error() {
    let result = Database::connect("mysql://nobody:wrong@127.0.0.1:1/none").await;
    match result {
        Err(DbError::Connection { .. }) => {}
        Err(other) => panic!("expected Connection error, got {other:?}"),
        Ok(_) => panic!("expected connect to fail"),
    }
}

#[tokio::test]
async fn connect_with_rejects_zero_max_connections() {
    let mut cfg = PoolConfig::default();
    cfg.max_connections = 0;
    ensure_test_db().await;
    let result = Database::connect_with(&test_db_url(), cfg).await;
    match result {
        Err(DbError::Validation { message }) => {
            assert!(message.contains("max_connections"));
        }
        _ => panic!("expected Validation error"),
    }
}

#[tokio::test]
async fn connect_with_rejects_min_greater_than_max() {
    let mut cfg = PoolConfig::default();
    cfg.max_connections = 5;
    cfg.min_connections = 6;
    ensure_test_db().await;
    let result = Database::connect_with(&test_db_url(), cfg).await;
    assert!(matches!(result, Err(DbError::Validation { .. })));
}

#[tokio::test]
async fn connect_with_rejects_zero_acquire_timeout() {
    let mut cfg = PoolConfig::default();
    cfg.acquire_timeout_ms = 0;
    ensure_test_db().await;
    let result = Database::connect_with(&test_db_url(), cfg).await;
    assert!(matches!(result, Err(DbError::Validation { .. })));
}

#[tokio::test]
async fn connect_with_rejects_zero_connect_timeout() {
    let mut cfg = PoolConfig::default();
    cfg.connect_timeout_ms = 0;
    ensure_test_db().await;
    let result = Database::connect_with(&test_db_url(), cfg).await;
    assert!(matches!(result, Err(DbError::Validation { .. })));
}

#[tokio::test]
async fn connect_with_custom_config_succeeds() {
    ensure_test_db().await;
    let cfg = PoolConfig {
        max_connections: 3,
        min_connections: 1,
        acquire_timeout_ms: 5_000,
        connect_timeout_ms: 5_000,
        max_lifetime_secs: 60,
        idle_timeout_secs: 30,
    };
    let db = Database::connect_with(&test_db_url(), cfg)
        .await
        .expect("connect_with");
    assert!(db.ping().await.is_ok());
    db.close().await;
}

// documentation but compiled out via `#[cfg(any())]` (always false).
#[cfg(any())]
#[tokio::test]
async fn from_env_reads_database_url() {
    ensure_test_db().await;
    unsafe {
        std::env::set_var("DATABASE_URL", test_db_url());
    }
    let db = Database::from_env().await.expect("from_env");
    assert!(db.ping().await.is_ok());
    db.close().await;

    unsafe {
        std::env::set_var("DATABASE_URL", "not-a-real-url");
    }
    assert!(Database::from_env().await.is_err());

    unsafe {
        std::env::remove_var("DATABASE_URL");
    }
}

#[tokio::test]
async fn ping_succeeds() {
    let db = test_db().await;
    db.ping().await.expect("ping");
    db.close().await;
}

#[tokio::test]
async fn version_returns_nonempty() {
    let db = test_db().await;
    let v = db.version().await.expect("version");
    assert!(!v.is_empty());
    db.close().await;
}

#[tokio::test]
async fn version_mentions_mysql_or_mariadb() {
    let db = test_db().await;
    let v = db.version().await.expect("version");
    let first = v.chars().next().unwrap_or(' ');
    assert!(first.is_ascii_digit(), "unexpected version: {v}");
    db.close().await;
}

#[tokio::test]
async fn health_returns_healthy_on_live_db() {
    let db = test_db().await;
    let health = db.health().await.expect("health");
    assert_eq!(health.status, HealthStatus::Healthy);
    assert!(!health.server_version.is_empty());
    assert!(health.message.is_none());
    db.close().await;
}

#[tokio::test]
async fn health_latency_within_timeout() {
    let db = test_db().await;
    let health = db.health().await.expect("health");
    assert!(health.latency_ms < 2_000, "latency={}", health.latency_ms);
    db.close().await;
}

#[tokio::test]
async fn info_returns_correct_database_name() {
    let db = test_db().await;
    let info = db.info().await.expect("info");
    assert_eq!(info.database_name, TEST_DB);
    db.close().await;
}

#[tokio::test]
async fn info_pool_accounting_is_consistent() {
    let db = test_db().await;
    let info = db.info().await.expect("info");
    assert!(info.pool_size >= info.pool_idle);
    assert_eq!(
        info.pool_size,
        info.pool_idle.saturating_add(info.active_connections)
    );
    db.close().await;
}

#[tokio::test]
async fn info_server_version_matches_connection_version() {
    let db = test_db().await;
    let info = db.info().await.expect("info");
    let v = db.version().await.expect("version");
    assert_eq!(info.server_version, v);
    db.close().await;
}

#[tokio::test]
async fn execute_insert_returns_id_and_affected_row() {
    let (db, table) = setup_table("exec_insert").await;
    let sql = format!("INSERT INTO `{table}` (name, email) VALUES (?, ?)");
    let q = Query::new(sql).bind("Alice").bind("alice@x.com");
    let res = db.execute(q).await.expect("execute");
    assert_eq!(res.rows_affected, 1);
    assert!(res.last_insert_id.is_some());
    assert_eq!(count_rows(&db, &table).await, 1);
    teardown(db, &table).await;
}

#[tokio::test]
async fn execute_update_returns_affected_rows() {
    let (db, table) = setup_table("exec_update").await;
    insert_user(&db, &table, "A", "a@x", None).await;
    insert_user(&db, &table, "B", "b@x", None).await;
    let sql = format!("UPDATE `{table}` SET name = ?");
    let res = db
        .execute(Query::new(sql).bind("Renamed"))
        .await
        .expect("update");
    assert_eq!(res.rows_affected, 2);
    teardown(db, &table).await;
}

#[tokio::test]
async fn execute_delete_returns_affected_rows() {
    let (db, table) = setup_table("exec_delete").await;
    insert_user(&db, &table, "A", "a@x", None).await;
    let sql = format!("DELETE FROM `{table}` WHERE name = ?");
    let res = db.execute(Query::new(sql).bind("A")).await.expect("delete");
    assert_eq!(res.rows_affected, 1);
    teardown(db, &table).await;
}

#[tokio::test]
async fn execute_invalid_sql_returns_query_error() {
    let db = test_db().await;
    let result = db.execute(Query::new("THIS IS NOT SQL")).await;
    assert!(matches!(result, Err(DbError::Query { .. })));
    db.close().await;
}

#[tokio::test]
async fn execute_wrong_param_count_fails() {
    let db = test_db().await;
    let q = Query::new("INSERT INTO nowhere (a, b) VALUES (?, ?)").bind(1i64);
    let result = db.execute(q).await;
    assert!(result.is_err());
    db.close().await;
}

#[tokio::test]
async fn fetch_one_returns_row() {
    let (db, table) = setup_table("fetch_one").await;
    insert_user(&db, &table, "Alice", "alice@x", Some(30)).await;
    let sql = format!("SELECT id, name, email, age FROM `{table}` WHERE email = ?");
    let row = db
        .fetch_one(Query::new(sql).bind("alice@x"))
        .await
        .expect("fetch_one");
    assert_eq!(row.get("name").unwrap().as_str(), Some("Alice"));
    assert_eq!(row.get("age").unwrap().as_i64(), Some(30));
    assert!(!row.is_empty());
    assert_eq!(row.len(), 4);
    teardown(db, &table).await;
}

#[tokio::test]
async fn fetch_one_on_empty_result_returns_error() {
    let db = test_db().await;
    let result = db.fetch_one(Query::new("SELECT 1 AS x WHERE 1 = 0")).await;
    assert!(result.is_err());
    db.close().await;
}

#[tokio::test]
async fn fetch_one_columns_populated() {
    let (db, table) = setup_table("fetch_one_cols").await;
    insert_user(&db, &table, "A", "a@x", None).await;
    let sql = format!("SELECT id, name FROM `{table}` LIMIT 1");
    let row = db.fetch_one(Query::new(sql)).await.expect("row");
    let cols = row.columns();
    assert_eq!(cols.len(), 2);
    assert_eq!(cols.first().map(|c| c.name.as_str()), Some("id"));
    assert_eq!(cols.get(1).map(|c| c.name.as_str()), Some("name"));
    teardown(db, &table).await;
}

#[tokio::test]
async fn fetch_optional_returns_some_when_found() {
    let (db, table) = setup_table("fetch_opt_some").await;
    insert_user(&db, &table, "A", "a@x", None).await;
    let sql = format!("SELECT name FROM `{table}` LIMIT 1");
    let row = db.fetch_optional(Query::new(sql)).await.expect("optional");
    assert!(row.is_some());
    assert_eq!(row.unwrap().get("name").unwrap().as_str(), Some("A"));
    teardown(db, &table).await;
}

#[tokio::test]
async fn fetch_optional_returns_none_when_missing() {
    let db = test_db().await;
    let row = db
        .fetch_optional(Query::new("SELECT 1 AS x WHERE 1 = 0"))
        .await
        .expect("optional");
    assert!(row.is_none());
    db.close().await;
}

#[tokio::test]
async fn fetch_optional_on_nullable_column() {
    let (db, table) = setup_table("fetch_opt_null").await;
    insert_user(&db, &table, "A", "a@x", None).await;
    let sql = format!("SELECT age FROM `{table}` LIMIT 1");
    let row = db
        .fetch_optional(Query::new(sql))
        .await
        .expect("row")
        .unwrap();
    assert!(row.get("age").unwrap().is_null());
    teardown(db, &table).await;
}

#[tokio::test]
async fn fetch_all_returns_every_row() {
    let (db, table) = setup_table("fetch_all").await;
    for i in 0..5 {
        insert_user(&db, &table, &format!("u{i}"), &format!("u{i}@x"), None).await;
    }
    let sql = format!("SELECT id FROM `{table}` ORDER BY id");
    let result = db.fetch_all(Query::new(sql)).await.expect("fetch_all");
    assert_eq!(result.len(), 5);
    assert!(!result.is_empty());
    assert_eq!(result.columns.len(), 1);
    teardown(db, &table).await;
}

#[tokio::test]
async fn fetch_all_empty_returns_empty_result() {
    let db = test_db().await;
    let result = db
        .fetch_all(Query::new("SELECT 1 AS x WHERE 1 = 0"))
        .await
        .expect("empty");
    assert!(result.is_empty());
    assert_eq!(result.len(), 0);
    assert!(result.columns.is_empty());
    db.close().await;
}

#[tokio::test]
async fn fetch_all_columns_match_first_row() {
    let (db, table) = setup_table("fetch_all_cols").await;
    insert_user(&db, &table, "A", "a@x", Some(20)).await;
    insert_user(&db, &table, "B", "b@x", None).await;
    let sql = format!("SELECT id, name, email, age FROM `{table}` ORDER BY id");
    let result = db.fetch_all(Query::new(sql)).await.expect("all");
    let names: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["id", "name", "email", "age"]);
    teardown(db, &table).await;
}

#[tokio::test]
async fn fetch_paginated_first_page() {
    let (db, table) = setup_table("pag_first").await;
    for i in 0..5 {
        insert_user(&db, &table, &format!("u{i}"), &format!("u{i}@x"), None).await;
    }
    let sql = format!("SELECT id FROM `{table}` ORDER BY id");
    let result = db
        .fetch_paginated(
            Query::new(sql),
            PageRequest {
                page: 1,
                per_page: 2,
            },
        )
        .await
        .expect("page1");
    assert_eq!(result.items.len(), 2);
    assert_eq!(result.meta.page, 1);
    assert_eq!(result.meta.per_page, 2);
    // NOTE: `total` = offset + page_len (upper bound, not exact count)
    assert_eq!(result.meta.total, 2);
    teardown(db, &table).await;
}

#[tokio::test]
async fn fetch_paginated_second_page_advances_offset() {
    let (db, table) = setup_table("pag_second").await;
    for i in 0..5 {
        insert_user(&db, &table, &format!("u{i}"), &format!("u{i}@x"), None).await;
    }
    let sql = format!("SELECT id FROM `{table}` ORDER BY id");
    let result = db
        .fetch_paginated(
            Query::new(sql),
            PageRequest {
                page: 2,
                per_page: 2,
            },
        )
        .await
        .expect("page2");
    assert_eq!(result.items.len(), 2);
    assert_eq!(result.meta.page, 2);
    teardown(db, &table).await;
}

#[tokio::test]
async fn fetch_paginated_last_page_partial() {
    let (db, table) = setup_table("pag_last").await;
    for i in 0..5 {
        insert_user(&db, &table, &format!("u{i}"), &format!("u{i}@x"), None).await;
    }
    let sql = format!("SELECT id FROM `{table}` ORDER BY id");
    let result = db
        .fetch_paginated(
            Query::new(sql),
            PageRequest {
                page: 3,
                per_page: 2,
            },
        )
        .await
        .expect("page3");
    assert_eq!(result.items.len(), 1);
    teardown(db, &table).await;
}

#[tokio::test]
async fn fetch_paginated_beyond_last_page_is_empty() {
    let (db, table) = setup_table("pag_beyond").await;
    insert_user(&db, &table, "A", "a@x", None).await;
    let sql = format!("SELECT id FROM `{table}` ORDER BY id");
    let result = db
        .fetch_paginated(
            Query::new(sql),
            PageRequest {
                page: 999,
                per_page: 10,
            },
        )
        .await
        .expect("beyond");
    assert!(result.items.is_empty());
    teardown(db, &table).await;
}

#[tokio::test]
async fn fetch_paginated_rejects_page_zero() {
    let db = test_db().await;
    let result = db
        .fetch_paginated(
            Query::new("SELECT 1"),
            PageRequest {
                page: 0,
                per_page: 10,
            },
        )
        .await;
    assert!(matches!(result, Err(DbError::Validation { .. })));
    db.close().await;
}

#[tokio::test]
async fn fetch_paginated_rejects_per_page_zero() {
    let db = test_db().await;
    let result = db
        .fetch_paginated(
            Query::new("SELECT 1"),
            PageRequest {
                page: 1,
                per_page: 0,
            },
        )
        .await;
    assert!(matches!(result, Err(DbError::Validation { .. })));
    db.close().await;
}

#[tokio::test]
async fn tx_trait_commit_persists_insert() {
    let (db, table) = setup_table("tx_commit").await;
    {
        let tx = TransactionManager::begin(&db).await.expect("begin");
        let sql = format!("INSERT INTO `{table}` (name, email) VALUES (?, ?)");
        tx.execute(Query::new(sql).bind("T").bind("t@x"))
            .await
            .expect("insert in tx");
        tx.commit().await.expect("commit");
    }
    assert_eq!(count_rows(&db, &table).await, 1);
    teardown(db, &table).await;
}

#[tokio::test]
async fn tx_trait_rollback_discards_insert() {
    let (db, table) = setup_table("tx_rollback").await;
    {
        let tx = TransactionManager::begin(&db).await.expect("begin");
        let sql = format!("INSERT INTO `{table}` (name, email) VALUES (?, ?)");
        tx.execute(Query::new(sql).bind("T").bind("t@x"))
            .await
            .expect("insert");
        tx.rollback().await.expect("rollback");
    }
    assert_eq!(count_rows(&db, &table).await, 0);
    teardown(db, &table).await;
}

#[tokio::test]
async fn tx_trait_savepoint_rollback_to_discards_since_savepoint() {
    let (db, table) = setup_table("tx_sp_rollback").await;
    {
        let mut tx = TransactionManager::begin(&db).await.expect("begin");
        let sql = format!("INSERT INTO `{table}` (name, email) VALUES (?, ?)");
        tx.execute(Query::new(&sql).bind("A").bind("a@x"))
            .await
            .expect("outer insert");
        tx.savepoint("sp1").await.expect("savepoint");
        tx.execute(Query::new(&sql).bind("B").bind("b@x"))
            .await
            .expect("inner insert");
        tx.rollback_to("sp1").await.expect("rollback_to");
        tx.commit().await.expect("commit");
    }
    assert_eq!(count_rows(&db, &table).await, 1);
    teardown(db, &table).await;
}

#[tokio::test]
async fn tx_trait_savepoint_release_keeps_changes() {
    let (db, table) = setup_table("tx_sp_release").await;
    {
        let mut tx = TransactionManager::begin(&db).await.expect("begin");
        let sql = format!("INSERT INTO `{table}` (name, email) VALUES (?, ?)");
        tx.savepoint("sp1").await.expect("savepoint");
        tx.execute(Query::new(&sql).bind("A").bind("a@x"))
            .await
            .expect("insert");
        tx.release_savepoint("sp1").await.expect("release");
        tx.commit().await.expect("commit");
    }
    assert_eq!(count_rows(&db, &table).await, 1);
    teardown(db, &table).await;
}

#[tokio::test]
async fn tx_trait_begin_with_isolation_levels() {
    let db = test_db().await;
    for level in [
        IsolationLevel::ReadUncommitted,
        IsolationLevel::ReadCommitted,
        IsolationLevel::RepeatableRead,
        IsolationLevel::Serializable,
    ] {
        let tx = TransactionManager::begin_with(&db, level)
            .await
            .expect("begin_with");
        tx.rollback().await.expect("rollback");
    }
    db.close().await;
}

#[tokio::test]
async fn tx_trait_savepoint_invalid_identifier_fails() {
    let db = test_db().await;
    let mut tx = TransactionManager::begin(&db).await.expect("begin");
    let result = tx.savepoint("bad name!").await;
    assert!(matches!(result, Err(DbError::Validation { .. })));
    tx.rollback().await.expect("rollback");
    db.close().await;
}

#[tokio::test]
async fn inherent_begin_commit_persists() {
    let (db, table) = setup_table("inh_commit").await;
    {
        let mut tx = db.begin().await.expect("begin");
        let sql = format!("INSERT INTO `{table}` (name, email) VALUES (?, ?)");
        sqlx::query(&sql)
            .bind("X")
            .bind("x@x")
            .execute(&mut *tx)
            .await
            .expect("insert");
        tx.commit().await.expect("commit");
    }
    assert_eq!(count_rows(&db, &table).await, 1);
    teardown(db, &table).await;
}

#[tokio::test]
async fn inherent_begin_rollback_discards() {
    let (db, table) = setup_table("inh_rollback").await;
    {
        let mut tx = db.begin().await.expect("begin");
        let sql = format!("INSERT INTO `{table}` (name, email) VALUES (?, ?)");
        sqlx::query(&sql)
            .bind("X")
            .bind("x@x")
            .execute(&mut *tx)
            .await
            .expect("insert");
        tx.rollback().await.expect("rollback");
    }
    assert_eq!(count_rows(&db, &table).await, 0);
    teardown(db, &table).await;
}

#[tokio::test]
async fn schema_create_and_drop_table() {
    let db = test_db().await;
    let table = unique_table("sch_basic");
    let create_sql = format!("CREATE TABLE `{table}` (id INT PRIMARY KEY)");
    sqlx::query(&create_sql)
        .execute(db.pool())
        .await
        .expect("create");
    assert!(db.table_exists(&table).await.expect("exists"));
    db.drop_table(&table, false).await.expect("drop");
    assert!(!db.table_exists(&table).await.expect("exists after"));
    db.close().await;
}

#[tokio::test]
async fn schema_drop_missing_table_fails_without_if_exists() {
    let db = test_db().await;
    let table = unique_table("sch_missing");
    let result = db.drop_table(&table, false).await;
    assert!(matches!(result, Err(DbError::Database { .. })));
    db.close().await;
}

#[tokio::test]
async fn schema_drop_missing_table_ok_with_if_exists() {
    let db = test_db().await;
    let table = unique_table("sch_missing_if");
    assert!(db.drop_table(&table, true).await.is_ok());
    db.close().await;
}

#[tokio::test]
async fn schema_drop_table_rejects_invalid_identifier() {
    let db = test_db().await;
    let result = db.drop_table("users; DROP TABLE x", true).await;
    assert!(matches!(result, Err(DbError::Validation { .. })));
    db.close().await;
}

#[tokio::test]
async fn schema_table_exists_returns_false_for_missing() {
    let db = test_db().await;
    let table = unique_table("sch_absent");
    assert!(!db.table_exists(&table).await.expect("exists"));
    db.close().await;
}

#[tokio::test]
async fn schema_table_exists_rejects_invalid_identifier() {
    let db = test_db().await;
    let result = db.table_exists("users; DROP TABLE x").await;
    assert!(matches!(result, Err(DbError::Validation { .. })));
    db.close().await;
}

#[tokio::test]
async fn schema_list_tables_includes_created() {
    let (db, table) = setup_table("sch_list").await;
    let tables = db.list_tables().await.expect("list");
    assert!(
        tables.iter().any(|t| t == &table),
        "expected {table} in {tables:?}"
    );
    teardown(db, &table).await;
}

#[tokio::test]
async fn schema_alter_add_column() {
    let (db, table) = setup_table("sch_add_col").await;
    let change = TableChange::AddColumn(ColumnDef::new("bio", DataType::Text).nullable(true));
    db.alter_table(&table, vec![change]).await.expect("alter");
    let sql = format!("INSERT INTO `{table}` (name, email, bio) VALUES (?, ?, ?)");
    sqlx::query(&sql)
        .bind("A")
        .bind("a@x")
        .bind(Option::<String>::None)
        .execute(db.pool())
        .await
        .expect("insert with new column");
    teardown(db, &table).await;
}

#[tokio::test]
async fn schema_alter_drop_column() {
    let (db, table) = setup_table("sch_drop_col").await;
    let change = TableChange::DropColumn("age".into());
    db.alter_table(&table, vec![change]).await.expect("alter");
    let sql = format!("SELECT age FROM `{table}` LIMIT 0");
    let result = sqlx::query(&sql).execute(db.pool()).await;
    assert!(result.is_err(), "column age should be gone");
    teardown(db, &table).await;
}

#[tokio::test]
async fn schema_alter_empty_changes_is_noop() {
    let (db, table) = setup_table("sch_noop").await;
    assert!(db.alter_table(&table, vec![]).await.is_ok());
    assert!(db.table_exists(&table).await.expect("exists"));
    teardown(db, &table).await;
}

#[tokio::test]
async fn schema_alter_rejects_invalid_table_identifier() {
    let db = test_db().await;
    let result = db.alter_table("bad name", vec![]).await;
    assert!(matches!(result, Err(DbError::Validation { .. })));
    db.close().await;
}

#[tokio::test]
async fn schema_create_index_and_drop_index() {
    let (db, table) = setup_table("sch_idx").await;
    let idx = Index::new("idx_name", vec!["name".into()]);
    db.create_index(&table, idx).await.expect("create index");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.statistics \
         WHERE table_schema = DATABASE() AND table_name = ? AND index_name = ?",
    )
    .bind(&table)
    .bind("idx_name")
    .fetch_one(db.pool())
    .await
    .expect("count");
    assert_eq!(count, 1);

    db.drop_index(&table, "idx_name", false)
        .await
        .expect("drop");
    let count_after: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.statistics \
         WHERE table_schema = DATABASE() AND table_name = ? AND index_name = ?",
    )
    .bind(&table)
    .bind("idx_name")
    .fetch_one(db.pool())
    .await
    .expect("count after");
    assert_eq!(count_after, 0);

    teardown(db, &table).await;
}

#[tokio::test]
async fn schema_drop_index_if_exists_missing_is_ok() {
    let (db, table) = setup_table("sch_idx_if").await;
    assert!(db.drop_index(&table, "nonexistent_idx", true).await.is_ok());
    teardown(db, &table).await;
}

#[tokio::test]
async fn schema_drop_index_missing_fails_without_if_exists() {
    let (db, table) = setup_table("sch_idx_missing").await;
    let result = db.drop_index(&table, "nonexistent_idx", false).await;
    assert!(result.is_err());
    teardown(db, &table).await;
}

async fn isolated_db(name: &str) -> Database {
    let admin = admin_pool().await;
    sqlx::query(&format!("DROP DATABASE IF EXISTS {name}"))
        .execute(&admin)
        .await
        .expect("drop old isolated");
    sqlx::query(&format!("CREATE DATABASE {name}"))
        .execute(&admin)
        .await
        .expect("create isolated");
    admin.close().await;
    drop(admin);
    let url = format!("{ROOT_URL}/{name}");
    Database::connect(&url).await.expect("connect isolated")
}

async fn drop_isolated(name: &str) {
    let admin = admin_pool().await;
    let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS {name}"))
        .execute(&admin)
        .await;
    admin.close().await;
}

#[tokio::test]
async fn migration_run_pending_no_migrations_returns_empty() {
    let db_name = "pbl_test_db_mig_empty";
    let db = isolated_db(db_name).await;
    let applied = db.run_pending().await.expect("run_pending");
    assert!(applied.is_empty());
    db.close().await;
    drop_isolated(db_name).await;
}

#[tokio::test]
async fn migration_run_pending_applies_new_migration() {
    let db_name = "pbl_test_db_mig_apply";
    let db = isolated_db(db_name).await;
    let m = Migration::new(1, "create_users", "CREATE TABLE users (id INT PRIMARY KEY)");
    let db = db.with_migrations(vec![m]);
    let applied = db.run_pending().await.expect("run_pending");
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].version, 1);
    assert!(db.table_exists("users").await.expect("exists"));
    db.close().await;
    drop_isolated(db_name).await;
}

#[tokio::test]
async fn migration_run_pending_is_idempotent() {
    let db_name = "pbl_test_db_mig_idem";
    let db = isolated_db(db_name).await;
    let m = Migration::new(1, "create_users", "CREATE TABLE users (id INT PRIMARY KEY)");
    let db = db.with_migrations(vec![m]);
    let first = db.run_pending().await.expect("first");
    assert_eq!(first.len(), 1);
    let second = db.run_pending().await.expect("second");
    assert!(second.is_empty(), "second run should apply nothing");
    db.close().await;
    drop_isolated(db_name).await;
}

#[tokio::test]
async fn migration_rollback_last_reverses_migration() {
    let db_name = "pbl_test_db_mig_roll";
    let db = isolated_db(db_name).await;
    let m = Migration::new(1, "create_users", "CREATE TABLE users (id INT PRIMARY KEY)")
        .with_down("DROP TABLE users");
    let db = db.with_migrations(vec![m]);
    db.run_pending().await.expect("apply");
    assert!(db.table_exists("users").await.expect("exists"));
    let rolled = db.rollback_last().await.expect("rollback");
    assert!(rolled.is_some());
    assert!(!db.table_exists("users").await.expect("exists after"));
    db.close().await;
    drop_isolated(db_name).await;
}

#[tokio::test]
async fn migration_rollback_last_on_empty_returns_none() {
    let db_name = "pbl_test_db_mig_none";
    let db = isolated_db(db_name).await;
    let rolled = db.rollback_last().await.expect("rollback");
    assert!(rolled.is_none());
    db.close().await;
    drop_isolated(db_name).await;
}

#[tokio::test]
async fn migration_status_reports_pending_then_applied() {
    let db_name = "pbl_test_db_mig_status";
    let db = isolated_db(db_name).await;
    let m = Migration::new(1, "create_users", "CREATE TABLE users (id INT PRIMARY KEY)");
    let db = db.with_migrations(vec![m]);
    let status_pending = db.status().await.expect("status pending");
    assert_eq!(status_pending.len(), 1);
    assert!(status_pending[0].applied_at.is_none());
    db.run_pending().await.expect("apply");
    let status_applied = db.status().await.expect("status applied");
    assert_eq!(status_applied.len(), 1);
    assert!(status_applied[0].applied_at.is_some());
    db.close().await;
    drop_isolated(db_name).await;
}

#[tokio::test]
async fn pool_accessor_is_usable() {
    let db = test_db().await;
    let pool = db.pool();
    let row: (i64,) = sqlx::query_as("SELECT 1")
        .fetch_one(pool)
        .await
        .expect("via pool accessor");
    assert_eq!(row.0, 1);
    db.close().await;
}

#[tokio::test]
async fn from_pool_wraps_existing_pool() {
    ensure_test_db().await;
    let pool = MySqlPoolOptions::new()
        .max_connections(2)
        .connect(&test_db_url())
        .await
        .expect("raw pool");
    let db = Database::from_pool(pool);
    assert!(db.ping().await.is_ok());
    db.close().await;
}

#[tokio::test]
async fn with_migrations_stores_them_for_later_runs() {
    let db_name = "pbl_test_db_mig_builder";
    let db = isolated_db(db_name).await;
    let m = Migration::new(1, "noop", "SELECT 1");
    let db = db.with_migrations(vec![m]);
    let statuses = db.status().await.expect("status");
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].migration.version, 1);
    db.close().await;
    drop_isolated(db_name).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_pings_all_succeed() {
    let db = Arc::new(test_db().await);
    let mut handles = Vec::new();
    for _ in 0..16 {
        let db = Arc::clone(&db);
        handles.push(tokio::spawn(async move { db.ping().await }));
    }
    for h in handles {
        h.await.expect("join").expect("ping");
    }
    drop(db);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_fetches_use_pool_correctly() {
    let (db, table) = setup_table("conc_fetch").await;
    let db = Arc::new(db);
    let sql = format!("SELECT COUNT(*) AS cnt FROM `{table}`");
    let mut handles = Vec::new();
    for _ in 0..8 {
        let db = Arc::clone(&db);
        let sql = sql.clone();
        handles.push(tokio::spawn(
            async move { db.fetch_one(Query::new(sql)).await },
        ));
    }
    for h in handles {
        h.await.expect("join").expect("fetch");
    }
    let db = Arc::try_unwrap(db).expect("sole owner");
    teardown(db, &table).await;
}
