/* --> Imports */

use crate::sql_aux_funcs::{Record, RecordSet, SqlData, SqlError, SqlType, Translate};
use odbc::ColumnDescriptor;
pub use odbc::{
    create_environment_v3, odbc_safe::AutocommitOn, Connection, Data, DiagnosticRecord, NoData,
    Statement, Version3,
};
use std::collections::HashMap;
use std::io;

/* <-- Imports */
/* --> Structs */

/* <-- Structs */
/* --> Functions */

pub fn entry_point(
    dsn: String,
    sql_text: String,
) -> Result<RecordSet, SqlError> {
    let recordset: RecordSet = RecordSet {
        column_info: HashMap::<String, SqlType>::new(),
        column_order: Vec::new(),
        records: Vec::<Record>::new(),
    };

    connect(dsn, sql_text, recordset)
}

pub fn connect(
    dsn: String,
    sql_text: String,
    recordset: RecordSet,
) -> Result<RecordSet, SqlError> {
    let environment: odbc::Environment<odbc::odbc_safe::Odbc3> =
        match create_environment_v3().map_err(|e| e.unwrap()) {
            Ok(T) => { T },
            Err(E) => { return Err(SqlError::Odbc(E)) },
    };

    let conn = match environment.connect_with_connection_string(
        //"Driver=Microsoft Access Text Driver (*.txt, *.csv);Dbq=D:\\;Extensions=asc,csv,tab,txt;",
        &dsn,
    ) {
        Ok(T) => { T },
        Err(E) => { return Err(SqlError::Odbc(E)) },
    };

    let result = match execute_statement(&conn, recordset, sql_text) {
        Ok(T) => { Ok(T) },
        Err(E) => { Err(E) },
    };

    result
}

fn execute_statement<'env>(
    conn: &Connection<'env, AutocommitOn>,
    mut recordset: RecordSet,
    sql_text: String,
) -> Result<RecordSet, SqlError> {
    let stmt = match Statement::with_parent(conn) {
        Ok(T) => { T },
        Err(E) => { return Err(SqlError::Odbc(E)) },
    };

    let results = match stmt.exec_direct(&sql_text) {
        Ok(T) => { T },
        Err(E) => { return Err(SqlError::Odbc(E)) },
    };

    match results {
        Data(mut stmt) => {
            recordset.construct_odbc(&mut stmt)?;
            let cols = match stmt.num_result_cols() {
                Ok(T) => { T },
                Err(E) => { return Err(SqlError::Odbc(E)) },
            };

            while let Some(mut cursor) = match stmt.fetch() {
                Ok(T) => { T },
                Err(_) => { None },
            } {

                //.fetch() grabs another row of data. create a record here
                let mut rec: Record = Record {
                    columns: HashMap::new(),
                    data_type: Some(crate::sql_aux_funcs::ConnectionBase::Odbc),
                };
                rec.construct(&recordset.column_info);

                for i in 1..=cols {
                    let result = match cursor.get_data::<&str>(i as u16) {
                        Ok(T) => { T },
                        Err(E) => { return Err(SqlError::Odbc(E)) },
                    };
                    
                    match result {
                        Some(val) => rec.add(
                            recordset.column_order.get((i - 1) as usize).unwrap().clone(),
                            SqlData::Odbc(String::from(val)),
                        ),
                        None => rec.add(
                            recordset.column_order.get((i - 1) as usize).unwrap().clone(),
                            SqlData::Odbc(String::from("Null")),
                        ),
                    };
                }
                recordset.add(rec);
            }
        },
        NoData(_) => { // force a null recordset here
             /*
             let mut a: Vec<String> = Vec::new();
             let b: String = String::from("null");
             a.push(b);
             recordset.add(b);
              */
        }

    }
    Ok(recordset)
}
 
/* <-- Functions */
