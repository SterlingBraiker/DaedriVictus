/* --> directives */

#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]

/* <-- directives */
/* --> imports */

mod AuxFuncs;
mod fltk_messages;
mod learning_fltk;
mod odbc_interface;
mod sql_aux_funcs;
mod sqlite3_interface;

/* <--  imports */
/* -->  Functions */

fn main() -> Result<(), sqlite::Error> {
    learning_fltk::entry_point();
    Ok(())
}

/* <--  Functions */

/* --> Dead Code Examples

   --> [Generics Examples]

       let p = generics::pointSingleType { x: 5, y: 10 };
       println!("p.x = {}", p.x());
       println!("p.y = {}", p.y());

   [==]

       let number_list = vec![34, 50, 25, 100, 65];
       let result = generics::largest_i32(&number_list);
       println!("The largest number is: {}", result);

       let char_list = vec!['y', 'm', 'q', 'i'];
       let result = generics::largest_char(&char_list);
       println!("the largest char is {}", result);

   <-- [Generics Examples]


   --> [AuxFunc Examples]

       println!("{}", AuxFuncs::last_char_of_first_line("text line").unwrap());

   <-- [AuxFunc Examples]

   --> [Traits Examples]
       let article = NewsArticle {
           headline: String::from("Penguins win the Stanley Cup Championship!"),
           location: String::from("Pittsburgh, PA, USA"),
           author: String::from("Iceburgh"),
           content: String::from(
               "The Pittsburgh Penguins once again are the best \
               hockey team in the NHL.",
           ),
       };

       println!("New article available! {}", article.summarize());

       let newtweet = Tweet {
           username: String::from("tbol"),
           content: String::from("mauricon"),
           reply: false,
           retweet: false,
       };

       println!("Testing tweet: {}", newtweet.summarize());


   <-- [Traits Examples]

Dead Code Examples <-- */
