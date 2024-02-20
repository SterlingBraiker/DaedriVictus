/* --> Imports */

use crate::sql_aux_funcs::{Record, RecordSet, SqlData, SqlError, SqlType, Translate,
    QueryType, Request};
pub use odbc::{
    create_environment_v3, odbc_safe::AutocommitOn, Connection, Data, DiagnosticRecord, Executed,
    NoData, ResultSetState, Statement, Version3,
};
use odbc::{odbc_safe::ResultSet, ColumnDescriptor};
use std::{collections::HashMap, io, ptr::null_mut};

/* <-- Imports */
/* --> Structs */

/* <-- Structs */
/* --> Enums   */

/* <-- Enums   */
/* --> Functions */

pub fn entry_point(
    dsn: String,
    request: QueryType,
) -> Result<RecordSet, SqlError> {
    connect(dsn, request)
}

pub fn connect(
    dsn: String,
    request: QueryType,
) -> Result<RecordSet, SqlError> {
    let environment: odbc::Environment<odbc::odbc_safe::Odbc3> =
        match create_environment_v3().map_err(|e| e.unwrap()) {
            Ok(T) => T,
            Err(E) => return Err(SqlError::Odbc(E)),
        };

    let conn = match environment.connect_with_connection_string(
        //"Driver=Microsoft Access Text Driver (*.txt, *.csv);Dbq=D:\\;Extensions=asc,csv,tab,txt;",
        &dsn,
    ) {
        Ok(T) => T,
        Err(E) => return Err(SqlError::Odbc(E)),
    };

    let result = match execute_statement(&conn, request) {
        Ok(T) => Ok(T),
        Err(E) => Err(E),
    };

    result
}

fn execute_statement<'env>(
    conn: &Connection<'env, AutocommitOn>,
    request: QueryType,
) -> Result<RecordSet, SqlError> {
    let stmt = match Statement::with_parent(conn) {
        Ok(T) => T,
        Err(E) => return Err(SqlError::Odbc(E)),
    };

    let results: ResultSetState<'_, '_, _, AutocommitOn> = match request {
        QueryType::SqlFunction(c) => {
            let new_stmt = match get_tables(stmt, c) {
            //this is a call to ODBC SQLFunction SQLTables. The original Statement is passed in, consumed, and a new Statement returned (whose types change from <NoResult> to <Result>)
                Ok(T) => T,
                Err(E) => return Err(E),
            };
            ResultSetState::from(Data(new_stmt)) //the new Statement is converted to a ResultSetState<Statement> in order to match the return type that 'result' is defined as, on the line above. This allows a seamless transition to the 'match' statement below, on the 'result' variable
        
        },
        QueryType::UserDefined(s) => { 
            match stmt.exec_direct(&s) {
                Ok(T) => T,
                Err(E) => return Err(SqlError::Odbc(E)),
            }
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
            let cols = match stmt.num_result_cols() {
                Ok(T) => T,
                Err(E) => return Err(SqlError::Odbc(E)),
            };

            while let Some(mut cursor) = match stmt.fetch() {
                Ok(T) => T,
                Err(_) => None,
            } {
                //.fetch() grabs another row of data. create a record here
                let mut rec: Record = Record {
                    columns: HashMap::new(),
                    data_type: Some(crate::sql_aux_funcs::ConnectionBase::Odbc),
                };
                rec.construct(&recordset.column_info);

                for i in 1..=cols {
                    let result = match cursor.get_data::<&str>(i as u16) {
                        Ok(T) => T,
                        Err(E) => return Err(SqlError::Odbc(E)),
                    };

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
    Ok(recordset)
}

fn get_tables<'a, 'b>(
    stmt: Statement<'a, 'b, odbc::Allocated, odbc::NoResult, AutocommitOn>,
    request: Request,
) -> Result<Statement<'a, 'b, odbc::Allocated, odbc::HasResult, AutocommitOn>, SqlError> {
    // define the 4 parameters to be passed into stmt.tables()
    // stmt.tables() can return information about schemas, tables, or columns
    // see msdn for more information
    let (c, s, tn, tt) = match request {
        Request::Schema(index) => {
            (
                String::from(""),
                String::from(""),
                String::from(""),
                String::from(""),
            )
        },
        Request::Tables(_index) => {
            (
                String::from(""),
                String::from("dba"),
                String::from(""),
                String::from(""),
            )
        },
        Request::Columns(table_name) => {
            (
                String::from(""),
                String::from("dba"),
                String::from(table_name),
                String::from(""),
            )
        },
    };

    match stmt.tables(&c, &s, &tn, &tt) {
        Ok(T) => Ok(T),
        Err(E) => Err(SqlError::Odbc(E)),
    }
}

/* <-- Functions */
