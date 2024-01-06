/* --> Imports */

pub use sqlite::{Connection, State, Statement, 
	Value::{Binary as SqliteBinary, 
			Float as SqliteFloat, 
			Integer as SqliteInteger, 
			Null as SqliteNull, 
			String as SqliteString},
	Type,
	BindableWithIndex,
	Bindable};
use std::collections::HashMap;
use std::io;
use std::fs;
use crate::sql_aux_funcs::SqliteTranslation;
pub use crate::sql_aux_funcs::{RecordSet, Record};

/* <-- Imports */
/* --> Enums */



/* <-- Enums */
/* --> Functions */

pub fn raw_query(db_name: String, query: String) -> Result<RecordSet<sqlite::Value, sqlite::Type>, sqlite::Error> {
	let db_handle = sqlite::open(&db_name)?;

	//do I need to trim the query here? Is this always a safe practice?
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

		let result = select_from(&db_handle, query.trim())?;
		
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
			break
		}

//		println!("Continuing...", state_trigger.trim(), confirmation);
		
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
	query: &str) -> 
	Result<
		RecordSet<sqlite::Value, sqlite::Type>,
		sqlite::Error> {
	let mut stmt = db_handle.prepare(query)?;
	//bind parameters function call here
	
	//construct recordset meta data
	let mut record_set: RecordSet<sqlite::Value, sqlite::Type> = RecordSet { 
		column_info: HashMap::new(),
		column_order: Vec::new(),
		records: Vec::new(),
		paged_records: Vec::new(),
	};
	record_set.construct(&mut stmt)?;
	
	//then read recordsets from Sqlite
	while let Ok(State::Row) = stmt.next() {
		//new row available
		//create a new record object
		let mut current_row: Record<sqlite::Value> = Record {  columns: HashMap::new(), };
		
		//parse the columns in the row
		for (name, _) in &record_set.column_info {
			
			// 'name' will index the row and fetch columns
			match stmt.read::<Option<sqlite::Value>, _>(&name[..])? {
				
				//fetched data from a column
				Some(value) => { 
					//'value' is the data in the column
					//println!("{:?}", value);
					
					//add value to the Record object
					current_row.add(name.clone(), value.clone());
				
				},
				
				//no data found in the column? not even a SqliteNull?
				None => (),
			}
		}
		record_set.add(current_row);
	}

  	Ok(record_set)
}

pub fn print_results(record_set: &RecordSet<sqlite::Value, sqlite::Type>) -> String {
	println!("Printing records\n==============\n");
	let mut text_payload: String = String::new();
	let mut current_line: String = String::new();

	//first printout is columns	
	for column_name in &record_set.column_order {  
		current_line.push_str(&column_name.to_string()[..]);  
		if &column_name != &record_set.column_order.last().unwrap() {  
			current_line.push(',');  
			current_line.push(' ');  
		}  
		else { current_line.push('\n') }  
	}  
	text_payload.push_str(&current_line.to_string()[..]);  
  
	//followed by a printout of data  
	for record in &record_set.records {  
		current_line.clear();  
		for v in &record_set.column_order {  
			match record.columns.get(v) {
				Some(value) => {
					current_line.push_str(&value.translate())
				},
				None => { },
			}
			if &v != &record_set.column_order.last().unwrap() {
				current_line.push(',');
				current_line.push(' ');
			} else { current_line.push('\n') }
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

fn bind_parameters<T>(stmt: &mut Statement, bindings: Vec<T>) 
-> Result<(), sqlite::Error>
where
	T: Bindable + Copy {
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
