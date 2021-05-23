use crate::{table::TableQueryable, Table};

pub mod delete;
use delete::Delete;

pub mod filter;
use filter::Filter;
pub use filter::Filterable;

pub mod insert;
pub use insert::Insertable;
use insert::{InsertStatement, Values};

pub mod predicate;
pub use predicate::Predicate;
use predicate::{And, Or};

pub mod select;
use select::queryable::{Count, Queryable, WildCard};
use select::{GroupBy, GroupOrder, Limit, Order, OrderBy, SelectStatement, Selectable};
pub use select::{Join, Joined, Select};

pub mod update;
use update::{Update, UpdateSet};

pub trait Query: Sized {
    /// # Examples
    /// ```
    /// use typed_sql::{Query, Table, ToSql};
    ///
    /// #[derive(Table)]
    /// struct Post {
    ///     content: String
    /// }
    ///
    /// let stmt = Post::table().select().filter(|p| p.content.eq("foo"));
    ///
    /// assert_eq!(stmt.to_sql(), "SELECT * FROM posts WHERE posts.content = 'foo';");
    /// ```
    fn select(self) -> SelectStatement<Self, WildCard>
    where
        Self: Selectable,
    {
        self.query(WildCard)
    }

    fn query<Q>(self, query: Q) -> SelectStatement<Self, Q>
    where
        Self: Selectable,
        Q: Queryable,
    {
        SelectStatement::new(self, query)
    }

    /// # Examples
    /// ```
    /// use typed_sql::{Query, Table, ToSql};
    ///
    /// #[derive(Table)]
    /// struct Post {
    ///    content: Option<String>
    /// }
    ///
    /// let stmt = Post::table().count(|post| post.content);
    ///
    /// assert_eq!(stmt.to_sql(), "SELECT COUNT(posts.content) FROM posts;");
    /// ```
    /// ## Wildcard
    /// ```
    /// use typed_sql::{Query, Table, ToSql};
    ///
    /// #[derive(Table)]
    /// struct Post {}
    ///
    /// let stmt = Post::table().count(|_| {});
    ///
    /// assert_eq!(stmt.to_sql(), "SELECT COUNT(*) FROM posts;");
    /// ```
    fn count<F, T>(self, f: F) -> SelectStatement<Self, Count<T>>
    where
        Self: Selectable,
        F: FnOnce(Self::Fields) -> T,
        Count<T>: Queryable,
    {
        self.query(Count::new(f(Default::default())))
    }

    /// ```
    /// use typed_sql::{Query, Table};
    ///
    /// #[derive(Table)]
    /// struct User {
    ///     id: i64,
    ///     name: String
    /// }
    ///
    /// struct UserInsert {}
    /// ```
    fn insert<I>(self, value: I) -> InsertStatement<Self::Table, I>
    where
        Self: TableQueryable,
        I: Insertable,
    {
        InsertStatement::new(value)
    }

    fn insert_values<I>(self, values: I) -> InsertStatement<Self::Table, Values<I>>
    where
        Self: TableQueryable,
        I: IntoIterator + Clone,
        I::Item: Insertable,
    {
        InsertStatement::new(Values::new(values))
    }

    /// ```
    /// use typed_sql::{Query, Table, ToSql};
    ///
    /// #[derive(Table)]
    /// struct Post {
    ///     id: i64,
    ///     name: String
    /// }
    ///
    /// let stmt = Post::table()
    ///     .update(|p| p.id.eq(2).and(p.name.eq("foo")))
    ///     .filter(|p| p.id.eq(1));
    ///
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "UPDATE posts \
    ///     SET posts.id = 2,posts.name = 'foo' \
    ///     WHERE posts.id = 1;"
    /// );
    /// ```
    fn update<F, S>(self, f: F) -> Update<Self::Table, S>
    where
        Self: TableQueryable,
        F: FnOnce(<Self::Table as Table>::Fields) -> S,
        S: UpdateSet,
    {
        Update::new(f(Default::default()))
    }

    /// ```
    /// use typed_sql::{Query, Table, ToSql};
    ///
    /// #[derive(Table)]
    /// struct Post {
    ///     id: i64
    /// }
    ///
    /// let stmt = Post::table().delete().filter(|p| p.id.eq(2));
    ///
    /// assert_eq!(stmt.to_sql(), "DELETE FROM posts WHERE posts.id = 2;");
    /// ```
    fn delete(self) -> Delete<Self::Table>
    where
        Self: TableQueryable,
    {
        Delete::new()
    }

