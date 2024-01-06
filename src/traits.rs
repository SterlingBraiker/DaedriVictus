/* --> Summary of contents



<--  Summary of contents */

/* --> imports */

use std::fmt::Display;


/* <--  imports */


pub struct NewsArticle {
	pub headline:	String,
	pub location:	String,
	pub author:		String,
	pub content:	String,
}

pub struct Tweet {
	pub username:	String,
	pub content:	String,
	pub reply:		bool,
	pub retweet:	bool,
}

impl Summary for NewsArticle {
	
	/*
	//force NewsArticle to fetch the default
	//trait implementation of summarize()
	
	fn summarize(&self) -> String {
		format!("{}, {} ({})", self.headline, self.author, self.location)
	}
	*/
	
	fn summarize_author(&self) -> String { format!("{}", self.author) }
}

impl Summary for Tweet {
	fn summarize(&self) -> String {
		format!("{}: {}", self.username, self.content)
	}
	
	fn summarize_author(&self) -> String {
		format!("@{}", self.username)
	}
}

//example of a trait definition
pub trait Summary {
	fn summarize_author(&self) -> String;

	fn summarize(&self) -> String {
		format!("(Read more from {}... )", self.summarize_author())
	}
}

pub fn notify(item: &impl Summary) {
	println!("Breaking news! {}", item.summarize());
}

/*	Explicit syntax
	// Accepts a single type
pub fn notify<T: Summary>(item: &T) {}

	//Accept two parameters of the same type that both implement Summary
pub fn notify<T: Summary>(item1: &T, item2: &T) {}

	//Multiple traits. With the two trait bounds specified, the body of 
	//notify can call summarize and use {} to format item.
pub fn notify<T: Summary + Display>(item: &T) {}

*/