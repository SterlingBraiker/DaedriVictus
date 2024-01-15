use clipboard::{ClipboardContext, ClipboardProvider};
pub use odbc::{
    create_environment_v3, odbc_safe::AutocommitOn, Connection, Data, DiagnosticRecord, NoData,
    OdbcType, Statement,
};
use std::io;

pub fn entry_point() {
    let mut recordset: Vec<Vec<String>> = Vec::new();

    match connect(
        &mut recordset,
        String::from("DSN=odbc2;"),
        String::from("select * from results.csv;"),
    ) {
        Ok(()) => println!("Success"),
        Err(diag) => {
            println!("Error: {}", diag);
            return;
        }
    }

    let mut payload: String = String::new();

    for ROW in recordset {
        for CELL in &ROW {
            payload.push_str(&CELL[..]);
            if CELL != &ROW[ROW.len() - 1] {
                payload.push_str(", ");
            }
        }
        payload.push_str("\n");
    }

    let mut clipctx = ClipboardContext::new().unwrap();

    match clipctx.set_contents(payload) {
        Ok(_) => println!("Success"),
        Err(_) => println!("Fail"),
    }
}

pub fn connect(
    recordset: &mut Vec<Vec<String>>,
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
    recordset: &mut Vec<Vec<String>>,
    sql_text: String,
) -> odbc::Result<()> {
    let stmt = Statement::with_parent(conn)?;

    match stmt.exec_direct(&sql_text)? {
        Data(mut stmt) => {
            let cols = stmt.num_result_cols()?;
            while let Some(mut cursor) = stmt.fetch()? {
                let mut consumption: Vec<String> = Vec::with_capacity(cols as usize);
                for i in 1..(cols + 1) {
                    match cursor.get_data::<&str>(i as u16)? {
                        Some(val) => consumption.push(val.to_owned()),
                        None => consumption.push(String::from("NULL")),
                    }
                }
                recordset.push(consumption);
                println!("len: {}", recordset.len());
                println!("consumptionlen: {}", recordset[0].len());
                println!("{}", recordset[recordset.len() - 1][0]);
            }
        }
        NoData(_) => {
            let mut a: Vec<String> = Vec::new();
            let b: String = String::from("null");
            a.push(b);
            recordset.push(a);
        }
    }
    //Ok(recordset)
    Ok(())
}
