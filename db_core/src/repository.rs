use crate::Database;
use async_trait::async_trait;
use db_handler::{
    Condition, CrudRepository, DbError, Executor, Filter, InsertInput, PageMeta, PageRequest,
    Paginated, Query, Result, Row, UpdateInput, Value,
};

pub struct EntityRepository<T, F>
where
    F: Fn(&dyn Row) -> Result<T> + Send + Sync,
{
    db: Database,
    table: String,
    pk: String,
    mapper: F,
}

impl<T, F> std::fmt::Debug for EntityRepository<T, F>
where
    F: Fn(&dyn Row) -> Result<T> + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityRepository")
            .field("db", &self.db)
            .field("table", &self.table)
            .field("pk", &self.pk)
            .finish_non_exhaustive()
    }
}

impl<T, F> EntityRepository<T, F>
where
    F: Fn(&dyn Row) -> Result<T> + Send + Sync,
{
    pub fn new(db: Database, table: impl Into<String>, pk: impl Into<String>, mapper: F) -> Self {
        Self {
            db,
            table: table.into(),
            pk: pk.into(),
            mapper,
        }
    }
}

#[async_trait]
impl<T, F> CrudRepository<T> for EntityRepository<T, F>
where
    T: Send + Sync + 'static,
    F: Fn(&dyn Row) -> Result<T> + Send + Sync,
{
    async fn create(&self, input: InsertInput) -> Result<T> {
        let (sql, params) = crate::internal::build_insert(&input)?;
        let mut query = Query::new(sql);
        for p in params {
            query = query.bind(p);
        }
        let res = self.db.execute(query).await?;
        let id = res.last_insert_id.ok_or_else(|| DbError::Database {
            message: "no last insert id returned".into(),
        })?;
        self.get_by_id(&Value::Uint(id))
            .await?
            .ok_or(DbError::NotFound)
    }

    async fn get_by_id(&self, id: &Value) -> Result<Option<T>> {
        let sql = format!(
            "SELECT * FROM `{}` WHERE `{}` = ? LIMIT 1",
            self.table, self.pk
        );
        let query = Query::new(sql).bind(id.clone());
        let row = self.db.fetch_optional(query).await?;
        match row {
            Some(r) => Ok(Some((self.mapper)(r.as_ref())?)),
            None => Ok(None),
        }
    }

    async fn update(&self, id: &Value, input: UpdateInput) -> Result<T> {
        let mut update_input = input;
        update_input.filter = std::mem::take(&mut update_input.filter)
            .add(Condition::eq(self.pk.as_str(), id.clone()));
        let (sql, params) = crate::internal::build_update(&update_input)?;
        let mut query = Query::new(sql);
        for p in params {
            query = query.bind(p);
        }
        self.db.execute(query).await.map(|_| ())?;
        self.get_by_id(id).await?.ok_or(DbError::NotFound)
    }

    async fn delete(&self, id: &Value) -> Result<()> {
        let filter = Filter::new().add(Condition::eq(self.pk.as_str(), id.clone()));
        let input = db_handler::DeleteInput::new(self.table.as_str()).where_(filter);
        let (sql, params) = crate::internal::build_delete(&input)?;
        let mut query = Query::new(sql);
        for p in params {
            query = query.bind(p);
        }
        self.db.execute(query).await.map(|_| ())?;
        Ok(())
    }

    async fn list(&self, filter: Filter, page: PageRequest) -> Result<Paginated<T>> {
        let (where_sql, params) = crate::internal::build_filter(&filter)?;

        let count_sql = format!("SELECT COUNT(*) AS cnt FROM `{}`{}", self.table, where_sql);
        let mut count_query = Query::new(count_sql);
        for p in &params {
            count_query = count_query.bind(p.clone());
        }
        let count_row = self.db.fetch_one(count_query).await?;
        let count_i64 = count_row.get("cnt")?.as_i64().unwrap_or(0);
        let total = u64::try_from(count_i64).unwrap_or(0);

        let select_sql = format!("SELECT * FROM `{}`{}", self.table, where_sql);
        let mut select_query = Query::new(select_sql);
        for p in params {
            select_query = select_query.bind(p);
        }
        let paginated_rows = self.db.fetch_paginated(select_query, page).await?;

        let mut items = Vec::with_capacity(paginated_rows.items.len());
        for r in paginated_rows.items {
            items.push((self.mapper)(r.as_ref())?);
        }

        Ok(Paginated {
            items,
            meta: PageMeta::new(page.page, page.per_page, total),
        })
    }

    async fn count(&self, filter: Filter) -> Result<u64> {
        let (where_sql, params) = crate::internal::build_filter(&filter)?;
        let sql = format!("SELECT COUNT(*) AS cnt FROM `{}`{}", self.table, where_sql);
        let mut query = Query::new(sql);
        for p in params {
            query = query.bind(p);
        }
        let row = self.db.fetch_one(query).await?;
        let count_i64 = row.get("cnt")?.as_i64().unwrap_or(0);
        Ok(u64::try_from(count_i64).unwrap_or(0))
    }
}
