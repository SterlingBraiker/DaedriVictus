use std::io;
use rand::Rng;
use std::cmp::Ordering;

pub struct Guess {
	value: i32,
}

impl Guess {
	pub fn new(value: i32) -> Result<Guess, u8> {
		if value < 1 || value > 100 {
			Err(0)
		} else {
			Ok(Guess { value })
		}
	}
	
	pub fn value(&self) -> i32 {
		self.value
	}
}

pub fn guessingGame() {
    println!("Guess the number!");
	let mut guessesRemaining: i32 = rand::thread_rng().gen_range(1..=10);
	let secret_number: i32 = rand::thread_rng().gen_range(1..=100);
	
	loop {
		let mut userInput:String = String::new();
		println!("Guesses remaining: {}", guessesRemaining);
		if guessesRemaining == 0 { break };
		println!("Please input your guess");
		println!("on the following line");
		
		io::stdin()
			.read_line(&mut userInput)
			.expect("Failed to read line");
		
//First match statement ensures it captures a value that can
//be parsed
		let userGuess: Guess = match userInput.trim().parse() {
			Ok(g) => match Guess::new(g){
				Ok(g) => g,
				Err(_) => continue,
			},
			Err(_) => continue,
		};

		if userGuess.value < 1 || userGuess.value > 100 {
			println!("The secret number will be between 1 and 100.");
			continue;
		}
		
		match userGuess.value.cmp(&secret_number) {
			Ordering::Less => { println!("Too small!"); guessesRemaining -= 1; },
			Ordering::Greater => { println!("Too big!"); guessesRemaining -= 1; },
			Ordering::Equal => { println!("You win!"); break; },
		}
	}
}