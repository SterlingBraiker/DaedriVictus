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
pub struct Connection {
    pub record_set: Option<RecordSet>,
    pub connection: Option<String>,
    pub result_code: Option<i32>,
    pub result_details: Option<String>,
    pub connection_type: Option<ConnectionBase>,
    pub error_interface: Option<SqlError>,
}

// Handles all results (records, errors)
#[derive(Clone)]
pub struct RecordSet {
    pub column_info: HashMap<String, SqlType>,
    pub column_order: Vec<String>,
    pub records: Vec<Record>,
}

#[derive(Clone, Default)]
pub struct Record {
    pub columns: HashMap<String, Option<SqlData>>,
    pub data_type: Option<ConnectionBase>,
}

#[derive(Clone)]
pub enum SqlData {
    Sqlite(sqlite::Value),
    Odbc(String),
}

#[derive(Clone)]
pub enum SqlType {
    Sqlite(sqlite::Type),
    Odbc(odbc::ffi::SqlDataType),
}

#[derive(Default)]
pub enum SqlError {
    Sqlite(sqlite::Error),
    Odbc(DiagnosticRecord),
    #[default]
    None,
}

// What kind of DB are we connecting to?
#[derive(Clone)]
pub enum ConnectionBase {
    Odbc,
    Sqlite,
}

impl RecordSet {
    pub fn construct_sqlite(
        &mut self,
        stmt: &mut sqlite::Statement,
    ) -> std::result::Result<(), SqlError> {
        match stmt.next() {
            Ok(_) => {},
            Err(E) => { return Err(SqlError::Sqlite(E)) },
        };

        for name in stmt.column_names() {
            let res = match stmt.column_type(&String::from(&name[..])[..]) {
                Ok(T) => { T },
                Err(E) => {  return Err(SqlError::Sqlite(E))  },
            };
            self.column_info.insert(
                String::from(&name[..]),
                SqlType::Sqlite(res),
            );
            self.column_order.push(String::from(&name[..]));
        }
        match stmt.reset() {
            Ok(_) => { Ok(()) },
            Err(E) => { return Err(SqlError::Sqlite(E)) },
        }
    } //fill fields 'column_count', 'column_info'

    pub fn construct_odbc(
        &mut self,
        stmt: &mut odbc::Statement<'_, '_, Allocated, HasResult, odbc_safe::AutocommitOn>,
    ) -> std::result::Result<(), SqlError> {
        let num_cols: i16 = match stmt.num_result_cols() {
            Ok(i) => { i as i16 },
            Err(E) => { return Err(SqlError::Odbc(E)) },
        };
        
        for col_index in 1..num_cols {
            let col_description: ColumnDescriptor = match stmt.describe_col(col_index as u16) {
                Ok(T) => { T },
                Err(E) => {  return Err(SqlError::Odbc(E)) },
            };
            self.column_info.insert(
                String::from(col_description.name.clone()),
                SqlType::Odbc(col_description.data_type),
            );
            self.column_order
                .push(String::from(col_description.name.clone()));
        }
        Ok(())
    }

    //pub fn add(&mut self, rec: Record<sqlite::Value>) {
    pub fn add(&mut self, rec: Record) {
        self.records.push(rec);
    } //insert a record into the recordset

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn column_count(&self) -> usize {
        self.records.first().unwrap().columns.len()
    }

    pub fn fetch_page_of_records(&self, page: usize) -> Vec<Record> {
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

    pub fn default() -> Self {
        Self {
            column_info: HashMap::<String, SqlType>::new(),
            column_order: Vec::<String>::new(),
            records: Vec::<Record>::new(),
        }
    }
}

/*
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
} */

impl Record {
    pub fn add(&mut self, key: String, val: SqlData) {
        self.columns.insert(key, Some(val));
    }
    // only being used for odbc currently
    pub fn construct(&mut self, column_info: &HashMap<String, SqlType>) {
        for key in column_info.keys() {
            self.columns.insert(key.clone(), None); // refactor this to actually use the SqlType and point to the real data types?
        }
    }
}

/*
impl Record<String> {
    pub fn construct(&mut self, columns: &HashMap<String, odbc::ffi::SqlDataType>) {
        for key in columns.keys() {
            self.columns.insert(key.clone(), String::new());
        }
    }
} */

impl Connection {
    pub fn assemble_rs(&mut self, donor_rs: RecordSet) {
        self.record_set = Some(donor_rs);
        self.result_code = Some(1 as i32);
    }

    pub fn assemble_err(&mut self, _the_error: SqlError) -> () {
        self.result_code = Some(-1 as i32);
        self.result_details = match self.connection_type.clone() {
            Some(ConnectionBase::Odbc) => Some(String::from("Text")),
            Some(ConnectionBase::Sqlite) => Some(String::from("Text")),
            None => { None }
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

// sqlite implementation
// can I extend this to odbc too?
impl Translate for SqlData {
    fn translate(&self) -> String {
        let mut payload: String = String::new();

        match self {
            SqlData::Sqlite(val) => match val {
                SqliteFloat(value) => payload.push_str(&value.to_string()),
                SqliteInteger(value) => payload.push_str(&value.to_string()),
                SqliteString(value) => payload.push_str(&value.to_string()),
                SqliteBinary(value) => {
                    for element in value {
                        payload.push_str(&element.to_string())
                    }
                }
                SqliteNull => payload.push_str("Null"),
            },
            SqlData::Odbc(_val) => { },
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
