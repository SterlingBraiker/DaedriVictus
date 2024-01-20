/*

    SQL library
        Sqlite3
        ODBC

*/

use crate::odbc_interface;
use odbc::*;
use crate::sqlite3_interface::*;
use std::collections::HashMap;

/* --> Structs */

pub struct Connection<T> {
    pub record_set: T,
    pub connection: Connectable,
    pub result_code: i32,
    pub result_details: Option<String>,
}

#[derive(Clone)]
pub struct RecordSet<T, U> {
    pub column_info: HashMap<String, U>,
    pub column_order: Vec<String>,
    pub records: Vec<Record<T>>,
}

#[derive(Clone, Default)]
pub struct Record<T> {
    pub columns: HashMap<String, T>,
}

#[derive(Debug, PartialEq)]
pub enum Connectable {
    Sqlite3(String),
    Odbc(String),
    None,
}

impl RecordSet<sqlite::Value, sqlite::Type> {
    pub fn construct(&mut self, stmt: &mut sqlite::Statement) -> std::result::Result<(), sqlite::Error> {
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

impl Default for RecordSet<sqlite::Value, sqlite::Type> {
    fn default() -> Self {
        Self {
            column_info: HashMap::<String, sqlite::Type>::new(),
            column_order: Vec::<String>::new(),
            records: Vec::<Record<sqlite::Value>>::new(),
        }
    }
}

impl RecordSet<String, odbc::ffi::SqlDataType> {
    pub fn construct(&mut self, stmt: &mut odbc::Statement<'_, '_, Allocated, HasResult, odbc_safe::AutocommitOn>) -> std::result::Result<(), odbc::DiagnosticRecord> {

        for col_index in 1..stmt.num_result_cols()? {
            let col_description: ColumnDescriptor = stmt.describe_col(col_index as u16)?;
            self.column_info.insert(
                String::from(col_description.name.clone()),
                col_description.data_type,
            );
            self.column_order.push(String::from(col_description.name.clone()));
        };
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

impl Default for RecordSet<String, odbc::ffi::SqlDataType> {
    fn default() -> Self {
        Self {
            column_info: HashMap::<String, odbc::ffi::SqlDataType>::new(),
            column_order: Vec::<String>::new(),
            records: Vec::<Record<String>>::new(),
        }
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

//odbc impl
impl Connection<RecordSet<String, odbc::ffi::SqlDataType>> {
    pub fn assemble_rs(&mut self, donor_rs: RecordSet<String, odbc::ffi::SqlDataType>) {
        self.record_set = donor_rs;
        self.result_code = 1;
    }

    pub fn assemble_err(&mut self, e_msg: DiagnosticRecord) {
        self.result_code = -1;
        self.result_details = Some(String::from_utf8(e_msg.get_raw_message().to_owned()).unwrap());
    }
}

//sqlite impl
impl Connection<RecordSet<sqlite::Value, sqlite::Type>> {
    pub fn assemble_rs(&mut self, donor_rs: RecordSet<sqlite::Value, sqlite::Type>) {
        self.record_set = donor_rs;
        self.result_code = 1;
    }

    pub fn assemble_err(&mut self, E: sqlite::Error) {
        self.result_code = -1;
        self.result_details = E.message;
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
