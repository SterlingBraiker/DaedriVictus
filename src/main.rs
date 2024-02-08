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

fn main() -> Result<(), sql_aux_funcs::SqlError> {
    learning_fltk::entry_point()
}

/* <--  Functions */