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
    let stmt: Statement<'_, '_, odbc::Allocated, odbc::NoResult, AutocommitOn> = Statement::with_parent(conn)?;

    let recordset: RecordSet = match request.clone() {
        QueryType::SqlFunction(c) => {
            match c {
                Request::Columns(c) => { 
                    let rss = get_columns(String::from(c), stmt)?;
                    let rs = resultset_to_recordset(rss)?;
                    sqlcolumns(rs)
                },
                Request::Tables(t) => { 
                    let rss = get_tables(stmt, t)?;
                    let rs = resultset_to_recordset(rss)?;
                    sqltables(rs)
                },
                Request::Schema(sch) => { return Err(DiagnosticRecord::empty()) },
            }
        },
        QueryType::UserDefined(s) => { 
            resultset_to_recordset(stmt.exec_direct(&s)?)?
        },
    };

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


fn get_columns<'a, 'b>(
    table_name: String, 
    stmt: Statement<'a, 'a, odbc::Allocated, odbc::NoResult, AutocommitOn>,
) -> Result<ResultSetState<'a, 'b, odbc::Executed, AutocommitOn>, DiagnosticRecord> where 'a: 'b{ 
    println!("table name in get_columns: {}", table_name);
    let result: ResultSetState<'a, 'b, odbc::Allocated, AutocommitOn> = stmt.exec_direct(&format!("select top 1 * from {}", table_name))?;

    Ok(result)
}

fn resultset_to_recordset(
    rss: ResultSetState<'_, '_, odbc::Allocated, AutocommitOn>
) -> Result<RecordSet, DiagnosticRecord> {
    let mut recordset: RecordSet = RecordSet {
        column_info: HashMap::<String, SqlType>::new(),
        column_order: Vec::new(),
        records: Vec::<Record>::new(),
    };

    match rss {
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
        },
    };
    Ok(recordset)
}

fn sqlcolumns(mut rs: RecordSet) -> RecordSet {
    let mut new_rs: RecordSet = RecordSet {
        column_info: HashMap::new(),
        column_order: Vec::from([String::from("COLUMN_NAME")]),
        records: Vec::new(),
    };

    if rs.records.len() > 0 {
        let mut new_records: Vec<Record> = Vec::new();
        let record: Record = rs.records.pop().unwrap();
        let list_of_column_names: Vec<String> = record.columns.into_keys().collect();
        for x in list_of_column_names {
            new_records.push(Record {
                columns: HashMap::from([
                    (String::from("COLUMN_NAME"), Some(SqlData::Odbc(x))),
                    ]),
                data_type: None,
        })};
        new_rs.records = new_records;
    };
    new_rs
}

fn sqltables(mut rs: RecordSet) -> RecordSet {
    rs.keep(String::from("TABLE_NAME"));
    rs
}
/* <-- Functions */
