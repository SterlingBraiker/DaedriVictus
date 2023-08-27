/* --> imports */
use std::collections::HashMap;
/* <--  imports */

pub fn mapper() {
    let mut scores: HashMap<String, i32> = HashMap::new();

    scores.insert(String::from("Blue"), 10 as i32);
    scores.insert(String::from("Yellow"), 50 as i32);

    let team_name: String = String::from("Blue");
    let score: i32 = scores.get(&team_name).copied().unwrap_or(0);

    println!("score: {}", score);
}