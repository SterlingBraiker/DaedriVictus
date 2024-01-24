use crate::sql_aux_funcs::Translate;
use crate::sql_aux_funcs::{Record, RecordSet};
use odbc::ColumnDescriptor;
pub use odbc::{
    create_environment_v3, odbc_safe::AutocommitOn, Connection, Data, DiagnosticRecord, NoData,
    Statement, Version3,
};
use std::collections::HashMap;
use std::io;

/* <-- Imports */
/* --> Structs */

pub struct OdbcDiagnosticRecord {
    wrapped_error: Option<DiagnosticRecord>,
}

impl Default for OdbcDiagnosticRecord {
    fn default() -> Self { 
        OdbcDiagnosticRecord {
            wrapped_error: None,
        }
    }
}

/* <-- Structs */
/* --> Functions */

pub fn entry_point(
    dsn: String,
    sql_text: String,
) -> std::result::Result<RecordSet<String, odbc::ffi::SqlDataType, DiagnosticRecord>, DiagnosticRecord> {
    let recordset = RecordSet {
        column_info: HashMap::new(),
        column_order: Vec::new(),
        records: Vec::new(),
        error_interface: OdbcDiagnosticRecord::default(),
    };

    connect(dsn, sql_text, recordset)
}

pub fn connect(
    dsn: String,
    sql_text: String,
    recordset: RecordSet<String, odbc::ffi::SqlDataType, DiagnosticRecord>,
) -> std::result::Result<RecordSet<String, odbc::ffi::SqlDataType, DiagnosticRecord>, DiagnosticRecord> {
    let environment: odbc::Environment<odbc::odbc_safe::Odbc3> =
        create_environment_v3().map_err(|e| e.unwrap())?;

    let conn = environment.connect_with_connection_string(
        //"Driver=Microsoft Access Text Driver (*.txt, *.csv);Dbq=D:\\;Extensions=asc,csv,tab,txt;",
        &dsn,
    )?;

    execute_statement(&conn, recordset, sql_text)
}

fn execute_statement<'env>(
    conn: &Connection<'env, AutocommitOn>,
    mut recordset: RecordSet<String, odbc::ffi::SqlDataType, DiagnosticRecord>,
    sql_text: String
) -> odbc::Result<RecordSet<String, odbc::ffi::SqlDataType, DiagnosticRecord>> {
    //add a lifetime to the recordset which originates from outside of entry_point()
    let stmt = Statement::with_parent(conn)?;

    match stmt.exec_direct(&sql_text)? {
        Data(mut stmt) => {
            recordset.construct(&mut stmt)?;
            let cols = stmt.num_result_cols()?;
            while let Some(mut cursor) = stmt.fetch()? {
                //.fetch() grabs another row of data. create a record here
                //let mut consumption: Vec<String> = Vec::with_capacity(cols as usize);

                let mut rec: Record<String> = Record {
                    columns: HashMap::new(),
                };
                rec.construct(&recordset.column_info);

                for i in 1..(cols + 1) {
                    match cursor.get_data::<&str>(i as u16)? {
                        Some(val) => rec.add(
                            recordset.column_order.get(i as usize).unwrap().clone(),
                            String::from(val),
                        ),
                        None => rec.add(
                            recordset.column_order.get(i as usize).unwrap().clone(),
                            String::from("Null"),
                        ),
                    }
                }
                recordset.add(rec);
            }
        }
        NoData(_) => { /*
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