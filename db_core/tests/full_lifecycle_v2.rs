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
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args,
    unused_results
)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use db_core::{Database, EntityRepository};
use db_handler::{
    ColumnDef, Condition, Connection, CrudRepository, DataType, DatabaseInfoProvider, DbError,
    Filter, HealthCheck, HealthStatus, Index, InsertInput, IsolationLevel, Migration,
    MigrationRunner, PageRequest, PoolConfig, Query, Result, Row, SchemaManager, Table,
    TableChange, TransactionManager, UpdateInput, Value,
};

use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

const TEST_DB: &str = "pbl_test_full_v2";
const ROOT_URL: &str = "mysql://root:root@127.0.0.1:3307";

static TABLE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn db_url(name: &str) -> String {
    format!("{ROOT_URL}/{name}")
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
    Database::connect(&db_url(TEST_DB))
        .await
        .expect("connect test db")
}

async fn test_db_single_conn() -> Database {
    ensure_test_db().await;
    Database::connect_with(
        &db_url(TEST_DB),
        PoolConfig {
            max_connections: 1,
            min_connections: 0,
            ..Default::default()
        },
    )
    .await
    .expect("connect single-conn test db")
}

async fn isolated_db(name: &str) -> Database {
    {
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
    }

    Database::connect(&db_url(name))
        .await
        .expect("connect isolated")
}

async fn drop_isolated(name: &str) {
    let admin = admin_pool().await;
    let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS {name}"))
        .execute(&admin)
        .await;
    admin.close().await;
}


async fn create_user_table(db: &Database, table: &str) {
    let _ = sqlx::query(&format!("DROP TABLE IF EXISTS `{table}`"))
        .execute(db.pool())
        .await;
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
        .expect("create user table");
}

async fn count_rows(db: &Database, table: &str) -> i64 {
    let sql = format!("SELECT COUNT(*) FROM `{table}`");
    let row = sqlx::query(&sql)
        .fetch_one(db.pool())
        .await
        .expect("count rows");
    sqlx::Row::try_get::<i64, _>(&row, 0).expect("count as i64")
}

async fn teardown(db: Database, tables: &[&str]) {
    for t in tables {
        let _ = sqlx::query(&format!("DROP TABLE IF EXISTS `{t}`"))
            .execute(db.pool())
            .await;
    }
    db.close().await;
}


#[derive(Debug, Clone, PartialEq)]
struct User {
    id: i64,
    name: String,
    email: String,
    age: Option<i64>,
}

type UserMapper = fn(&dyn Row) -> Result<User>;
type UserRepo = EntityRepository<User, UserMapper>;

fn map_user(row: &dyn Row) -> Result<User> {
    let id = row.get("id")?.as_i64().ok_or_else(|| DbError::Validation {
        message: "column `id` is not an integer".into(),
    })?;
    let name = row
        .get("name")?
        .as_str()
        .map(String::from)
        .ok_or_else(|| DbError::Validation {
            message: "column `name` is not a string".into(),
        })?;
    let email = row
        .get("email")?
        .as_str()
        .map(String::from)
        .ok_or_else(|| DbError::Validation {
            message: "column `email` is not a string".into(),
        })?;
    let age = row.get("age")?.as_i64();
    Ok(User {
        id,
        name,
        email,
        age,
    })
}


