use diesel::{
    pg::Pg, prelude::*, query_builder::*, query_dsl::methods::LoadQuery, sql_types::BigInt,
};

use crate::db::Connection;

pub trait Paginate: Sized {
    fn paginate(self, limit: i64, offset: i64) -> Pagination<Self>;
}

impl<T> Paginate for T {
    fn paginate(self, limit: i64, offset: i64) -> Pagination<Self> {
        Pagination {
            query: self,
            limit,
            offset,
        }
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct Pagination<T> {
    query: T,
    offset: i64,
    limit: i64,
}

impl<T> Pagination<T> {
    pub fn load_with_pagination<U>(self, conn: &Connection) -> QueryResult<(Vec<U>, i64)>
    where
        Self: LoadQuery<Connection, (U, i64)>,
    {
        let results = self.load::<(U, i64)>(conn)?;
        let total = results.get(0).map(|x| x.1).unwrap_or(0);
        let records = results.into_iter().map(|x| x.0).collect();
        Ok((records, total))
    }
}

impl<T: Query> Query for Pagination<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T> RunQueryDsl<PgConnection> for Pagination<T> {}

impl<T> QueryFragment<Pg> for Pagination<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") t LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.limit)?;
        out.push_sql(" OFFSET ");
        let offset = self.offset;
        out.push_bind_param::<BigInt, _>(&offset)?;
        Ok(())
    }
}
