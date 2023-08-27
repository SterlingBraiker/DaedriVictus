
/*
TwoDArrayToCSVString(ARR)
	STR := ""
	For IND1, ROW in ARR
	{
		For IND2, COL in ROW
			STR .= IND2 < ROW.Length ? COL . "," : COL . "`n"
	}
	Return STR
*/

pub fn translateVecToCSV(aVEC: &Vec<Vec<usize>>) -> String {
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