#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn full_lifecycle_v2() {
    let db_name = "pbl_test_full_v2_main";
    let setup_db = isolated_db(db_name).await;
    let repo_db = Database::connect(&db_url(db_name))
        .await
        .expect("repo connect");
    let table = unique_table("lifecycle_users");

    setup_db.ping().await.expect("ping");

    let health = setup_db.health().await.expect("health");
    assert_eq!(health.status, HealthStatus::Healthy);
    assert!(!health.server_version.is_empty());

    let info = setup_db.info().await.expect("info");
    assert_eq!(info.database_name, db_name);
    assert!(info.pool_size >= info.pool_idle);
    assert_eq!(
        info.pool_size,
        info.pool_idle.saturating_add(info.active_connections)
    );

    let table_def = Table::new(
        &table,
        vec![
            ColumnDef::new("id", DataType::BigInt)
                .primary_key()
                .auto_increment(),
            ColumnDef::new("name", DataType::Varchar(255)),
            ColumnDef::new("email", DataType::Varchar(255)).unique(),
            ColumnDef::new("age", DataType::Int).nullable(true),
        ],
    );
    setup_db.create_table(table_def).await.expect("create table");
    assert!(setup_db.table_exists(&table).await.expect("exists"));
    assert!(setup_db
        .list_tables()
        .await
        .expect("list tables")
        .iter()
        .any(|t| t == &table));

    let migration = Migration::new(
        1,
        "add_created_at",
        format!("ALTER TABLE `{table}` ADD COLUMN `created_at` TIMESTAMP NULL"),
    );
    let setup_db = setup_db.with_migrations(vec![migration]);
    let applied = setup_db.run_pending().await.expect("run migration");
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].version, 1);

    let statuses = setup_db.status().await.expect("migration status");
    assert_eq!(statuses.len(), 1);
    assert!(statuses[0].applied_at.is_some());

    let applied_again = setup_db.run_pending().await.expect("re-run migration");
    assert!(applied_again.is_empty(), "second run must apply nothing");

    let mapper: fn(&dyn Row) -> Result<User> = map_user;
    let repo = EntityRepository::new(repo_db, &table, "id", mapper);

    let alice = repo
        .create(
            InsertInput::new(&table)
                .set("name", "Alice")
                .set("email", "alice@x.com")
                .set("age", 30i64),
        )
        .await
        .expect("create alice");
    assert_eq!(alice.name, "Alice");
    assert_eq!(alice.email, "alice@x.com");
    assert_eq!(alice.age, Some(30));
    assert!(alice.id > 0);

    let bob = repo
        .create(
            InsertInput::new(&table)
                .set("name", "Bob")
                .set("email", "bob@x.com"),
        )
        .await
        .expect("create bob");
    assert_eq!(bob.age, None);

    let fetched = repo
        .get_by_id(&Value::Int(alice.id))
        .await
        .expect("get alice")
        .expect("some");
    assert_eq!(fetched, alice);

    assert_eq!(repo.count(Filter::new()).await.expect("count"), 2);

    let updated = repo
        .update(
            &Value::Int(alice.id),
            UpdateInput::new(&table).set("age", 31i64),
        )
        .await
        .expect("update");
    assert_eq!(updated.age, Some(31));

    let page = repo
        .list(
            Filter::new(),
            PageRequest {
                page: 1,
                per_page: 1,
            },
        )
        .await
        .expect("list page 1");
    assert_eq!(page.items.len(), 1);
    assert_eq!(
        page.meta.total, 2,
        "total must come from COUNT(*), not offset + page length"
    );
    assert_eq!(page.meta.total_pages, 2);

    let eq_filter = Filter::new().add(Condition::eq("name", "Bob"));
    let eq_page = repo
        .list(eq_filter, PageRequest::default())
        .await
        .expect("filter eq");
    assert_eq!(eq_page.items.len(), 1);
    assert_eq!(eq_page.items[0].name, "Bob");

    let in_filter = Filter::new().add(Condition::In {
        column: "name".into(),
        values: vec![Value::String("Alice".into()), Value::String("Bob".into())],
    });
    let in_page = repo
        .list(in_filter, PageRequest::default())
        .await
        .expect("filter in");
    assert_eq!(in_page.items.len(), 2);

    let null_filter = Filter::new().add(Condition::is_null("age"));
    let null_page = repo
        .list(null_filter, PageRequest::default())
        .await
        .expect("filter null");
    assert!(null_page.items.iter().all(|u| u.age.is_none()));

    repo.delete(&Value::Int(bob.id)).await.expect("delete bob");
    assert_eq!(repo.count(Filter::new()).await.expect("count"), 1);
    assert!(repo
        .get_by_id(&Value::Int(bob.id))
        .await
        .expect("get bob")
        .is_none());

    let baseline = count_rows(&setup_db, &table).await;

    {
        let tx = TransactionManager::begin(&setup_db).await.expect("begin");
        let sql = format!("INSERT INTO `{table}` (name, email) VALUES (?, ?)");
        tx.execute(Query::new(sql).bind("Rollback").bind("r@x"))
            .await
            .expect("insert in tx");
        tx.rollback().await.expect("rollback");
    }
    assert_eq!(count_rows(&setup_db, &table).await, baseline);

    {
        let tx = TransactionManager::begin(&setup_db).await.expect("begin");
        let sql = format!("INSERT INTO `{table}` (name, email) VALUES (?, ?)");
        tx.execute(Query::new(sql).bind("Committed").bind("c@x"))
            .await
            .expect("insert in tx");
        tx.commit().await.expect("commit");
    }
    assert_eq!(count_rows(&setup_db, &table).await, baseline + 1);

    {
        let mut tx = TransactionManager::begin(&setup_db).await.expect("begin");
        let sql = format!("INSERT INTO `{table}` (name, email) VALUES (?, ?)");
        tx.execute(Query::new(&sql).bind("Outer").bind("outer@x"))
            .await
            .expect("outer insert");
        tx.savepoint("sp1").await.expect("savepoint");
        tx.execute(Query::new(&sql).bind("Inner").bind("inner@x"))
            .await
            .expect("inner insert");
        tx.rollback_to("sp1").await.expect("rollback_to");
        tx.commit().await.expect("commit");
    }
    assert_eq!(count_rows(&setup_db, &table).await, baseline + 2);

    for level in [
        IsolationLevel::ReadUncommitted,
        IsolationLevel::ReadCommitted,
        IsolationLevel::RepeatableRead,
        IsolationLevel::Serializable,
    ] {
        let tx = TransactionManager::begin_with(&setup_db, level)
            .await
            .expect("begin_with");
        tx.rollback().await.expect("rollback");
    }

    setup_db
        .alter_table(
            &table,
            vec![TableChange::AddColumn(
                ColumnDef::new("bio", DataType::Text).nullable(true),
            )],
        )
        .await
        .expect("alter add column");

    setup_db
        .create_index(&table, Index::new("idx_name", vec!["name".into()]))
        .await
        .expect("create index");
    setup_db
        .drop_index(&table, "idx_name", false)
        .await
        .expect("drop index");

    setup_db.drop_table(&table, false).await.expect("drop table");
    assert!(!setup_db.table_exists(&table).await.expect("exists after"));

    drop(repo);
    setup_db.close().await;
    drop_isolated(db_name).await;
}