    fn filter<F, P>(self, f: F) -> Filter<Self, P>
    where
        Self: Filterable,
        F: FnOnce(Self::Fields) -> P,
    {
        Filter::new(self, f(Default::default()))
    }

    /// ```
    /// use typed_sql::{Query, Table, ToSql};
    ///
    /// #[derive(Table)]
    /// struct User {
    ///     id: i64   
    /// }
    ///
    /// let stmt = User::table().select().filter(|user| user.id.neq(2).and(user.id.lt(5)));
    ///
    /// assert_eq!(stmt.to_sql(), "SELECT * FROM users WHERE users.id != 2 AND users.id < 5;");
    /// ```
    fn and<P>(self, predicate: P) -> And<Self, P>
    where
        Self: Predicate,
        P: Predicate,
    {
        And {
            head: self,
            tail: predicate,
        }
    }

    /// ```
    /// use typed_sql::{Query, Table, ToSql};
    ///
    /// #[derive(Table)]
    /// struct User {
    ///     id: i64   
    /// }
    ///
    /// let stmt = User::table().select().filter(|user| user.id.eq(1).or(user.id.eq(3)));
    ///
    /// assert_eq!(stmt.to_sql(), "SELECT * FROM users WHERE users.id = 1 OR users.id = 3;");
    fn or<P>(self, predicate: P) -> Or<Self, P>
    where
        Self: Predicate,
        P: Predicate,
    {
        Or {
            head: self,
            tail: predicate,
        }
    }

    /// # Examples
    /// ```
    /// use typed_sql::{Table, ToSql, Query};
    ///
    /// #[derive(Table)]
    /// struct User {
    ///     id: i64
    /// }
    ///
    /// let stmt = User::table().select().group_by(|user| user.id);
    ///
    /// assert_eq!(stmt.to_sql(), "SELECT * FROM users GROUP BY users.id;");
    /// ```
    /// ## Multiple columns
    /// ```
    /// use typed_sql::{Query, Table, ToSql};
    ///
    /// #[derive(Table)]
    /// struct User {
    ///     id: i64,
    ///     name: String
    /// }
    ///
    /// let stmt = User::table().select().group_by(|user| user.id.then(user.name));
    ///
    /// assert_eq!(stmt.to_sql(), "SELECT * FROM users GROUP BY users.id,users.name;");
    /// ```
    fn group_by<F, O>(self, f: F) -> GroupBy<Self, O>
    where
        Self: Select,
        F: FnOnce(<Self::Selectable as Selectable>::Fields) -> O,
        O: GroupOrder,
    {
        GroupBy::new(self, f(Default::default()))
    }

    /// # Examples
    /// ```
    /// use typed_sql::{Query, Table, ToSql};
    ///
    /// #[derive(Table)]
    /// struct User {
    ///     id: i64,
    ///     name: String
    /// }
    ///
    /// let stmt = User::table().select().order_by(|user| user.id);
    ///
    /// assert_eq!(stmt.to_sql(), "SELECT * FROM users ORDER BY users.id;");
    /// ```
    /// ## Direction
    /// ```
    /// use typed_sql::{Query, Table, ToSql};
    ///
    /// #[derive(Table)]
    /// struct User {
    ///     id: i64
    /// }
    ///
    /// let stmt = User::table().select().order_by(|user| user.id.ascending());
    ///
    /// assert_eq!(stmt.to_sql(), "SELECT * FROM users ORDER BY users.id ASC;");
    /// ```
    /// ## Multiple columns
    /// ```
    /// use typed_sql::{Query, Table, ToSql};
    ///
    /// #[derive(Table)]
    /// struct User {
    ///     id: i64,
    ///     name: String
    /// }
    ///
    /// let stmt = User::table().select()
    ///     .order_by(|user| user.id.ascending().then(user.name.descending()));
    ///
    /// assert_eq!(stmt.to_sql(), "SELECT * FROM users ORDER BY users.id ASC,users.name DESC;");
    /// ```
    fn order_by<F, O>(self, f: F) -> OrderBy<Self, O>
    where
        Self: Select,
        F: FnOnce(<Self::Selectable as Selectable>::Fields) -> O,
        O: Order,
    {
        OrderBy::new(self, f(Default::default()))
    }

    fn limit(self, limit: usize) -> Limit<Self>
    where
        Self: Select,
    {
        Limit::new(self, limit)
    }
}

impl<T> Query for T {}
