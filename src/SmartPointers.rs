use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
enum List {
	Cons(Rc<RefCell<i32>>, Rc<List>),
	Nil,
}

use crate::SmartPointers::List::{Cons, Nil};

mod boxes {
	use std::ops::Deref;
	
	struct MyBox<T>(T);
	struct CustomSmartPointer {
		data: String,
	}
	
	impl Drop for CustomSmartPointer {
		fn drop(&mut self) {
			println!("About the drop CustomSmartPointer! `{}`!", self.data);
		}
	}
	
	impl<T> MyBox<T> {
		fn new(x: T) -> MyBox<T> {
			MyBox(x)
		}
	}
	
	impl<T> Deref for MyBox<T> {
		type Target = T;
		
		fn deref(&self) -> &Self::Target {
			&self.0
		}
	}
	
	pub fn entry_point() {
		let m = MyBox::new(String::from("Johnson"));
		hello(&m);
		
		let m = CustomSmartPointer { data: String::from("my stuff"), };
		let m2 = CustomSmartPointer { data: String::from("my other stuff"), };
		
		println!("CustomSmartPointers created.");
	}
	
	
	fn hello(name: &str) -> () {
		println!("Hello, {name}!");
	}
}
mod refcount {
	use crate::SmartPointers::List::{Cons, Nil};
	use std::rc::Rc;
	
	pub fn entry_point() {
		let a = Rc::new(Cons(5, Rc::new(Cons(10, Rc::new(Nil)))));
		print_ref_count(&a);
		let b = Cons(3, Rc::clone(&a));
		print_ref_count(&a);
		let c = Cons(4, Rc::clone(&a));
		print_ref_count(&a);
	}
	
	fn print_ref_count<T>(the_list: &Rc<T>) {
		println!("Current reference count: {}", Rc::strong_count(the_list));
	}
}
mod refcells {
	pub trait Messenger {
		fn send(&self, msg: &str);
	}
	
	pub struct LimitTracker<'a, T: Messenger> {
		messenger: &'a T,
		value: usize,
		max: usize,
	}
	
	impl<'a, T> LimitTracker<'a, T>
	where
		T: Messenger,
	{
		pub fn new(messenger: &'a T, max: usize) -> LimitTracker<'a, T> {
			LimitTracker {
				messenger,
				value: 0,
				max,
			}
		}
		
		pub fn set_value(&mut self, value: usize) {
			self.value = value;
			
			let percentage_of_max = self.value as f64 / self.max as f64;
			if percentage_of_max >= 1.0 {
				self.messenger.send("Error: You are over your quota");
			} else if percentage_of_max >= 0.9 {
				self.messenger.send("Urgent warning: You've over 90% of your quota");
			} else if percentage_of_max >= 0.75 {
				self.messenger.send("Warning: You've used 75% of your quota");
			}
			

		}
	}
	
	#[cfg(test)]
	mod tests {
		use super::*;
		use std::cell::RefCell;
		
		struct MockMessenger {
			sent_messages: RefCell<Vec<String>>,
		}
		
		impl MockMessenger {
			fn new() -> MockMessenger {
				MockMessenger {
					sent_messages: RefCell::new(vec![]),
				}
			}
		}
		
		impl Messenger for MockMessenger {
			fn send(&self, message: &str) {
				self.sent_messages.borrow_mut().push(String::from(message));
			}
		}
		
		#[test]
		fn it_sends_an_over_75_percent_warning_message() {
			let mock_messenger = MockMessenger::new();
			let mut limit_tracker = LimitTracker::new(&mock_messenger, 100);
			
			limit_tracker.set_value(80);
			
			assert_eq!(mock_messenger.sent_messages.borrow().len(), 1);
		}
		
	}
	
	pub fn entry_point() {
		
	}
}

mod rcinrefcells {
	
	use crate::SmartPointers::List::{Cons, Nil};
	use std::cell::RefCell;
	use std::rc::Rc;
	
	pub fn entry_point() {
		let value = Rc::new(RefCell::new(5));
		
		let a = Rc::new(Cons(Rc::clone(&value), Rc::new(Nil)));
		
		let b = Cons(Rc::new(RefCell::new(3)), Rc::clone(&a));
		let c = Cons(Rc::new(RefCell::new(4)), Rc::clone(&a));
		
		*value.borrow_mut() += 10;
		
		println!("{}", *value.borrow());
	}
}

mod memoryleakage {
	use crate::SmartPointers::memoryleakage::List::{Cons, Nil};
	use std::cell::RefCell;
	use std::rc::Rc;

	#[derive(Debug)]
	enum List {
		Cons(i32, RefCell<Rc<List>>),
		Nil,
	}
	

	
	impl List {
		fn tail(&self) -> Option<&RefCell<Rc<List>>> {
			match self {
				Cons(_, item) => Some(item),
				Nil => None,
			}
		}
	}
	
	pub fn entry_point() {}
	
}

pub fn entry_point() {
	memoryleakage::entry_point();
}