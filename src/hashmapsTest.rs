use std::collections::HashMap;

/* 
  Given a list of integers, use a vector and return the median 
  (when sorted, the value in the middle position) and mode 
  (the value that occurs most often; a hash map will be helpful here) 
  of the list.
  vec![1, 2, 4, 3, 1, 1, 2, 2, 3, 1, 1];
*/
pub fn test1(){
    let mut VEC: Vec<i32> = vec![1, 2, 4, 3, 1, 1, 2, 2, 3, 1, 1];
    VEC.sort();
    let meanPos: f32 = (VEC.len() as f32 / 2.0).round();
    let mean = VEC.get(meanPos as usize);
    println!("{}", mean.unwrap());

    let mut MAP: HashMap<i32, i32> = HashMap::new();

    for INT in VEC.iter() { 
        MAP.entry(*INT).and_modify(|VAL| { *VAL += 1 }).or_insert(1);
    };

    let mut largest: HashMap<&str, i32> = HashMap::new();
    
    largest.insert("value", 0);
    largest.insert("count", 0);
    for (k, v) in MAP {
        if v > largest["count"] {
            largest.entry("value").and_modify(|TMP| { *TMP = k.clone() });
            largest.entry("count").and_modify(|TMP| { *TMP = v.clone() });
        }
    }
    println!("{}", largest["value"]);

}

/*
    Convert strings to pig latin. The first consonant of each word
    is moved to the end of the word and “ay” is added, so “first” 
    becomes “irst-fay.” Words that start with a vowel have “hay” 
    added to the end instead (“apple” becomes “apple-hay”). Keep 
    in mind the details about UTF-8 encoding!
*/

