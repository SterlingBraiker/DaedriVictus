use std::io;

pub fn guess() {
    println!("Guess the number!");
	println!("Please input your guess");
	println!("on the following line");
	
	let mut guess:String = String::new();
	
	io::stdin()
		.read_line(&mut guess)
		.expect("Failed to read line");
		
	println!("You guessed: {guess}");
}
