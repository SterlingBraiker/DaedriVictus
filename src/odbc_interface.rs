/* --> Imports */

use crate::sql_aux_funcs::{Record, RecordSet, SqlData, SqlType, Translate,
    QueryType, Request};
pub use odbc::{
    create_environment_v3, odbc_safe::AutocommitOn, Connection, Data, DiagnosticRecord, Executed,
    NoData, ResultSetState, Statement, Version3, Handle,
};
use odbc::{odbc_safe::ResultSet, ColumnDescriptor};
use std::{collections::HashMap, io, ptr::null_mut, ptr::*};

/* <-- Imports */
/* --> Structs */

/* <-- Structs */
/* --> Enums   */

/* <-- Enums   */
/* --> Functions */

pub fn entry_point(
    dsn: String,
    request: QueryType,
) -> Result<RecordSet, DiagnosticRecord> {
    connect(dsn, request)
}

pub fn connect(
    dsn: String,
    request: QueryType,
) -> Result<RecordSet, DiagnosticRecord> {

    let environment: odbc::Environment<odbc::odbc_safe::Odbc3> = create_environment_v3().map_err(|e| e.unwrap())?;
        //"Driver=Microsoft Access Text Driver (*.txt, *.csv);Dbq=D:\\;Extensions=asc,csv,tab,txt;",
    let conn = environment.connect_with_connection_string(&dsn)?;

    execute_statement(&conn, request)
}

fn execute_statement<'env>(
    conn: &Connection<'env, AutocommitOn>,
    request: QueryType,
) -> Result<RecordSet, DiagnosticRecord> {
    let stmt = Statement::with_parent(conn)?;

    let results: ResultSetState<'_, '_, _, AutocommitOn> = match request {
        QueryType::SqlFunction(c) => {
            match c {
                Request::Columns(c) => { get_columns(conn, String::from("cd_employees"), String::from("%"), String::from(""), String::from(""))? },
                Request::Tables(t) => { get_tables(stmt, t)? },
                Request::Schema(sch) => { return Err(DiagnosticRecord::empty()) },
            }
        },
        QueryType::UserDefined(s) => { 
            stmt.exec_direct(&s)?
        },
    };

    let mut recordset: RecordSet = RecordSet {
        column_info: HashMap::<String, SqlType>::new(),
        column_order: Vec::new(),
        records: Vec::<Record>::new(),
    };

    match results {
        Data(mut stmt) => {
            
            recordset.construct_odbc(&mut stmt)?;
            let cols = stmt.num_result_cols()?;

            while let Some(mut cursor) = stmt.fetch()? {
                //.fetch() grabs another row of data. create a record here
                let mut rec: Record = Record {
                    columns: HashMap::new(),
                    data_type: Some(crate::sql_aux_funcs::ConnectionBase::Odbc),
                };
                rec.construct(&recordset.column_info);

                for i in 1..=cols {
                    let result = cursor.get_data::<&str>(i as u16)?;

                    match result {
                        Some(val) => rec.add(
                            recordset
                                .column_order
                                .get((i - 1) as usize)
                                .unwrap()
                                .clone(),
                            SqlData::Odbc(String::from(val)),
                        ),
                        None => rec.add(
                            recordset
                                .column_order
                                .get((i - 1) as usize)
                                .unwrap()
                                .clone(),
                            SqlData::Odbc(String::from("Null")),
                        ),
                    };
                }
                recordset.add(rec);
            }
        }
        NoData(_) => { // force a null recordset here
             /*
             let mut a: Vec<String> = Vec::new();
             let b: String = String::from("null");
             a.push(b);
             recordset.add(b);
              */
        }
    }
    recordset.keep(String::from("TABLE_NAME"));
    Ok(recordset)
}

fn get_tables<'a, 'b>(
    stmt: Statement<'a, 'b, odbc::Allocated, odbc::NoResult, AutocommitOn>,
    _table_index: u8,
) -> Result<ResultSetState<'a, 'b, odbc::Executed, AutocommitOn>, DiagnosticRecord> {
    // define the 4 parameters to be passed into stmt.tables()
    // stmt.tables() can return information about schemas, tables, or columns
    // see msdn for more information
    let (c, s, tn, tt) =
    (
        String::from(""),
        String::from("dba"),
        String::from(""),
        String::from(""),
    );

    let new_stmt: Statement<'a, 'b, _, odbc::HasResult, AutocommitOn> = stmt.tables(&c, &s, &tn, &tt)?;
    Ok(ResultSetState::from(Data(new_stmt))) //the new Statement is converted to a ResultSetState<Statement> in order to match the return type that 'result' is defined as, on the line above. This allows a seamless transition to the 'match' statement below, on the 'result' variable
}


fn get_columns(
    conn: &Connection<'env, AutocommitOn>,
    mut table_name: String, 
    mut column_name: String, 
    mut catalog_name: String, 
    mut schema_name: String,
) -> Result<ResultSetState<'a, 'b, odbc::Executed, AutocommitOn>, DiagnosticRecord> {
    let table_length: odbc::ffi::SQLSMALLINT = table_name.len() as i16;
    let table_encoded: Vec<u16> = String::from(table_name).encode_utf16().collect();
    let table_ptr: *const u16 = std::ptr::addr_of!(table_encoded);

    let column_length: odbc::ffi::SQLSMALLINT = column_name.len() as i16;
    let column_location: Vec<u16> = column_name.encode_utf16().collect();

    let catalog_length: odbc::ffi::SQLSMALLINT = catalog_name.len() as i16;
    let catalog_location: Vec<u16> = catalog_name.encode_utf16().collect();

    let schema_length: odbc::ffi::SQLSMALLINT = schema_name.len() as i16;
    let schema_location: Vec<u16> = schema_name.encode_utf16().collect();

    let mut stmt = match Statement::with_parent(conn) {
        Ok(s) => s,
        Err(_) => return Err(DiagnosticRecord::empty()) ,
    };
    
    unsafe {
        odbc::ffi::SQLColumnsW(
        stmt.handle(), 
        *catalog_location,
        catalog_length,
        *schema_location,
        schema_length,
        *table_location,
        table_length,
        *column_location,
        column_length
        );
    }

    Ok(ResultSetState::from(Data(stmt)));
}
/* <-- Functions */
