/* --> Panics & Errors <-- */

/* --> Summary of contents

	[panics]
		
	panicFunc()
		Force the program into a panic
	
	illegalAccess()
		Force a panic by accessing an out of bounds index
	
	fileAccess()
		Handle a Result<T, E> with panics if file is not found
		
	fileAccessUnwrap()
		Similarly handle a Result<T, E> to fileAccess() using
		the *.unwrap() method, however it can only panic

	fileAccessExpect()
		Similarly handle a Result<T, E> to fileAccessUnwrap()
		however you can provide more context during a panic.i
		
	read_username_from_file() -> Result<String, io::Error>
		Attempt to open a file, propagate any Errors.
		On successf, read the file to a variable, propagate
		the username on success or the error on failure.
		
	read_username_from_file_maybe_operator() -> Result<String, io::Error>
		Condensing the syntax from read_username_from_file
		using the ? operator
		
	shorter_read_username_from_file() -> Result<String, io::Error>
		Chaining the methods from read_username_from_file_maybe_operator
		
	shortest_read_username_from_file() -> Result<String, io::Error>
		leverage the standard library fs module for a one-liner 
		version of read_username_from_file()

--> Summary of contents */

/* --> imports <-- */
use std::fs::{self, File};
use std::io::{self, Read, ErrorKind};


pub fn panicFunc() {
	panic!("crash and burn");
}

pub fn illegalAccess() {
	let v = vec![1, 2, 3];
	v[99];
}

pub fn fileAccess() {
	let greeting_file_result = File::open("hello.txt");
	
	let greetings_file = match greeting_file_result {
		Ok(file) => file,
		Err(error) => match error.kind() {
			ErrorKind::NotFound => match File::create("hello.txt") {
				Ok(fc) => fc,
				Err(e) => panic!("Problem creating the file: {:?}", e),
			},
			other_error => {
				panic!("Problem opening the file: {:?}", other_error);
			}
		},
	};
}

pub fn fileAccessUnwrap() {
	let greeting_file = File::open("hellot.txt").unwrap();
}

pub fn fileAccessExpect() {
	let greeting_file = File::open("hello.txt")
		.expect("Hello.txt should be included in this project");
}

pub fn read_username_from_file() -> Result<String, io::Error> {
	let username_file_result = File::open("hello.txt");
	
	let mut username_file = match username_file_result {
		Ok(file) => file,
		Err(e) => return Err(e),
	};
	
	let mut username = String::new();
	
	match username_file.read_to_string(&mut username) {
		Ok(_) => Ok(username),
		Err(e) => Err(e),
	}
}

pub fn read_username_from_file_maybe_operator() -> Result<String, io::Error> {
	let mut username_file = File::open("hello.txt")?;
	let mut username = String::new();
	username_file.read_to_string(&mut username)?;
	Ok(username)
}

pub fn shorter_read_username_from_file() -> Result<String, io::Error> {
	let mut username = String::new();
	File::open("hello.txt")?.read_to_string(&mut username)?;
	Ok(username)
}

pub fn shortest_read_username_from_file() -> Result<String, io::Error> {
	fs::read_to_string("hello.txt")
}

