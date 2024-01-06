/* --> Summary of Contents

Organizing code (using what you learned about modules in Chapter 7)
Using vectors and strings (collections, Chapter 8)
Handling errors (Chapter 9)
Using traits and lifetimes where appropriate (Chapter 10)
Writing tests (Chapter 11)

Summary of Contents <-- */


/* --> Imports */
use std::env;
use std::fs;
use std::process;
use std::error::Error;

/* <-- Imports */

/* Structs --> */

pub struct Config {
	query: String,
	file_path: String,
}

impl Config {
	fn build(args: &[String]) -> Result<Config, &'static str> {
		if args.len() < 3 { return Err("not enough arguments"); }
		
		let query = args[1].clone();
		let file_path = args[2].clone();
		
		Ok(Config { query, file_path })
	}
}

/* <-- Structs */

/* --> Functions */

pub fn save_args() -> Config {
	let args: Vec<String> = env::args().collect();
	let conf = Config::build(&args).unwrap_or_else(|err| {
		println!("Problem parsing arguments: {err}");
		process::exit(1);
	});

	conf
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
	let contents = fs::read_to_string(config.file_path)?;
		
	for line in search(&config.query, &contents) {
		println!("{line}");
	}
	
	Ok(())
}

fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
	let mut results: Vec<&str> = Vec::new();
	
	for line in contents.lines() {
		if line.contains(query) {
		 results.push(line);
		}
	}
	results
}

/* <-- Functions */

/* --> Modules */

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn one_result() {
		let query = "duct";
		let contents = "\
Rust:
safe, fast, productive.
Pick three.";
		assert_eq!(vec!["safe, fast, productive."], search(query, contents));
	}
}

/* <-- Modules */