
/*
	translateVecToCSV(vector<vector<usize>>) -> String
		Translate a 2d vector of integers into a CSV string
		
	last_char_of_first_line(&str) -> Option<char>
		provide a string, return the last char of 
		the first line
*/

pub fn translateIntVecToCSV(aVEC: &Vec<Vec<usize>>) -> String {
    let mut accumulator: String = String::new();
    let mLength: usize = aVEC.len() as usize;

    for ROW in aVEC {
        let mut counter: usize = 0;
        for VAL in ROW {
            counter += 1;
            accumulator.push_str(&VAL.clone().to_string());
            if counter < mLength { accumulator.push(','); }
        }
        accumulator.push_str(&String::from('\n'));
    }
    accumulator
}

pub fn translateStringVecToCSV(aVEC: &Vec<Vec<String>>) -> String {
    let mut accumulator: String = String::new();
    let mLength: usize = aVEC.len() as usize;

    for ROW in aVEC {
        let mut counter: usize = 0;
        for VAL in ROW {
            counter += 1;
            accumulator.push_str(&VAL.clone());
            if counter < mLength { accumulator.push(','); }
        }
        accumulator.push_str(&String::from('\n'));
    }
    accumulator
}

pub fn last_char_of_first_line(text: &str) -> Option<char> {
	text.lines().next()?.chars().last()
}