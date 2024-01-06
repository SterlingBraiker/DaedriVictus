/* --> Summary of contents

	[threads]
	
	join_threads()
		demonstrate *.join() method on a thread. This
		blocks the main thread until the secondary thread
		exits

	move_closures()
		Allows threads to take ownership of variables from
		the spawning thread's scope 
		
 Summary of contents <-- */

/* --> imports */ 
use std::thread;
use std::time::Duration;
/* <-- imports */

pub fn join_threads() {
	let handle = thread::spawn( || {
		for i in 1..10 {
			println!("hi number {} from the spawned thread", i);
			thread::sleep(Duration::from_millis(1));
		}
	});
	
	handle.join().unwrap();

	for i in 1..5 {
		println!("hi number {} from the main thread!", i);
		thread::sleep(Duration::from_millis(1));
	}
}

pub fn move_closures() {
	let v:Vec<i32> = vec![1, 2, 3];
	let handle:std::thread::JoinHandle<_> = thread::spawn(move || {
		println!("Here's a vector: {:?}", v);
	});
	
	handle.join().unwrap();
}