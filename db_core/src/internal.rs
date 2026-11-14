use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use db_handler::validation::validate_identifier;
use db_handler::{Column, ColumnDef, DataType, DbError, Result, Row, Table, TableChange, Value};
use sqlx::mysql::{MySql, MySqlArguments, MySqlRow};
use sqlx::query::Query as SqlxQuery;
use sqlx::TypeInfo;
use sqlx::{Column as _, Row as _};

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
        let mut values = Vec::new();
        let mut columns = Vec::new();
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
        Value::Float(f) => f.to_string(),
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
    let mut parts = Vec::new();
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
    let mut cols = Vec::new();
    let mut placeholders = Vec::new();
    let mut params = Vec::new();
    for cv in &input.values {
        validate_identifier(&cv.column)?;
        cols.push(format!("`{}`", cv.column));
        placeholders.push("?".to_string());
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
    let mut sets = Vec::new();
    let mut params = Vec::new();
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
    let mut parts = Vec::new();
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
            Ok((format!("`{}` = ?", column), vec![value.clone()]))
        }
        Condition::Ne { column, value } => {
            validate_identifier(column)?;
            Ok((format!("`{}` != ?", column), vec![value.clone()]))
        }
        Condition::Lt { column, value } => {
            validate_identifier(column)?;
            Ok((format!("`{}` < ?", column), vec![value.clone()]))
        }
        Condition::Le { column, value } => {
            validate_identifier(column)?;
            Ok((format!("`{}` <= ?", column), vec![value.clone()]))
        }
        Condition::Gt { column, value } => {
            validate_identifier(column)?;
            Ok((format!("`{}` > ?", column), vec![value.clone()]))
        }
        Condition::Ge { column, value } => {
            validate_identifier(column)?;
            Ok((format!("`{}` >= ?", column), vec![value.clone()]))
        }
        Condition::In { column, values } => {
            validate_identifier(column)?;
            if values.is_empty() {
                return Ok(("1=0".into(), Vec::new()));
            }
            let placeholders = values.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            Ok((
                format!("`{}` IN ({})", column, placeholders),
                values.clone(),
            ))
        }
        Condition::IsNull { column } => {
            validate_identifier(column)?;
            Ok((format!("`{}` IS NULL", column), Vec::new()))
        }
        Condition::IsNotNull { column } => {
            validate_identifier(column)?;
            Ok((format!("`{}` IS NOT NULL", column), Vec::new()))
        }
        Condition::And(conds) => {
            if conds.is_empty() {
                return Ok(("1=1".into(), Vec::new()));
            }
            let (mut parts, mut params) = (Vec::new(), Vec::new());
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
            let (mut parts, mut params) = (Vec::new(), Vec::new());
            for c in conds {
                let (p, ps) = build_condition(c)?;
                parts.push(p);
                params.extend(ps);
            }
            Ok((format!("({})", parts.join(" OR ")), params))
        }
    }
}
