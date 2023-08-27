/* --> directives */
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]
/* <-- directives */



/* --> imports */
mod guessing_game;
mod restaurant;
mod hashmaps;
mod AuxFuncs;

use crate::AuxFuncs::*;
/* <--  imports */



fn main() {
    let aVec: Vec<Vec<usize>> = vec![vec![1, 2, 3], vec![1, 2, 3], vec![1, 2, 3]];
    let aString: String = translateVecToCSV(&aVec);
    println!("aString:\n{}", aString);
}