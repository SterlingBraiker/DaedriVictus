pub mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
        pub fn seat_at_table() {}
    }

    pub mod serving {
        pub fn take_my_order() {}
        pub fn serve_my_order() {}
        pub fn take_payment() {}
    }
}

mod back_of_house {
    pub struct Breakfast {
        pub toast: String,
        seasonal_fruit: String,
    }

    pub enum Appetizer {
        Soup,
        Salad,
    }

    impl Breakfast {
        pub fn summer(toast: &str) -> Breakfast {
            Breakfast {
                toast: String::from(toast),
                seasonal_fruit: String::from("peaches"),
            }
        }

        pub fn new(toast: &str) -> Breakfast {
            Breakfast {
                toast: String::from(toast),
                seasonal_fruit: String::from("Apples"),
            }
        }
    }
}

pub fn eat_at_restaurant() {
    // Absolute Path
    crate::restaurant::front_of_house::hosting::add_to_waitlist();

    // Relative path
    front_of_house::hosting::add_to_waitlist();

    let mut meal = back_of_house::Breakfast::summer("Rye");
    meal.toast = String::from("Wheat");
    println!("I'd like {} toast please", meal.toast);

    let meal = back_of_house::Breakfast::new("white bread");

    println!("Let me change my mind, I want {} toast", meal.toast);

    //let order1 = back_of_house::Appetizer::Soup;
    //let order2 = back_of_house::Appetizer::Salad;

}