#[tokio::test]
async fn entity_repository_rejects_invalid_identifiers() {
    let lazy_pool = MySqlPoolOptions::new()
        .connect_lazy(&db_url(TEST_DB))
        .expect("lazy pool");

    let mapper: fn(&dyn Row) -> Result<User> = map_user;

    for bad in ["", "has space", "1abc", "x;drop", "users'--"] {
        let db = Database::from_pool(lazy_pool.clone());
        let result = EntityRepository::try_new(db, bad, "id", mapper);
        assert!(
            matches!(result, Err(DbError::Validation { .. })),
            "table {bad:?} should be rejected"
        );
    }

    for bad in ["", "has space", "1abc", "id;drop"] {
        let db = Database::from_pool(lazy_pool.clone());
        let result = EntityRepository::try_new(db, "users", bad, mapper);
        assert!(
            matches!(result, Err(DbError::Validation { .. })),
            "pk {bad:?} should be rejected"
        );
    }

    lazy_pool.close().await;
}


#[tokio::test]
async fn entity_repository_forces_bound_table() {
    let verify_db = test_db().await;
    let repo_db = test_db().await;

    let bound_table = unique_table("force_bound");
    let wrong_table = unique_table("force_wrong");
    create_user_table(&verify_db, &bound_table).await;

    let mapper: fn(&dyn Row) -> Result<User> = map_user;
    let repo = EntityRepository::new(repo_db, &bound_table, "id", mapper);

    let user = repo
        .create(
            InsertInput::new(&wrong_table)
                .set("name", "Alice")
                .set("email", "a@x"),
        )
        .await
        .expect("create must normalize table");
    assert_eq!(user.name, "Alice");
    assert_eq!(count_rows(&verify_db, &bound_table).await, 1);

    let updated = repo
        .update(
            &Value::Int(user.id),
            UpdateInput::new(&wrong_table).set("name", "Alice-renamed"),
        )
        .await
        .expect("update must normalize table");
    assert_eq!(updated.name, "Alice-renamed");

    teardown(verify_db, &[&bound_table]).await;
}


