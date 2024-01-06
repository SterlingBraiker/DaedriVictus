/*

	SQL library
		Sqlite3
		ODBC

*/

use std::collections::HashMap;
use crate::sqlite3_interface::*;

/* --> Structs */

#[derive(Clone, Default)]
pub struct RecordSet<T> {
	pub column_info: HashMap<String, T>,
	pub column_order: Vec<String>,
	pub records: Vec<Record<T>>,
	pub paged_records: Vec<Record<T>>,
}

#[derive(Clone, Default)]
pub struct Record<T> {
	pub columns: HashMap<String, T>,
}

impl RecordSet {
	fn construct(&mut self, stmt: &mut sqlite::Statement) -> Result<(), sqlite::Error> { 
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
	
	fn add(&mut self, rec: Record) { 
		self.records.push(rec);
	} //insert a record into the recordset

	fn record_count(&self) -> usize {
		self.records.len()
	}
	
	fn column_count(&self) -> usize {
		self.records.first().unwrap().columns.len()
	}
	
	pub fn fetch_paged_records(
	&mut self,
	page: usize) {
		let range_upper: usize = if (page * 50) < self.records.len() { page *  50 } else { self.records.len() };
		let range_lower: usize = if range_upper > 50 { range_upper - 50 } else { 0 };
		self.paged_records = self.records[range_lower..range_upper].to_vec();
	}
	
}
/*
impl <T> Default for RecordSet<T>
	where
	T: Default {
		fn default() -> Self {
			Self { 
				column_info: HashMap::<String, T::default()>::new(),
				column_order: Vec::<String>::new(),
				records: Vec::<Record>::new(),
				paged_records: Vec::<Record>::new(),
			}
		}
}
*/
impl Record {
	fn add(&mut self, key: String, val: sqlite::Value) {
		self.columns.insert(key, val);
	}
}


/* <-- Structs */
/* --> Traits */


pub trait SqliteTranslation {
	fn translate(&self) -> String;
}
//convert this to `to_string()` and implement a proper `translate` to convert from sqlite::value to native rust values
impl SqliteTranslation for sqlite::Value {
	fn translate(&self) -> String {
		let mut payload: String = String::new();
		
		match self {
			SqliteFloat(value)		=> payload.push_str(&value.to_string()),
			SqliteInteger(value)	=> payload.push_str(&value.to_string()),
			SqliteString(value)		=> payload.push_str(&value.to_string()),
			SqliteNull				=> payload.push_str("Null"),
			SqliteBinary(value) 	=> { 
				for element in value {
					payload.push_str(&element.to_string())
				} 
			},
		}

		payload
	}
}

/* <-- Traits */