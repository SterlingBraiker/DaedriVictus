/* --> Summary of contents

	[lifetimes]
	
	fn longest 
		compare the length of two string slices and return the longer
		
	struct ImportantExcerpt
		
 Summary of contents <-- */

use std::fmt::Display;
 
 struct ImportantExcerpt<'a> {
	part: &'a str,
}
 
 pub fn main2(){
	println!("{}", longest_with_an_announcement("abc", "abcd", "Announcement"));
 }
 
 fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
	if x.len() > y.len() { x } else { y }
 }
 
 fn structLifetimes() {
	let novel: String = String::from("Call me Ishmael. Some years ago...");
	let first_sentence = novel.split(".").next().expect("Could not find a '.'");
	let i = ImportantExcerpt { part: first_sentence, };
	println!("{}", i.level());
	let s: &'static str = "I have a static lifetime. I live for the length of the program.";
 }

impl<'a> ImportantExcerpt<'a> {
	fn level(&self) -> i32 {
		3
	}
}

fn longest_with_an_announcement<'a, T> (
	x: &'a str,
	y: &'a str,
	ann: T,
) -> &'a str where T: Display
{
	println!("Announcement! {}", ann);
	if x.len() > y.len() {
		x
	} else {
		y
	}
}