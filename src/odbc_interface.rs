/* --> Imports */

use crate::sql_aux_funcs::{Record, RecordSet, SqlData, SqlType, Translate,
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
//  omit, implement a temporary call to fetch column titles until get_tables is fully implemented
//            let new_stmt = get_tables(stmt, c)?;
//            ResultSetState::from(Data(new_stmt)) //the new Statement is converted to a ResultSetState<Statement> in order to match the return type that 'result' is defined as, on the line above. This allows a seamless transition to the 'match' statement below, on the 'result' variable
            match c {
                Request::Columns(c) => stmt.exec_direct(&c)?,
                _ => { return Err(DiagnosticRecord::empty()) }
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
    request: Request,
) -> Result<Statement<'a, 'b, odbc::Executed, odbc::HasResult, AutocommitOn>, DiagnosticRecord> {
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

    stmt.tables(&c, &s, &tn, &tt)
}

/* <-- Functions */
