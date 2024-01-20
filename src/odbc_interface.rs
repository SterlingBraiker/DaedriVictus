use crate::sql_aux_funcs::Translate;
use crate::sql_aux_funcs::{Record, RecordSet};
use odbc::ColumnDescriptor;
pub use odbc::{
    create_environment_v3, odbc_safe::AutocommitOn, Connection, Data, DiagnosticRecord, NoData,
    OdbcType, Statement,
};
use std::io;
use std::collections::HashMap;

pub fn entry_point() -> RecordSet<String, odbc::ffi::SqlDataType> {
    let mut recordset: RecordSet<String, odbc::ffi::SqlDataType> = RecordSet {
        column_info: HashMap::new(),
        column_order: Vec::new(),
        records: Vec::new(),
    };
    
    match connect(
        &mut recordset,
        String::from("DSN=odbc2;"),
        String::from("select * from results.csv;"),
    ) {
        Ok(()) => println!("Success"),
        Err(diag) => { println!("Error: {}", diag); }
    }
/*
    for ROW in recordset {
        for CELL in &ROW {
            payload.push_str(&CELL[..]);
            if CELL != &ROW[ROW.len() - 1] {
                payload.push_str(", ");
            }
        }
        payload.push_str("\n");
    } 
*/
    recordset
}

pub fn connect(
    recordset: &mut RecordSet<String, odbc::ffi::SqlDataType>,
    _dsn: String,
    sql_text: String,
) -> std::result::Result<(), DiagnosticRecord> {
    let env = create_environment_v3().map_err(|e| e.unwrap())?;
    //let conn = env.connect_with_connection_string(&dsn)?; //force the connection now to a single file, allow for user choice later

    //migrate to a DSN instead of conn string?
    let conn = env.connect_with_connection_string(
        "Driver=Microsoft Access Text Driver (*.txt, *.csv);Dbq=D:\\;Extensions=asc,csv,tab,txt;",
    )?;

    execute_statement(&conn, recordset, sql_text)
}

fn execute_statement<'env>(
    conn: &Connection<'env, AutocommitOn>,
    recordset: &mut RecordSet<String, odbc::ffi::SqlDataType>,
    sql_text: String,
) -> odbc::Result<()> {
    let stmt = Statement::with_parent(conn)?;

    match stmt.exec_direct(&sql_text)? {
        Data(mut stmt) => {
            recordset.construct(&mut stmt);
            let cols = stmt.num_result_cols()?;
            while let Some(mut cursor) = stmt.fetch()? {
                //.fetch() grabs another row of data. create a record here
                //let mut consumption: Vec<String> = Vec::with_capacity(cols as usize);

                let mut rec: Record<String> = Record { columns: HashMap::new(), };
                rec.construct(&recordset.column_info);

                for i in 1..(cols + 1) {

//move this *.describe_col to the *.construct method 
                    match cursor.get_data::<&str>(i as u16)? {
                        Some(val) => rec.add(recordset.column_order.get(i as usize).unwrap().clone(), String::from(val)),
                        None => { rec.add(recordset.column_order.get(i as usize).unwrap().clone(), String::from("Null")) },
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
    //Ok(recordset)
    Ok(())
}
