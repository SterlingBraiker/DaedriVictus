/*

	SQL library
		Sqlite3
		ODBC

*/

use std::collections::HashMap;
use crate::sqlite3_interface::*;

/* --> Structs */
/*
#[derive(Clone)]
pub struct RecordSet<T, U> {
	pub column_info: HashMap<String, U>,
	pub column_order: Vec<String>,
	pub records: Vec<Record<T>>,
	pub paged_records: Vec<Record<T>>, //consider changing this to a std::slice for performance?
}
*/

#[derive(Clone)]
pub struct RecordSet<'a, T, U> {
	pub column_info: HashMap<String, U>,
	pub column_order: Vec<String>,
	pub records: Vec<Record<T>>,
	pub paged_records: Option<&'a [Record<T>]>,
}


#[derive(Clone, Default)]
pub struct Record<T> {
	pub columns: HashMap<String, T>,
}

impl<'a> RecordSet<'a, sqlite::Value, sqlite::Type> {
	pub fn construct(&mut self, stmt: &mut sqlite::Statement) -> Result<(), sqlite::Error> { 
		stmt.next()?;

		for name in stmt.column_names() {
			self.column_info.insert(
				String::from(&name[..]), 
				stmt.column_type(&String::from(&name[..])[..])?
			);
			self.column_order.push(String::from(&name[..]));
		}
		stmt.reset()
	} //fill fields 'column_count', 'column_info'
	
	pub fn add(&mut self, rec: Record<sqlite::Value>) { 
		self.records.push(rec);
	} //insert a record into the recordset

	pub fn record_count(&self) -> usize {
		self.records.len()
	}
	
	pub fn column_count(&self) -> usize {
		self.records.first().unwrap().columns.len()
	}
	
	pub fn fetch_paged_records(
	&'a self,
	page: usize) -> Option<&[Record<sqlite::Value>]> {
		let range_upper: usize = if (page * 50) < self.records.len() { page *  50 } else { self.records.len() };
		let range_lower: usize = if range_upper > 50 { range_upper - 50 } else { 0 };
		//self.paged_records = self.records[range_lower..range_upper].to_vec();
		Some(&self.records[range_lower..range_upper])
	}
}

impl Default for RecordSet<'_, sqlite::Value, sqlite::Type> {
	fn default() -> Self {
		Self { 
			column_info: HashMap::<String, sqlite::Type>::new(),
			column_order: Vec::<String>::new(),
			records: Vec::<Record<sqlite::Value>>::new(),
			paged_records: None,
		}
	}
}

impl Record<sqlite::Value> {
	pub fn add(&mut self, key: String, val: sqlite::Value) {
		self.columns.insert(key, val);
	}
}


/* <-- Structs */
/* --> Traits */

pub trait SqliteTranslation {
	fn translate(&self) -> String;
}

impl SqliteTranslation for sqlite::Value {
	fn translate(&self) -> String {
		let mut payload: String = String::new();
		
		match self {
			SqliteFloat(value)		=> payload.push_str(&value.to_string()),
			SqliteInteger(value)		=> payload.push_str(&value.to_string()),
			SqliteString(value)	=> payload.push_str(&value.to_string()),
			SqliteBinary(value) 	=> { 
				for element in value {
					payload.push_str(&element.to_string())
				} 
			},
			SqliteNull	=> payload.push_str("Null"),
		}

		payload
	}
}


/* <-- Traits */