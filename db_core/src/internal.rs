use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use db_handler::validation::validate_identifier;
use db_handler::{Column, ColumnDef, DataType, DbError, Result, Row, Table, TableChange, Value};
use sqlx::mysql::{MySql, MySqlArguments, MySqlRow};
use sqlx::pool::PoolConnection;
use sqlx::query::Query as SqlxQuery;
use sqlx::TypeInfo;
use sqlx::{Column as _, Row as _};

pub(crate) async fn exec_raw_text(
    conn: PoolConnection<MySql>,
    sql: String,
) -> Result<PoolConnection<MySql>> {
    tokio::task::spawn_blocking(move || {
        let mut conn = conn;
        pollster::block_on(async {
            let _ = sqlx::raw_sql(&sql)
                .execute(&mut *conn)
                .await
                .map_err(|e| e.to_string())?;
            Ok::<_, String>(conn)
        })
    })
    .await
    .map_err(|e| DbError::Database {
        message: format!("spawn_blocking join error: {e}"),
    })?
    .map_err(|e| DbError::Database { message: e })
}

pub(crate) fn bind_value(
    q: SqlxQuery<'_, MySql, MySqlArguments>,
    v: Value,
) -> SqlxQuery<'_, MySql, MySqlArguments> {
    match v {
        Value::Null => q.bind(Option::<i64>::None),
        Value::Bool(b) => q.bind(b),
        Value::Int(i) => q.bind(i),
        Value::Uint(u) => q.bind(u),
        Value::Float(f) => q.bind(f),
        Value::String(s) => q.bind(s),
        Value::Bytes(b) => q.bind(b),
        Value::Timestamp(t) => q.bind(t),
    }
}