#[tokio::test]
async fn entity_repository_list_rejects_bad_pagination() {
    let verify_db = test_db().await;
    let repo_db = test_db().await;
    let table = unique_table("bad_page");
    create_user_table(&verify_db, &table).await;

    let mapper: fn(&dyn Row) -> Result<User> = map_user;
    let repo = EntityRepository::new(repo_db, &table, "id", mapper);

    for (page, per_page) in [(0u64, 10u64), (1, 0), (0, 0)] {
        let result = repo
            .list(Filter::new(), PageRequest { page, per_page })
            .await;
        assert!(
            matches!(result, Err(DbError::Validation { .. })),
            "page={page}, per_page={per_page} should be rejected"
        );
    }

    teardown(verify_db, &[&table]).await;
}


#[tokio::test]
async fn dropped_transaction_does_not_leak_to_pool() {
    let db = test_db_single_conn().await;
    let table = unique_table("tx_drop");
    create_user_table(&db, &table).await;

    {
        let tx = TransactionManager::begin(&db).await.expect("begin");
        let sql = format!("INSERT INTO `{table}` (name, email) VALUES (?, ?)");
        tx.execute(Query::new(sql).bind("leaked").bind("leak@x"))
            .await
            .expect("insert in tx");
    }

    tokio::time::sleep(Duration::from_millis(150)).await;

    let count = count_rows(&db, &table).await;
    assert_eq!(
        count, 0,
        "abandoned transaction must be rolled back, not leaked to the pool"
    );

    teardown(db, &[&table]).await;
}


#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_entity_repository_crud() {
    let verify_db = test_db().await;
    let repo_db = test_db().await;
    let table = unique_table("conc_repo");
    create_user_table(&verify_db, &table).await;

    let mapper: fn(&dyn Row) -> Result<User> = map_user;
    let repo: Arc<UserRepo> = Arc::new(EntityRepository::new(repo_db, &table, "id", mapper));


    let mut handles = Vec::new();
    for i in 0..16u32 {
        let repo = Arc::clone(&repo);
        let table = table.clone();
        handles.push(tokio::spawn(async move {
            repo.create(
                InsertInput::new(table)
                    .set("name", format!("u{i}"))
                    .set("email", format!("u{i}@x")),
            )
            .await
            .map(|u| u.id)
        }));
    }

    let mut ids = Vec::new();
    for h in handles {
        ids.push(h.await.expect("join").expect("create"));
    }

    assert_eq!(count_rows(&verify_db, &table).await, 16);

    let unique: std::collections::HashSet<_> = ids.iter().copied().collect();
    assert_eq!(unique.len(), 16, "auto-increment ids must be unique");

    drop(repo);
    tokio::time::sleep(Duration::from_millis(100)).await;
    teardown(verify_db, &[&table]).await;
}


#[tokio::test]
async fn entity_repository_missing_row_semantics() {
    let verify_db = test_db().await;
    let repo_db = test_db().await;
    let table = unique_table("missing_row");
    create_user_table(&verify_db, &table).await;

    let mapper: fn(&dyn Row) -> Result<User> = map_user;
    let repo = EntityRepository::new(repo_db, &table, "id", mapper);

    let missing = repo
        .get_by_id(&Value::Int(999_999))
        .await
        .expect("get_by_id is not an error for missing row");
    assert!(missing.is_none());

    repo.delete(&Value::Int(999_999))
        .await
        .expect("delete is a silent no-op on missing row");

    let result = repo
        .update(
            &Value::Int(999_999),
            UpdateInput::new(&table).set("name", "x"),
        )
        .await;
    assert!(matches!(result, Err(DbError::NotFound)));

    teardown(verify_db, &[&table]).await;
}
