/* --> Summary of Contents

	pub fn shared_state()
		Create 10 threads and allow a mutable reference to
		a single i32 value, which is counted up to 10.

<-- Summary of Contents */

/* --> imports */

use std::sync::{Arc, Mutex};
use std::thread;

/* <--  imports */


pub fn shared_state() {
	let counter = Arc::new(Mutex::new(0));
	let mut handles = vec![];
	
	for _ in 0..10 {
		let counter = Arc::clone(&counter);
		let handle = thread::spawn(move || {
			let mut num = counter.lock().unwrap();
			println!("num: {}", num);
			*num += 1 as i32;
		});
		
		handles.push(handle);
	}
	
	for handle in handles {
		handle.join().unwrap();
	}
	
	println!("Result: {}", *counter.lock().unwrap());
}