/* --> Imports */

//use crate::sql_aux_funcs::Translate;
use crate::sql_aux_funcs::{Record, RecordSet, SqlData, SqlType, SqlError, Translate};
pub use sqlite::{
    Bindable, BindableWithIndex, Connection, Error, State, Statement, Type,
    Value::{
        Binary as SqliteBinary, Float as SqliteFloat, Integer as SqliteInteger, Null as SqliteNull,
        String as SqliteString,
    },
};
use std::collections::HashMap;
use std::fs;
use std::io;

/* <-- Imports */
/* --> Structs */



/* <-- Structs */
/* --> Enums */

/* <-- Enums */
/* --> Functions */

pub fn raw_query(
    db_name: String,
    query: String,
) -> Result<RecordSet, SqlError> {
    let db_handle = match sqlite::open(&db_name) {
        Ok(T) => { T },
        Err(E) => {  return Err(SqlError::Sqlite(E)) },
    };

    //do I need to trim the query here? Is this always a safe practice?
    // will most sql engines trim query strings by default anyway?
    let result = select_from(&db_handle, query.trim())?;

    Ok(result)
}

pub fn cli_query(db_name: String) -> Result<(), sqlite::Error> {
    //start the db
    let db_handle = sqlite::open(&db_name)?;
    let confirmation = &String::from("y")[..];
    //add rows
    if db_name == ":memory:" {
        build_db(&db_handle)?;
    }
    loop {
        let mut state_trigger: String = String::new();
        let mut save_trigger: String = String::new();

        println!("Enter a query.");
        let mut query: String = String::new();
        io::stdin()
            .read_line(&mut query)
            .expect("Failed to read line.");

        let result = match select_from(&db_handle, query.trim()) {
            Ok(T) => { T },
            Err(E) => { 
                match E {
                    SqlError::Sqlite(Er) => { return Err(Er) } ,
                    _ => { panic!("How did a different kind of error get in here?") },
                };
            },
        };

        let csv_printout: String = print_results(&result);
        println!("\n==============\nDone printing.\nSave results?(Y/N)");

        io::stdin()
            .read_line(&mut save_trigger)
            .expect("Failed to read line.");

        if save_trigger.trim() == confirmation {
            match save_results(csv_printout) {
                Ok(_) => println!("Results saved."),
                Error => println!("Failed to save results.\n{:?}", Error),
            }
        }
        println!("Continue to query?(y/n)");
        io::stdin()
            .read_line(&mut state_trigger)
            .expect("Failed to read line.");
        if state_trigger.trim() != confirmation {
            break;
        }
    }
    Ok(())
}

fn build_db(db_handle: &sqlite::Connection) -> Result<(), sqlite::Error> {
    let query: &str = "BEGIN TRANSACTION;
		CREATE TABLE users (name TEXT, age INTEGER, location_id INTEGER, gender_id INTEGER);
		INSERT INTO users VALUES ('Alice', 42, 1, 1);
		INSERT INTO users VALUES ('Bob', 69, 2, 2);
		INSERT INTO users VALUES ('Mark', 50, 3, 2);
		INSERT INTO users VALUES ('Alex', 5, 4, 1);
		CREATE TABLE fo_location (location_id INTEGER, location TEXT);
		INSERT INTO fo_location VALUES (1, 'Denmark');
		INSERT INTO fo_location VALUES (2, 'Brazil');
		INSERT INTO fo_location VALUES (3, 'Russia');
		INSERT INTO fo_location VALUES (4, 'Canada');
		COMMIT;";

    db_handle.execute(query)?;

    Ok(())
}

fn select_from(
    db_handle: &sqlite::Connection,
    query: &str,
) -> Result<RecordSet, SqlError> {
    let mut stmt = match db_handle.prepare(query) {
        Ok(T) => { T },
        Err(E) => { return Err(SqlError::Sqlite(E)) },
    };
    //bind parameters function call here

    //construct recordset
    let mut record_set: RecordSet = RecordSet {
        column_info: HashMap::new(),
        column_order: Vec::new(),
        records: Vec::new(),
        error_interface: SqlError::None,
    };

    record_set.construct_sqlite(&mut stmt)?;

    //then read recordsets from Sqlite
    while let Ok(State::Row) = stmt.next() {
        //new row available
        //create a new record object
        let mut current_row: Record = Record {
            columns: HashMap::new(),
            data_type: crate::sql_aux_funcs::ConnectionBase::Sqlite,
        };


        /* match db_handle.prepare(query) {
        Ok(T) => { T },
        Err(E) => { return Err(SqlError::Sqlite(E)) },
    }; */
        //parse the columns in the row
        for (name, _) in &record_set.column_info {
            // 'name' will index the row and fetch columns
            let read_value = match stmt.read::<Option<sqlite::Value>, _>(&name[..]) {
                Ok(T) => { T },
                Err(E) => { return Err(SqlError::Sqlite(E)) },
            };
            match read_value {
                //fetched data from a column
                Some(value) => {
                    //'value' is the data in the column

                    //add value to the Record object
                    current_row.add(name.clone(), SqlData::Sqlite(value.clone()));
                }

                //no data found in the column? not even a SqliteNull?
                None => (),
            }
        }
        record_set.add(current_row);
    }

    Ok(record_set)
}

pub fn print_results(record_set: &RecordSet) -> String {
    println!("Printing records\n==============\n");
    let mut text_payload: String = String::new();
    let mut current_line: String = String::new();

    //first printout is columns
    for column_name in &record_set.column_order {
        current_line.push_str(&column_name.to_string()[..]);
        if &column_name != &record_set.column_order.last().unwrap() {
            current_line.push(',');
            current_line.push(' ');
        } else {
            current_line.push('\n')
        }
    }
    text_payload.push_str(&current_line.to_string()[..]);

    //followed by a printout of data
    for record in &record_set.records {
        current_line.clear();
        for v in &record_set.column_order {
            match record.columns.get(v) {
                Some(value) => current_line.push_str(&value.translate()),
                None => {}
            }
            if &v != &record_set.column_order.last().unwrap() {
                current_line.push(',');
                current_line.push(' ');
            } else {
                current_line.push('\n')
            }
        }
        text_payload.push_str(&current_line.to_string()[..]);
    }

    println!("{text_payload}");
    text_payload
}

pub fn save_results(csv: String) -> io::Result<()> {
    fs::write("results.csv", &csv)?;
    Ok(())
}

fn bind_parameters<T>(stmt: &mut Statement, bindings: Vec<T>) -> Result<(), sqlite::Error>
where
    T: Bindable + Copy,
{
    /*
        binding parameters required a vector of tuples
        where each tuple (position, parameter) binds
        an individual parameter. This function needs
        to be fleshed out to handled generic types?
        only tested so far with integers
    */

    //let bind_from = [(1, 1)];;

    for value in bindings.iter() {
        stmt.bind(*value)?;
    }

    Ok(())
}

/* <-- Functions */