pub(crate) fn decode_value(row: &MySqlRow, name: &str) -> Result<Value> {
    if let Ok(v) = row.try_get::<Option<i64>, _>(name) {
        return Ok(v.map_or(Value::Null, Value::Int));
    }
    if let Ok(v) = row.try_get::<Option<u64>, _>(name) {
        return Ok(v.map_or(Value::Null, Value::Uint));
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(name) {
        return Ok(v.map_or(Value::Null, Value::Float));
    }
    if let Ok(v) = row.try_get::<Option<String>, _>(name) {
        return Ok(v.map_or(Value::Null, Value::String));
    }
    if let Ok(v) = row.try_get::<Option<Vec<u8>>, _>(name) {
        return Ok(v.map_or(Value::Null, Value::Bytes));
    }
    if let Ok(v) = row.try_get::<Option<NaiveDateTime>, _>(name) {
        return Ok(v.map_or(Value::Null, |dt| {
            Value::Timestamp(dt.format("%Y-%m-%dT%H:%M:%S%.f").to_string())
        }));
    }
    if let Ok(v) = row.try_get::<Option<NaiveDate>, _>(name) {
        return Ok(v.map_or(Value::Null, |d| {
            Value::Timestamp(d.format("%Y-%m-%d").to_string())
        }));
    }
    if let Ok(v) = row.try_get::<Option<NaiveTime>, _>(name) {
        return Ok(v.map_or(Value::Null, |t| {
            Value::Timestamp(t.format("%H:%M:%S%.f").to_string())
        }));
    }
    Err(DbError::Query {
        message: format!("failed to decode column {name}"),
    })
}

#[derive(Debug)]
pub(crate) struct MySqlRowWrapper {
    values: Vec<(String, Value)>,
    columns: Vec<Column>,
}

impl MySqlRowWrapper {
    pub(crate) fn from_row(row: &MySqlRow) -> Result<Self> {
        let mut values = Vec::with_capacity(row.len());
        let mut columns = Vec::with_capacity(row.len());
        for col in row.columns() {
            let name = col.name().to_string();
            let data_type = col.type_info().name().to_string();
            columns.push(Column::new(name.clone(), data_type, true));
            let v = decode_value(row, &name)?;
            values.push((name, v));
        }
        Ok(Self { values, columns })
    }

    pub(crate) fn columns_from_row(row: &MySqlRow) -> Vec<Column> {
        row.columns()
            .iter()
            .map(|c| Column::new(c.name(), c.type_info().name(), true))
            .collect()
    }
}

impl Row for MySqlRowWrapper {
    fn get(&self, column: &str) -> Result<&Value> {
        self.values
            .iter()
            .find(|(n, _)| n == column)
            .map(|(_, v)| v)
            .ok_or_else(|| DbError::Validation {
                message: format!("column not found: {column}"),
            })
    }

    fn columns(&self) -> &[Column] {
        &self.columns
    }

    fn len(&self) -> usize {
        self.columns.len()
    }

    fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}

pub(crate) fn data_type_sql(t: DataType) -> String {
    match t {
        DataType::Int => "INT".into(),
        DataType::BigInt => "BIGINT".into(),
        DataType::SmallInt => "SMALLINT".into(),
        DataType::Varchar(n) => format!("VARCHAR({n})"),
        DataType::Text => "TEXT".into(),
        DataType::Boolean => "TINYINT(1)".into(),
        DataType::Timestamp => "TIMESTAMP".into(),
        DataType::Date => "DATE".into(),
        DataType::Float => "FLOAT".into(),
        DataType::Double => "DOUBLE".into(),
        DataType::Blob => "BLOB".into(),
    }
}

pub(crate) fn value_to_sql_literal(v: &Value) -> Result<String> {
    Ok(match v {
        Value::Null => "NULL".into(),
        Value::Bool(b) => {
            if *b {
                "TRUE".into()
            } else {
                "FALSE".into()
            }
        }
        Value::Int(i) => i.to_string(),
        Value::Uint(u) => u.to_string(),
        Value::Float(f) => {
            if !f.is_finite() {
                return Err(DbError::Validation {
                    message: format!("cannot embed non-finite float ({f}) in DDL"),
                });
            }
            f.to_string()
        }
        Value::String(s) | Value::Timestamp(s) => {
            format!("'{}'", s.replace('\'', "''"))
        }
        Value::Bytes(_) => {
            return Err(DbError::Validation {
                message: "cannot embed binary literal in DDL".into(),
            });
        }
    })
}

pub(crate) fn column_def_sql(cd: &ColumnDef) -> Result<String> {
    validate_identifier(&cd.name)?;
    let mut parts = vec![format!("`{}` {}", cd.name, data_type_sql(cd.data_type))];
    if !cd.nullable {
        parts.push("NOT NULL".into());
    }
    if cd.auto_increment {
        parts.push("AUTO_INCREMENT".into());
    }
    if cd.primary_key {
        parts.push("PRIMARY KEY".into());
    }
    if cd.unique {
        parts.push("UNIQUE".into());
    }
    if let Some(d) = &cd.default {
        parts.push(format!("DEFAULT {}", value_to_sql_literal(d)?));
    }
    Ok(parts.join(" "))
}

pub(crate) fn build_create_table(table: &Table) -> Result<String> {
    validate_identifier(&table.name)?;
    if table.columns.is_empty() {
        return Err(DbError::Validation {
            message: "table must have at least one column".into(),
        });
    }
    let mut parts = Vec::with_capacity(table.columns.len());
    for col in &table.columns {
        parts.push(column_def_sql(col)?);
    }
    Ok(format!(
        "CREATE TABLE `{}` ({})",
        table.name,
        parts.join(", ")
    ))
}

pub(crate) fn table_change_sql(c: &TableChange) -> Result<String> {
    Ok(match c {
        TableChange::AddColumn(cd) => format!("ADD COLUMN {}", column_def_sql(cd)?),
        TableChange::DropColumn(name) => {
            validate_identifier(name)?;
            format!("DROP COLUMN `{name}`")
        }
        TableChange::RenameColumn { from, to } => {
            validate_identifier(from)?;
            validate_identifier(to)?;
            format!("RENAME COLUMN `{from}` TO `{to}`")
        }
        TableChange::ModifyColumn(cd) => format!("MODIFY COLUMN {}", column_def_sql(cd)?),
        TableChange::AddIndex(idx) => {
            validate_identifier(&idx.name)?;
            for col in &idx.columns {
                validate_identifier(col)?;
            }
            let cols = idx
                .columns
                .iter()
                .map(|c| format!("`{c}`"))
                .collect::<Vec<_>>()
                .join(", ");
            let unique = if idx.unique { "UNIQUE " } else { "" };
            format!("ADD {unique}INDEX `{}` ({cols})", idx.name)
        }
        TableChange::DropIndex(name) => {
            validate_identifier(name)?;
            format!("DROP INDEX `{name}`")
        }
        TableChange::AddConstraint(_) => {
            return Err(DbError::Validation {
                message: "AddConstraint not yet implemented".into(),
            });
        }
        TableChange::DropConstraint(name) => {
            validate_identifier(name)?;
            format!("DROP CONSTRAINT `{name}`")
        }
    })
}

pub(crate) async fn ensure_migration_table(pool: &sqlx::MySqlPool) -> Result<()> {
    let sql = r"
        CREATE TABLE IF NOT EXISTS `_migrations` (
            `version` BIGINT PRIMARY KEY,
            `name` VARCHAR(255) NOT NULL,
            `applied_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    ";
    sqlx::query(sql)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| DbError::Database {
            message: e.to_string(),
        })
}

pub(crate) fn build_insert(input: &db_handler::InsertInput) -> Result<(String, Vec<Value>)> {
    validate_identifier(&input.table)?;
    if input.values.is_empty() {
        return Err(DbError::Validation {
            message: "insert must have at least one value".into(),
        });
    }
    let n = input.values.len();
    let mut cols = Vec::with_capacity(n);
    let mut placeholders = Vec::with_capacity(n);
    let mut params = Vec::with_capacity(n);
    for cv in &input.values {
        validate_identifier(&cv.column)?;
        cols.push(format!("`{}`", cv.column));
        placeholders.push("?");
        params.push(cv.value.clone());
    }
    let sql = format!(
        "INSERT INTO `{}` ({}) VALUES ({})",
        input.table,
        cols.join(", "),
        placeholders.join(", ")
    );
    Ok((sql, params))
}

pub(crate) fn build_update(input: &db_handler::UpdateInput) -> Result<(String, Vec<Value>)> {
    validate_identifier(&input.table)?;
    if input.set.is_empty() {
        return Err(DbError::Validation {
            message: "update must have at least one set value".into(),
        });
    }
    let mut sets = Vec::with_capacity(input.set.len());
    let mut params = Vec::with_capacity(input.set.len());
    for cv in &input.set {
        validate_identifier(&cv.column)?;
        sets.push(format!("`{}` = ?", cv.column));
        params.push(cv.value.clone());
    }
    let (where_sql, where_params) = build_filter(&input.filter)?;
    params.extend(where_params);
    let sql = format!(
        "UPDATE `{}` SET {}{}",
        input.table,
        sets.join(", "),
        where_sql
    );
    Ok((sql, params))
}

pub(crate) fn build_delete(input: &db_handler::DeleteInput) -> Result<(String, Vec<Value>)> {
    validate_identifier(&input.table)?;
    let (where_sql, params) = build_filter(&input.filter)?;
    let sql = format!("DELETE FROM `{}`{}", input.table, where_sql);
    Ok((sql, params))
}

pub(crate) fn build_filter(filter: &db_handler::Filter) -> Result<(String, Vec<Value>)> {
    if filter.conditions.is_empty() {
        return Ok((String::new(), Vec::new()));
    }
    let mut parts = Vec::with_capacity(filter.conditions.len());
    let mut params = Vec::new();
    for cond in &filter.conditions {
        let (part, p) = build_condition(cond)?;
        parts.push(part);
        params.extend(p);
    }
    Ok((format!(" WHERE {}", parts.join(" AND ")), params))
}

fn build_condition(cond: &db_handler::Condition) -> Result<(String, Vec<Value>)> {
    use db_handler::Condition;
    match cond {
        Condition::Eq { column, value } => {
            validate_identifier(column)?;
            Ok((format!("`{column}` = ?"), vec![value.clone()]))
        }
        Condition::Ne { column, value } => {
            validate_identifier(column)?;
            Ok((format!("`{column}` != ?"), vec![value.clone()]))
        }
        Condition::Lt { column, value } => {
            validate_identifier(column)?;
            Ok((format!("`{column}` < ?"), vec![value.clone()]))
        }
        Condition::Le { column, value } => {
            validate_identifier(column)?;
            Ok((format!("`{column}` <= ?"), vec![value.clone()]))
        }
        Condition::Gt { column, value } => {
            validate_identifier(column)?;
            Ok((format!("`{column}` > ?"), vec![value.clone()]))
        }
        Condition::Ge { column, value } => {
            validate_identifier(column)?;
            Ok((format!("`{column}` >= ?"), vec![value.clone()]))
        }
        Condition::In { column, values } => {
            validate_identifier(column)?;
            if values.is_empty() {
                return Ok(("1=0".into(), Vec::new()));
            }
            let placeholders = vec!["?"; values.len()].join(", ");
            Ok((format!("`{column}` IN ({placeholders})"), values.clone()))
        }
        Condition::IsNull { column } => {
            validate_identifier(column)?;
            Ok((format!("`{column}` IS NULL"), Vec::new()))
        }
        Condition::IsNotNull { column } => {
            validate_identifier(column)?;
            Ok((format!("`{column}` IS NOT NULL"), Vec::new()))
        }
        Condition::And(conds) => {
            if conds.is_empty() {
                return Ok(("1=1".into(), Vec::new()));
            }
            let (mut parts, mut params) = (Vec::with_capacity(conds.len()), Vec::new());
            for c in conds {
                let (p, ps) = build_condition(c)?;
                parts.push(p);
                params.extend(ps);
            }
            Ok((format!("({})", parts.join(" AND ")), params))
        }
        Condition::Or(conds) => {
            if conds.is_empty() {
                return Ok(("1=0".into(), Vec::new()));
            }
            let (mut parts, mut params) = (Vec::with_capacity(conds.len()), Vec::new());
            for c in conds {
                let (p, ps) = build_condition(c)?;
                parts.push(p);
                params.extend(ps);
            }
            Ok((format!("({})", parts.join(" OR ")), params))
        }
    }
}



#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::too_many_lines,
    clippy::items_after_statements,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::redundant_clone,
    clippy::needless_pass_by_value,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::unreadable_literal,
    unused_results
)]
mod tests {
    use super::*;
    use db_handler::{
        Condition, Constraint, ConstraintKind, DeleteInput, Filter, Index, InsertInput, UpdateInput,
    };

    fn filter_with(cond: Condition) -> Filter {
        Filter::new().add(cond)
    }


    #[test]
    fn data_type_sql_fixed_variants() {
        let cases = [
            (DataType::Int, "INT"),
            (DataType::BigInt, "BIGINT"),
            (DataType::SmallInt, "SMALLINT"),
            (DataType::Text, "TEXT"),
            (DataType::Boolean, "TINYINT(1)"),
            (DataType::Timestamp, "TIMESTAMP"),
            (DataType::Date, "DATE"),
            (DataType::Float, "FLOAT"),
            (DataType::Double, "DOUBLE"),
            (DataType::Blob, "BLOB"),
        ];
        for (dt, expected) in cases {
            assert_eq!(data_type_sql(dt), expected, "for {dt:?}");
        }
    }

    #[test]
    fn data_type_sql_varchar_embeds_length() {
        assert_eq!(data_type_sql(DataType::Varchar(255)), "VARCHAR(255)");
        assert_eq!(data_type_sql(DataType::Varchar(0)), "VARCHAR(0)");
        assert_eq!(
            data_type_sql(DataType::Varchar(u32::MAX)),
            format!("VARCHAR({})", u32::MAX)
        );
    }


    #[test]
    fn value_to_sql_literal_null() {
        assert_eq!(value_to_sql_literal(&Value::Null).unwrap(), "NULL");
    }

    #[test]
    fn value_to_sql_literal_bool() {
        assert_eq!(value_to_sql_literal(&Value::Bool(true)).unwrap(), "TRUE");
        assert_eq!(value_to_sql_literal(&Value::Bool(false)).unwrap(), "FALSE");
    }

    #[test]
    fn value_to_sql_literal_int_edges() {
        assert_eq!(value_to_sql_literal(&Value::Int(0)).unwrap(), "0");
        assert_eq!(value_to_sql_literal(&Value::Int(42)).unwrap(), "42");
        assert_eq!(value_to_sql_literal(&Value::Int(-1)).unwrap(), "-1");
        assert_eq!(
            value_to_sql_literal(&Value::Int(i64::MAX)).unwrap(),
            i64::MAX.to_string()
        );
        assert_eq!(
            value_to_sql_literal(&Value::Int(i64::MIN)).unwrap(),
            i64::MIN.to_string()
        );
    }

    #[test]
    fn value_to_sql_literal_uint_edges() {
        assert_eq!(value_to_sql_literal(&Value::Uint(0)).unwrap(), "0");
        assert_eq!(
            value_to_sql_literal(&Value::Uint(u64::MAX)).unwrap(),
            u64::MAX.to_string()
        );
    }

    #[test]
    fn value_to_sql_literal_float_finite() {
        assert_eq!(value_to_sql_literal(&Value::Float(1.5)).unwrap(), "1.5");
        assert_eq!(value_to_sql_literal(&Value::Float(-2.5)).unwrap(), "-2.5");
    }

    #[test]
    fn value_to_sql_literal_float_non_finite_rejected() {
        assert!(value_to_sql_literal(&Value::Float(f64::NAN)).is_err());
        assert!(value_to_sql_literal(&Value::Float(f64::INFINITY)).is_err());
        assert!(value_to_sql_literal(&Value::Float(f64::NEG_INFINITY)).is_err());
    }

    #[test]
    fn value_to_sql_literal_string_simple_and_empty() {
        assert_eq!(
            value_to_sql_literal(&Value::String("hi".into())).unwrap(),
            "'hi'"
        );
        assert_eq!(
            value_to_sql_literal(&Value::String(String::new())).unwrap(),
            "''"
        );
    }

    #[test]
    fn value_to_sql_literal_string_escapes_single_quotes() {
        assert_eq!(
            value_to_sql_literal(&Value::String("O'Brien".into())).unwrap(),
            "'O''Brien'"
        );
        assert_eq!(
            value_to_sql_literal(&Value::String("'''".into())).unwrap(),
            "''''''''"
        );
    }

    #[test]
    fn value_to_sql_literal_string_preserves_unicode_and_control() {
        assert_eq!(
            value_to_sql_literal(&Value::String("日本語🔥".into())).unwrap(),
            "'日本語🔥'"
        );
        assert_eq!(
            value_to_sql_literal(&Value::String("a\nb\tc".into())).unwrap(),
            "'a\nb\tc'"
        );
    }

    #[test]
    fn value_to_sql_literal_timestamp_behaves_like_string() {
        assert_eq!(
            value_to_sql_literal(&Value::Timestamp("2026-01-01".into())).unwrap(),
            "'2026-01-01'"
        );
        assert_eq!(
            value_to_sql_literal(&Value::Timestamp("a'b".into())).unwrap(),
            "'a''b'"
        );
    }

    #[test]
    fn value_to_sql_literal_bytes_rejected() {
        let err = value_to_sql_literal(&Value::Bytes(vec![1, 2, 3])).unwrap_err();
        assert!(matches!(err, DbError::Validation { .. }));
    }


    #[test]
    fn column_def_sql_minimal_is_not_null() {
        let cd = ColumnDef::new("id", DataType::Int);
        assert_eq!(column_def_sql(&cd).unwrap(), "`id` INT NOT NULL");
    }

    #[test]
    fn column_def_sql_nullable_omits_not_null() {
        let cd = ColumnDef::new("bio", DataType::Text).nullable(true);
        assert_eq!(column_def_sql(&cd).unwrap(), "`bio` TEXT");
    }

    #[test]
    fn column_def_sql_primary_key_implies_not_null() {
        let cd = ColumnDef::new("id", DataType::BigInt).primary_key();
        let sql = column_def_sql(&cd).unwrap();
        assert!(sql.contains("PRIMARY KEY"));
        assert!(sql.contains("NOT NULL"));
    }

    #[test]
    fn column_def_sql_auto_increment() {
        let cd = ColumnDef::new("id", DataType::BigInt).auto_increment();
        assert!(column_def_sql(&cd).unwrap().contains("AUTO_INCREMENT"));
    }

    #[test]
    fn column_def_sql_unique() {
        let cd = ColumnDef::new("email", DataType::Varchar(255)).unique();
        assert!(column_def_sql(&cd).unwrap().contains("UNIQUE"));
    }

    #[test]
    fn column_def_sql_all_flags_together() {
        let cd = ColumnDef::new("id", DataType::BigInt)
            .primary_key()
            .auto_increment()
            .unique();
        let sql = column_def_sql(&cd).unwrap();
        assert!(sql.contains("PRIMARY KEY"));
        assert!(sql.contains("AUTO_INCREMENT"));
        assert!(sql.contains("UNIQUE"));
        assert!(sql.contains("NOT NULL"));
    }

    #[test]
    fn column_def_sql_default_variants() {
        let cases = [
            (DataType::Int, Value::Int(0), "DEFAULT 0"),
            (DataType::Boolean, Value::Bool(true), "DEFAULT TRUE"),
            (
                DataType::Varchar(100),
                Value::String("anon".into()),
                "DEFAULT 'anon'",
            ),
            (DataType::Text, Value::Null, "DEFAULT NULL"),
        ];
        for (dt, default, expected) in cases {
            let mut cd = ColumnDef::new("c", dt).nullable(true);
            cd.default = Some(default);
            assert!(
                column_def_sql(&cd).unwrap().contains(expected),
                "expected {expected} in {}",
                column_def_sql(&cd).unwrap()
            );
        }
    }

    #[test]
    fn column_def_sql_default_nan_propagates_error() {
        let mut cd = ColumnDef::new("x", DataType::Float).nullable(true);
        cd.default = Some(Value::Float(f64::NAN));
        assert!(column_def_sql(&cd).is_err());
    }

    #[test]
    fn column_def_sql_invalid_name_rejected() {
        for bad in ["", "has space", "1abc", "x;drop"] {
            let cd = ColumnDef::new(bad, DataType::Int);
            assert!(column_def_sql(&cd).is_err(), "name {bad:?} should fail");
        }
    }


    #[test]
    fn build_create_table_single_column() {
        let t = Table::new(
            "users",
            vec![ColumnDef::new("id", DataType::BigInt).primary_key()],
        );
        let sql = build_create_table(&t).unwrap();
        assert!(sql.starts_with("CREATE TABLE `users` ("));
        assert!(sql.contains("`id` BIGINT"));
    }

    #[test]
    fn build_create_table_multiple_columns() {
        let t = Table::new(
            "users",
            vec![
                ColumnDef::new("id", DataType::BigInt).primary_key(),
                ColumnDef::new("email", DataType::Varchar(255)).unique(),
                ColumnDef::new("bio", DataType::Text).nullable(true),
            ],
        );
        let sql = build_create_table(&t).unwrap();
        for col in ["`id`", "`email`", "`bio`"] {
            assert!(sql.contains(col), "missing {col} in {sql}");
        }
    }

    #[test]
    fn build_create_table_empty_columns_rejected() {
        let err = build_create_table(&Table::new("t", vec![])).unwrap_err();
        assert!(matches!(err, DbError::Validation { .. }));
    }

    #[test]
    fn build_create_table_invalid_table_rejected() {
        assert!(build_create_table(&Table::new(
            "bad name",
            vec![ColumnDef::new("id", DataType::Int)],
        ))
        .is_err());
        assert!(build_create_table(&Table::new(
            "",
            vec![ColumnDef::new("id", DataType::Int)],
        ))
        .is_err());
    }

    #[test]
    fn build_create_table_invalid_column_rejected() {
        assert!(build_create_table(&Table::new(
            "t",
            vec![ColumnDef::new("1bad", DataType::Int)],
        ))
        .is_err());
    }


    #[test]
    fn table_change_add_column() {
        let c = TableChange::AddColumn(ColumnDef::new("bio", DataType::Text).nullable(true));
        assert_eq!(table_change_sql(&c).unwrap(), "ADD COLUMN `bio` TEXT");
    }

    #[test]
    fn table_change_drop_column() {
        let c = TableChange::DropColumn("bio".into());
        assert_eq!(table_change_sql(&c).unwrap(), "DROP COLUMN `bio`");
    }

    #[test]
    fn table_change_rename_column() {
        let c = TableChange::RenameColumn {
            from: "a".into(),
            to: "b".into(),
        };
        assert_eq!(table_change_sql(&c).unwrap(), "RENAME COLUMN `a` TO `b`");
    }

    #[test]
    fn table_change_modify_column() {
        let c = TableChange::ModifyColumn(ColumnDef::new("id", DataType::BigInt).nullable(true));
        assert_eq!(table_change_sql(&c).unwrap(), "MODIFY COLUMN `id` BIGINT");
    }

    #[test]
    fn table_change_add_index_variants() {
        let non_unique = TableChange::AddIndex(Index::new("idx", vec!["a".into(), "b".into()]));
        assert_eq!(
            table_change_sql(&non_unique).unwrap(),
            "ADD INDEX `idx` (`a`, `b`)"
        );

        let unique = TableChange::AddIndex(Index::new("idx", vec!["a".into()]).unique());
        assert_eq!(
            table_change_sql(&unique).unwrap(),
            "ADD UNIQUE INDEX `idx` (`a`)"
        );
    }

    #[test]
    fn table_change_drop_index() {
        let c = TableChange::DropIndex("idx".into());
        assert_eq!(table_change_sql(&c).unwrap(), "DROP INDEX `idx`");
    }

    #[test]
    fn table_change_add_constraint_is_unimplemented() {
        let c = TableChange::AddConstraint(Constraint::new("pk", ConstraintKind::PrimaryKey));
        let err = table_change_sql(&c).unwrap_err();
        let DbError::Validation { message } = err else {
            panic!("expected Validation error");
        };
        assert!(message.contains("not yet implemented"), "was {message}");
    }

    #[test]
    fn table_change_drop_constraint() {
        let c = TableChange::DropConstraint("pk".into());
        assert_eq!(table_change_sql(&c).unwrap(), "DROP CONSTRAINT `pk`");
    }

    #[test]
    fn table_change_invalid_identifiers_rejected() {
        assert!(table_change_sql(&TableChange::AddColumn(ColumnDef::new(
            "bad name",
            DataType::Int,
        )))
        .is_err());
        assert!(table_change_sql(&TableChange::AddIndex(Index::new(
            "bad idx",
            vec!["a".into()],
        )))
        .is_err());
        assert!(table_change_sql(&TableChange::AddIndex(Index::new(
            "idx",
            vec!["bad col".into()],
        )))
        .is_err());
    }


    #[test]
    fn build_insert_single_value() {
        let input = InsertInput::new("users").set("name", "Alice");
        let (sql, params) = build_insert(&input).unwrap();
        assert_eq!(sql, "INSERT INTO `users` (`name`) VALUES (?)");
        assert_eq!(params, vec![Value::String("Alice".into())]);
    }

    #[test]
    fn build_insert_multiple_values_preserve_order() {
        let input = InsertInput::new("users")
            .set("name", "Alice")
            .set("age", 30i64)
            .set("active", true);
        let (sql, params) = build_insert(&input).unwrap();
        assert_eq!(
            sql,
            "INSERT INTO `users` (`name`, `age`, `active`) VALUES (?, ?, ?)"
        );
        assert_eq!(
            params,
            vec![
                Value::String("Alice".into()),
                Value::Int(30),
                Value::Bool(true),
            ]
        );
    }

    #[test]
    fn build_insert_preserves_every_value_variant() {
        let input = InsertInput::new("t")
            .set("a", Value::Null)
            .set("b", true)
            .set("c", 42i64)
            .set("d", 42u64)
            .set("e", 1.5f64)
            .set("f", "text")
            .set("g", vec![1u8, 2u8]);
        let (_, params) = build_insert(&input).unwrap();
        assert_eq!(
            params,
            vec![
                Value::Null,
                Value::Bool(true),
                Value::Int(42),
                Value::Uint(42),
                Value::Float(1.5),
                Value::String("text".into()),
                Value::Bytes(vec![1, 2]),
            ]
        );
    }

    #[test]
    fn build_insert_empty_values_rejected() {
        let err = build_insert(&InsertInput::new("users")).unwrap_err();
        assert!(matches!(err, DbError::Validation { .. }));
    }

    #[test]
    fn build_insert_invalid_table_rejected() {
        assert!(build_insert(&InsertInput::new("bad table").set("a", 1i64)).is_err());
    }

    #[test]
    fn build_insert_invalid_column_rejected() {
        assert!(build_insert(&InsertInput::new("t").set("bad col", 1i64)).is_err());
    }


    #[test]
    fn build_update_without_filter() {
        let input = UpdateInput::new("users").set("name", "Bob");
        let (sql, params) = build_update(&input).unwrap();
        assert_eq!(sql, "UPDATE `users` SET `name` = ?");
        assert_eq!(params, vec![Value::String("Bob".into())]);
    }

    #[test]
    fn build_update_appends_where_params_after_set_params() {
        let filter = Filter::new().add(Condition::eq("id", 1i64));
        let input = UpdateInput::new("users").set("name", "Bob").where_(filter);
        let (sql, params) = build_update(&input).unwrap();
        assert_eq!(sql, "UPDATE `users` SET `name` = ? WHERE `id` = ?");
        assert_eq!(params, vec![Value::String("Bob".into()), Value::Int(1)]);
    }

    #[test]
    fn build_update_empty_set_rejected() {
        let err = build_update(&UpdateInput::new("users")).unwrap_err();
        assert!(matches!(err, DbError::Validation { .. }));
    }

    #[test]
    fn build_update_invalid_table_rejected() {
        assert!(build_update(&UpdateInput::new("bad table").set("a", 1i64)).is_err());
    }

    #[test]
    fn build_update_invalid_set_column_rejected() {
        assert!(build_update(&UpdateInput::new("t").set("bad col", 1i64)).is_err());
    }


    #[test]
    fn build_delete_without_filter() {
        let (sql, params) = build_delete(&DeleteInput::new("users")).unwrap();
        assert_eq!(sql, "DELETE FROM `users`");
        assert!(params.is_empty());
    }

    #[test]
    fn build_delete_with_filter() {
        let filter = Filter::new().add(Condition::eq("id", 1i64));
        let input = DeleteInput::new("users").where_(filter);
        let (sql, params) = build_delete(&input).unwrap();
        assert_eq!(sql, "DELETE FROM `users` WHERE `id` = ?");
        assert_eq!(params, vec![Value::Int(1)]);
    }

    #[test]
    fn build_delete_invalid_table_rejected() {
        assert!(build_delete(&DeleteInput::new("bad table")).is_err());
    }

    #[test]
    fn build_delete_invalid_column_rejected() {
        let f = Filter::new().add(Condition::eq("bad col", 1i64));
        assert!(build_delete(&DeleteInput::new("t").where_(f)).is_err());
    }


    #[test]
    fn build_filter_empty_is_empty_string() {
        let (sql, params) = build_filter(&Filter::new()).unwrap();
        assert_eq!(sql, "");
        assert!(params.is_empty());
    }

    #[test]
    fn build_filter_single_eq() {
        let (sql, params) = build_filter(&filter_with(Condition::eq("id", 42i64))).unwrap();
        assert_eq!(sql, " WHERE `id` = ?");
        assert_eq!(params, vec![Value::Int(42)]);
    }

    #[test]
    fn build_filter_comparison_operators() {
        let cases = [
            (
                Condition::Ne {
                    column: "x".into(),
                    value: Value::Int(1),
                },
                "`x` != ?",
            ),
            (
                Condition::Lt {
                    column: "x".into(),
                    value: Value::Int(1),
                },
                "`x` < ?",
            ),
            (
                Condition::Le {
                    column: "x".into(),
                    value: Value::Int(1),
                },
                "`x` <= ?",
            ),
            (
                Condition::Gt {
                    column: "x".into(),
                    value: Value::Int(1),
                },
                "`x` > ?",
            ),
            (
                Condition::Ge {
                    column: "x".into(),
                    value: Value::Int(1),
                },
                "`x` >= ?",
            ),
        ];
        for (cond, expected) in cases {
            let (sql, _) = build_filter(&filter_with(cond)).unwrap();
            assert_eq!(sql, format!(" WHERE {expected}"));
        }
    }

    #[test]
    fn build_filter_in_with_values() {
        let cond = Condition::In {
            column: "id".into(),
            values: vec![Value::Int(1), Value::Int(2), Value::Int(3)],
        };
        let (sql, params) = build_filter(&filter_with(cond)).unwrap();
        assert_eq!(sql, " WHERE `id` IN (?, ?, ?)");
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn build_filter_in_empty_becomes_always_false() {
        let cond = Condition::In {
            column: "id".into(),
            values: vec![],
        };
        let (sql, params) = build_filter(&filter_with(cond)).unwrap();
        assert_eq!(sql, " WHERE 1=0");
        assert!(params.is_empty());
    }

    #[test]
    fn build_filter_is_null_and_not_null() {
        let (sql, p) = build_filter(&filter_with(Condition::is_null("deleted_at"))).unwrap();
        assert_eq!(sql, " WHERE `deleted_at` IS NULL");
        assert!(p.is_empty());

        let (sql, p) = build_filter(&filter_with(Condition::IsNotNull {
            column: "id".into(),
        }))
        .unwrap();
        assert_eq!(sql, " WHERE `id` IS NOT NULL");
        assert!(p.is_empty());
    }

    #[test]
    fn build_filter_empty_and_or_degenerate() {
        let (sql, p) = build_filter(&filter_with(Condition::And(vec![]))).unwrap();
        assert_eq!(sql, " WHERE 1=1");
        assert!(p.is_empty());

        let (sql, p) = build_filter(&filter_with(Condition::Or(vec![]))).unwrap();
        assert_eq!(sql, " WHERE 1=0");
        assert!(p.is_empty());
    }

    #[test]
    fn build_filter_and_or_join_and_collect_params() {
        let and_cond = Condition::And(vec![
            Condition::eq("a", 1i64),
            Condition::eq("b", 2i64),
        ]);
        let (sql, params) = build_filter(&filter_with(and_cond)).unwrap();
        assert_eq!(sql, " WHERE (`a` = ? AND `b` = ?)");
        assert_eq!(params, vec![Value::Int(1), Value::Int(2)]);

        let or_cond = Condition::Or(vec![
            Condition::eq("a", 1i64),
            Condition::eq("b", 2i64),
        ]);
        let (sql, params) = build_filter(&filter_with(or_cond)).unwrap();
        assert_eq!(sql, " WHERE (`a` = ? OR `b` = ?)");
        assert_eq!(params, vec![Value::Int(1), Value::Int(2)]);
    }

    #[test]
    fn build_filter_nested_and_or() {
        let inner = Condition::And(vec![
            Condition::eq("a", 1i64),
            Condition::eq("b", 2i64),
        ]);
        let cond = Condition::Or(vec![inner, Condition::eq("c", 3i64)]);
        let (sql, params) = build_filter(&filter_with(cond)).unwrap();
        assert_eq!(sql, " WHERE ((`a` = ? AND `b` = ?) OR `c` = ?)");
        assert_eq!(params, vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    }

    #[test]
    fn build_filter_multiple_top_level_joined_with_and() {
        let f = Filter::new()
            .add(Condition::eq("a", 1i64))
            .add(Condition::eq("b", 2i64));
        let (sql, params) = build_filter(&f).unwrap();
        assert_eq!(sql, " WHERE `a` = ? AND `b` = ?");
        assert_eq!(params, vec![Value::Int(1), Value::Int(2)]);
    }

    #[test]
    fn build_filter_invalid_column_rejected() {
        assert!(build_filter(&filter_with(Condition::eq("bad col", 1i64))).is_err());
    }

    #[test]
    fn build_filter_nested_invalid_column_rejected() {
        let cond = Condition::And(vec![Condition::eq("bad col", 1i64)]);
        assert!(build_filter(&filter_with(cond)).is_err());
    }


    fn sample_wrapper() -> MySqlRowWrapper {
        MySqlRowWrapper {
            values: vec![
                ("id".to_string(), Value::Int(42)),
                ("name".to_string(), Value::String("Alice".into())),
                ("active".to_string(), Value::Bool(true)),
            ],
            columns: vec![
                Column::new("id", "BIGINT", false),
                Column::new("name", "VARCHAR", false),
                Column::new("active", "TINYINT", false),
            ],
        }
    }

    #[test]
    fn wrapper_get_existing_columns() {
        let w = sample_wrapper();
        assert_eq!(w.get("id").unwrap(), &Value::Int(42));
        assert_eq!(w.get("name").unwrap(), &Value::String("Alice".into()));
        assert_eq!(w.get("active").unwrap(), &Value::Bool(true));
    }

    #[test]
    fn wrapper_get_missing_column_is_validation_error() {
        let err = sample_wrapper().get("nope").unwrap_err();
        let DbError::Validation { message } = err else {
            panic!("expected Validation");
        };
        assert!(message.contains("nope"), "message was {message}");
    }

    #[test]
    fn wrapper_get_is_case_sensitive() {
        let w = sample_wrapper();
        assert!(w.get("ID").is_err());
        assert!(w.get("Id").is_err());
        assert!(w.get("id").is_ok());
    }

    #[test]
    fn wrapper_returns_null_values() {
        let w = MySqlRowWrapper {
            values: vec![("x".to_string(), Value::Null)],
            columns: vec![Column::new("x", "INT", true)],
        };
        assert_eq!(w.get("x").unwrap(), &Value::Null);
    }

    #[test]
    fn wrapper_len_and_columns_agree() {
        let w = sample_wrapper();
        assert_eq!(w.len(), 3);
        assert_eq!(w.columns().len(), 3);
    }

    #[test]
    fn wrapper_is_empty() {
        let empty = MySqlRowWrapper {
            values: Vec::new(),
            columns: Vec::new(),
        };
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert!(!sample_wrapper().is_empty());
    }

    #[test]
    fn wrapper_columns_preserves_order() {
        let w = sample_wrapper();
        let names: Vec<&str> = w.columns().iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["id", "name", "active"]);
    }

    #[test]
    fn wrapper_duplicate_column_returns_first_match() {
        let w = MySqlRowWrapper {
            values: vec![
                ("x".to_string(), Value::Int(1)),
                ("x".to_string(), Value::Int(2)),
            ],
            columns: vec![
                Column::new("x", "INT", true),
                Column::new("x", "INT", true),
            ],
        };
        assert_eq!(w.get("x").unwrap(), &Value::Int(1));
    }

    #[test]
    fn wrapper_get_empty_name_matches_only_empty_string() {
        let w = MySqlRowWrapper {
            values: vec![(String::new(), Value::Int(1))],
            columns: vec![Column::new("", "INT", true)],
        };
        assert_eq!(w.get("").unwrap(), &Value::Int(1));
        assert!(w.get("x").is_err());
    }
}
