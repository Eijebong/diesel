use std::error::Error;

use diesel::prelude::*;
use diesel::result::Error::NotFound;

use table_data::TableData;
use data_structures::{ColumnInformation, ColumnType, ForeignKeyConstraint};

pub enum InferConnection {
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteConnection),
    #[cfg(feature = "postgres")]
    Pg(PgConnection),
    #[cfg(feature = "mysql")]
    Mysql(MysqlConnection),
}

pub fn load_table_names(database_url: &str, schema_name: Option<&str>)
    -> Result<Vec<TableData>, Box<Error>>
{
    let connection = try!(establish_connection(database_url));

    match connection {
        #[cfg(feature = "sqlite")]
        InferConnection::Sqlite(c) => ::sqlite::load_table_names(&c, schema_name),
        #[cfg(feature = "postgres")]
        InferConnection::Pg(c) => ::information_schema::load_table_names(&c, schema_name),
        #[cfg(feature = "mysql")]
        InferConnection::Mysql(c) => ::information_schema::load_table_names(&c, schema_name),
    }
}

pub fn establish_connection(database_url: &str) -> Result<InferConnection, Box<Error>> {
    match database_url {
        #[cfg(feature = "postgres")]
        _ if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") => {
            establish_real_connection(database_url).map(InferConnection::Pg)
        }
        #[cfg(feature = "mysql")]
        _ if database_url.starts_with("mysql://") => {
            establish_real_connection(database_url).map(InferConnection::Mysql)
        }
        #[cfg(feature = "sqlite")]
        _ => establish_real_connection(database_url).map(InferConnection::Sqlite),
        #[cfg(all(feature = "postgres", not(feature = "sqlite")))]
        _ => {
            Err(format!(
                "{} is not a valid PG database URL. \
                It must start with postgres:// or postgresql://",
                database_url,
            ).into())
        }
        #[cfg(all(feature = "mysql", not(any(feature = "sqlite", feature = "postgres"))))]
        _ => {
            Err(format!(
                "{} is not a valid MySQL database URL. \
                It must start with mysql://",
                database_url,
            ).into())
        }
    }
}

fn establish_real_connection<Conn>(database_url: &str) -> Result<Conn, Box<Error>> where
    Conn: Connection,
{
    Conn::establish(database_url).map_err(|error| {
        format!(
            "Failed to establish a database connection at {}. Error: {:?}",
            database_url,
            error,
        ).into()
    })
}

pub fn get_table_data(conn: &InferConnection, table: &TableData)
    -> Result<Vec<ColumnInformation>, Box<Error>>
{
    let column_info = match *conn {
        #[cfg(feature = "sqlite")]
        InferConnection::Sqlite(ref c) => ::sqlite::get_table_data(c, table),
        #[cfg(feature = "postgres")]
        InferConnection::Pg(ref c) => ::information_schema::get_table_data(c, table),
        #[cfg(feature = "mysql")]
        InferConnection::Mysql(ref c) => ::information_schema::get_table_data(c, table),
    };
    if let Err(NotFound) = column_info {
        Err(format!("no table exists named {}", table.to_string()).into())
    } else {
        column_info.map_err(Into::into)
    }
}

pub fn determine_column_type(
    attr: &ColumnInformation,
    conn: &InferConnection,
) -> Result<ColumnType, Box<Error>> {
    match *conn {
        #[cfg(feature = "sqlite")]
        InferConnection::Sqlite(_) => ::sqlite::determine_column_type(attr),
        #[cfg(feature = "postgres")]
        InferConnection::Pg(_) => ::pg::determine_column_type(attr),
        #[cfg(feature = "mysql")]
        InferConnection::Mysql(_) => ::mysql::determine_column_type(attr),
    }
}

pub fn get_primary_keys(
    conn: &InferConnection,
    table: &TableData,
) -> Result<Vec<String>, Box<Error>> {
    let primary_keys: Vec<String> = try!(match *conn {
        #[cfg(feature = "sqlite")]
        InferConnection::Sqlite(ref c) => ::sqlite::get_primary_keys(c, table),
        #[cfg(feature = "postgres")]
        InferConnection::Pg(ref c) => ::information_schema::get_primary_keys(c, table),
        #[cfg(feature = "mysql")]
        InferConnection::Mysql(ref c) => ::information_schema::get_primary_keys(c, table),
    });
    if primary_keys.is_empty() {
        Err(format!("Diesel only supports tables with primary keys. \
                    Table {} has no primary key", table.to_string()).into())
    } else if primary_keys.len() > 5 {
        Err(format!("Diesel does not currently support tables with \
                     primary keys consisting of more than 5 columns. \
                     Table {} has {} columns in its primary key. \
                     Please open an issue and we will increase the \
                     limit.", table.to_string(), primary_keys.len()).into())
    } else {
        Ok(primary_keys)
    }
}

pub fn load_foreign_key_constraints(database_url: &str, schema_name: Option<&str>)
    -> Result<Vec<ForeignKeyConstraint>, Box<Error>>
{
    let connection = try!(establish_connection(database_url));

    match connection {
        #[cfg(feature = "sqlite")]
        InferConnection::Sqlite(c) => ::sqlite::load_foreign_key_constraints(&c, schema_name),
        #[cfg(feature = "postgres")]
        InferConnection::Pg(c) => ::information_schema::load_foreign_key_constraints(&c, schema_name).map_err(Into::into),
        #[cfg(feature = "mysql")]
        InferConnection::Mysql(c) => ::mysql::load_foreign_key_constraints(&c, schema_name).map_err(Into::into),
    }
}
