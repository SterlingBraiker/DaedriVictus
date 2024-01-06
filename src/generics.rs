/* --> Summary of contents

	[generics]
	
	Struct pointSingleType<T>
		Forces both x & y fields to be the same type T
		
	Struct pointMultiType<T, U>
		Allow fields to be of two different types
		
	largest_i32(list: &[i32]) -> &i32
		Return the largest i32 from a vector
		
	largest_char(list: &[char]) -> &char
		Return the largest u8 char from a vector
		
	largest<T>(list: &[T]) -> &T
		Return the largest item from a vector

	makePoints()
		Demonstrate type declarations in the point structs
		
 Summary of contents <-- */

/* Example
enum Option<T> {
	Some(T),
	None,
}	*/

use std::fmt::Display;

/* --> Structs */

pub struct pointSingleType<T> {
	pub x: T,
	pub y: T,
}

pub struct pointMultiType<T, U> {
	pub x: T,
	pub y: U,
}

/* <-- Structs */
/* --> Traits */

impl<T> pointSingleType<T> {
	pub fn x(&self) -> &T {
		&self.x
	}
	pub fn y(&self) -> &T {
		&self.y
	}
	
	pub fn new(x: T, y: T) -> Self {
		Self { x, y }
	}
}

impl<T: Display + PartialOrd> pointSingleType<T> {
	fn cmp_display(&self) {
		if self.x >= self.y {
			println!("The largest number is x = {}", self.x);
		} else {
			println!("The largest number is y = {}", self.y);
		}
	}
}

/* Haven't figured out how to implement cmp_display trait for pointMultiType
impl<T, U> pointMultiType<T, U> 
where
	T: Display + PartialOrd,
	U: Display + PartialOrd,
{
	fn cmp_display(&self) {
		if self.x >= self.y {
			println!("The largest number is x = {}", self.x);
		} else {
			println!("The largest number is y = {}", self.y);
		}
	}
}
*/


// This method only extends this struct with this type
impl pointSingleType<f32> {
	fn distance_from_origin(&self) -> f32 {
		(self.x.powi(2) + self.y.powi(2)).sqrt()
	}
}

/* <-- Traits */
/* --> Functions */

pub fn entry_point() {
	let a_vec = vec!(3, 5, 10);
	let return_value = largest(&a_vec);
	println!("{}", return_value);
}

pub fn makePoints() {
	let both_integer	= pointSingleType { x: 5, y: 10 };
	let both_float		= pointSingleType { x: 5.0, y: 10.3 };
	
/* Haven't figured out how to implement cmp_display trait for pointMultiType */
//	let different		= pointMultiType { x: 5.2, y: 10 };
	
	both_integer.cmp_display();
	both_float.cmp_display();
//	different.cmp_display();

//	println!("x: {}, y: {}\r\nx: {}, y: {}\r\nx: {}, y: {}", both_integer.x, both_integer.y, both_float.x, both_float.y, different.x, different.y);
}

pub fn largest_i32(list: &[i32]) -> &i32 {
	let mut largest = &list[0];
	
	for item in list {
		if item > largest {
			largest = item;
		}
	}
	
	largest
}

pub fn largest_char(list: &[char]) -> &char {
	let mut largest = &list[0];
	
	for item in list {
		if item > largest {
			largest = item;
		}
	}
	
	largest
}

pub fn largest<T: std::cmp::PartialOrd>(list: &[T]) -> &T {
	let mut largest = &list[0];
	
	for item in list {
		if item > largest {
			largest = item;
		}
	}
	
	largest
}

/* <-- Functions */
