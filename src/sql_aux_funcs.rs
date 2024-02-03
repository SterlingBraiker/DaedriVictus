/*

    SQL library
        Sqlite3
        ODBC

*/

use crate::odbc_interface::*;

//why am i importing this as Sqlite3Error? I have Sqlite3error defined within crate::sqlite3_interface
//use crate::sqlite3_interface::Error as Sqlite3Error;

use crate::sqlite3_interface::*;
use odbc::*;
use std::collections::HashMap;

/* --> Structs */

// Connection details & reference the recordset
pub struct Connection<RecordSet> {
    pub record_set: Option<RecordSet>,
    pub connection: Option<String>,
    pub result_code: i32,
    pub result_details: Option<String>,
    pub connection_type: ConnectionBase,
}

// What kind of DB are we connecting to?
pub enum ConnectionBase {
    Odbc,
    Sqlite,
}

// Handles all results (records, errors)
#[derive(Clone)]
pub struct RecordSet<SqlData, SqlType, SqlError> {
    pub column_info: HashMap<String, SqlType>,
    pub column_order: Vec<String>,
    pub records: Vec<Record<SqlData>>,
    pub error_interface: SqlError,
}

#[derive(Clone, Default)]
pub struct Record<SqlData> {
    pub columns: HashMap<String, SqlData>,
}

pub enum SqlData {
    Sqlite(sqlite::Value),
    Odbc(String),
}

pub enum SqlType {
    Sqlite(sqlite::Value),
    Odbc(odbc::ffi::SqlDataType),
}

pub enum SqlError {
    Sqlite(Sqlite3error),
    Odbc(OdbcDiagnosticRecord),
    None,
}

impl Default for SqlError {
    fn default() -> Self {
        SqlError::None
    }
}

impl RecordSet<sqlite::Value, sqlite::Type, Sqlite3error> {
    pub fn construct(
        &mut self,
        stmt: &mut sqlite::Statement,
    ) -> std::result::Result<(), sqlite::Error> {
        stmt.next()?;

        for name in stmt.column_names() {
            self.column_info.insert(
                String::from(&name[..]),
                stmt.column_type(&String::from(&name[..])[..])?,
            );
            self.column_order.push(String::from(&name[..]));
        }
        stmt.reset()
    } //fill fields 'column_count', 'column_info'

    pub fn add(&mut self, rec: Record<sqlite::Value>) {
        self.records.push(rec);
    } //insert a record into the recordset

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn column_count(&self) -> usize {
        self.records.first().unwrap().columns.len()
    }

    pub fn fetch_page_of_records(&self, page: usize) -> Vec<Record<sqlite::Value>> {
        let range_upper: usize = if (page * 50) < self.records.len() {
            page * 50
        } else {
            self.records.len()
        };
        let range_lower: usize = if range_upper > 50 {
            range_upper - 50
        } else {
            0
        };
        Vec::from(&self.records[range_lower..range_upper])
    }
}

impl<T, U, E: std::default::Default> Default for RecordSet<T, U, E> {
    fn default() -> Self {
        Self {
            column_info: HashMap::<String, U>::new(),
            column_order: Vec::<String>::new(),
            records: Vec::<Record<T>>::new(),
            error_interface: E::default(),
        }
    }
}

impl RecordSet<String, odbc::ffi::SqlDataType, DiagnosticRecord> {
    pub fn construct(
        &mut self,
        stmt: &mut odbc::Statement<'_, '_, Allocated, HasResult, odbc_safe::AutocommitOn>,
    ) -> std::result::Result<(), odbc::DiagnosticRecord> {
        for col_index in 1..stmt.num_result_cols()? {
            let col_description: ColumnDescriptor = stmt.describe_col(col_index as u16)?;
            self.column_info.insert(
                String::from(col_description.name.clone()),
                col_description.data_type,
            );
            self.column_order
                .push(String::from(col_description.name.clone()));
        }
        Ok(())
    } //fill fields 'column_count', 'column_info'

    pub fn add(&mut self, rec: Record<String>) {
        self.records.push(rec);
    } //insert a record into the recordset

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn column_count(&self) -> usize {
        self.records.first().unwrap().columns.len()
    }

    pub fn fetch_page_of_records(&self, page: usize) -> Vec<Record<String>> {
        let range_upper: usize = if (page * 50) < self.records.len() {
            page * 50
        } else {
            self.records.len()
        };
        let range_lower: usize = if range_upper > 50 {
            range_upper - 50
        } else {
            0
        };
        Vec::from(&self.records[range_lower..range_upper])
    }
}

impl<T> Record<T> {
    pub fn add(&mut self, key: String, val: T) {
        self.columns.insert(key, val);
    }
}

impl Record<String> {
    pub fn construct(&mut self, columns: &HashMap<String, odbc::ffi::SqlDataType>) {
        for key in columns.keys() {
            self.columns.insert(key.clone(), String::new());
        }
    }
}

/*
//odbc impl
impl Connection<RecordSet<String, odbc::ffi::SqlDataType>> {
    pub fn assemble_err(&mut self, e_msg: DiagnosticRecord) {
        self.result_code = -1;
        self.result_details = Some(String::from_utf8(e_msg.get_raw_message().to_owned()).unwrap());
    }
}

//sqlite impl
impl Connection<RecordSet<sqlite::Value, sqlite::Type>> {
    pub fn assemble_err(&mut self, E: sqlite::Error) {
        self.result_code = -1;
        self.result_details = E.message;
    }
}
 */
impl<SqlData, SqlType, SqlError> Connection<RecordSet<SqlData, SqlType, SqlError>> {
    pub fn assemble_rs(&mut self, donor_rs: RecordSet<SqlData, SqlType, SqlError>) {
        self.record_set = Some(donor_rs);
        self.result_code = 1;
    }

    pub fn assemble_err(&mut self, the_error: SqlError) -> () {
        self.result_code = -1;
        self.result_details = match self {
            Odbc => Some(String::from("Text")),
            Sqlite => Some(String::from("Text")),
            //            Some(crate::sql_aux_funcs::SqlError::Odbc(ref E)) => Some(E.message_string().clone()),
            //            Some(crate::sql_aux_funcs::SqlError::Sqlite(ref E)) => E.message.clone(),
            //            None => {}
        }
    }
}

/* <-- Structs */
/* --> Traits */

pub trait Translate {
    fn translate(&self) -> String;
}

impl Translate for sqlite::Value {
    fn translate(&self) -> String {
        let mut payload: String = String::new();

        match self {
            SqliteFloat(value) => payload.push_str(&value.to_string()),
            SqliteInteger(value) => payload.push_str(&value.to_string()),
            SqliteString(value) => payload.push_str(&value.to_string()),
            SqliteBinary(value) => {
                for element in value {
                    payload.push_str(&element.to_string())
                }
            }
            SqliteNull => payload.push_str("Null"),
        }

        payload
    }
}

impl Translate for String {
    fn translate(&self) -> String {
        self.clone()
    }
}

/*
impl Translate for odbc_interface::odbc::OdbcType {
    fn translate(&self) -> String {

    }
}
 */
/* <-- Traits */
