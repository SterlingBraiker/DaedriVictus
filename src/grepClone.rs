/* --> Summary of Contents

Organizing code (using what you learned about modules in Chapter 7)
Using vectors and strings (collections, Chapter 8)
Handling errors (Chapter 9)
Using traits and lifetimes where appropriate (Chapter 10)
Writing tests (Chapter 11)



Summary of Contents <-- */


/* --> Imports */
mod lib;
use std::process;

/* <-- Imports */

/* --> Functions <-- */

pub fn entry_point() {
	let conf = lib::save_args();
	if let Err(e) = lib::run(conf) {
		println!("Application error: {e}");
		process::exit(1);
	}
}