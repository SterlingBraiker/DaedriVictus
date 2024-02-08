/* --> Imports */

use std::{
    collections::HashMap, env::join_paths, sync::{mpsc, Arc, Mutex}, 
    thread::{self, Builder, JoinHandle}, io::ErrorKind,
};

use fltk::{
    app::{channel, App, Receiver, Scheme, Sender},
    button::Button,
    dialog, enums,
    enums::{Color, Shortcut},
    group::{Flex, Pack},
    input::{Input, MultilineInput},
    menu::{MenuBar, MenuFlag, MenuItem},
    output::MultilineOutput,
    prelude::{GroupExt, InputExt, MenuExt, WidgetBase, WidgetExt, WindowExt},
    window,
};

use crate::odbc_interface::*;
use crate::sqlite3_interface as sqlite3;
use fltk_table::{SmartTable, TableOpts};

use crate::sql_aux_funcs::{
    Connection, Record, RecordSet, SqlData, SqlError, SqlType, Translate,
    ConnectionBase,
};
use crate::AuxFuncs;
use rand::{thread_rng, Rng};

/* <-- Imports */
/* --> Enums */

#[derive(Clone)]
pub enum Message {
    Query(String, QueryFlag),
    FillGrid(i32),
    Save,
    ClearGrid,
    RandomNumber(usize, u64),
    LaunchObserver,
    SqlServerPacket(Option<i32>),
}

#[derive(Clone)]
pub enum QueryFlag {
    Tables,
    UserDefined,
}

/* <-- Enums */
/* --> Structs */

struct FltkHost {
    fltk_app: App,
    fltk_windows: Vec<fltk::window::Window>,
    conn: Connection,
}

/* <-- Structs */
/* --> Const */

static ODBC_TEST_TABLES: &str = "select * from 'results.csv';";
static SQLITE_TABLES: &str = "select name from sqlite_schema where type = 'table' and name not like 'sqlite_%';";

/* <-- Const */
/* --> Functions */

pub fn entry_point() -> Result<(), SqlError> {
    match init_gui() {
        Ok(_) => { Ok(()) },
        Err(E) => { Err(E) },
    }
}

fn init_gui<'a>() -> Result<(), SqlError> {
    
    //testing file input dialogs
    let mut f: FltkHost = FltkHost {
        fltk_app: App::default().with_scheme(Scheme::Oxy),
        fltk_windows: Vec::new(),
        conn: Connection {
            record_set: None,
            connection: None,
            result_code: None,
            result_details: None,
            connection_type: None,
            error_interface: None,
        },
    };

    // create SQL window
    // Create controls
    let sql_window = window::Window::default()
        .with_size(1280, 760)
        .center_screen();
    f.fltk_windows.push(sql_window);

    let mut main_menu: MenuBar = MenuBar::default().with_size(1280, 30).with_pos(0, 0);

    main_menu.add_choice("File|Option");

    let record_grid_group: Flex = Flex::default().with_size(1000, 650).below_of(&main_menu, 5);

    record_grid_group.begin();

    let mut record_grid = SmartTable::default().size_of_parent().with_opts(TableOpts {
        rows: 0,
        cols: 0,
        editable: false,
        cell_font_size: 9,
        header_font_size: 10,
        cell_border_color: enums::Color::Light2,
        ..Default::default()
    });
    record_grid_group.end();

    let tables_grid_group: Flex = Flex::default()
        .with_size(276, 650)
        .right_of(&record_grid_group, 5);

    tables_grid_group.begin();

    let mut table_grid = SmartTable::default()
        .with_size(276, 650)
        .with_opts(TableOpts {
            rows: 20,
            cols: 1,
            editable: false,
            cell_font_size: 9,
            header_font_size: 10,
            cell_border_color: enums::Color::Light2,
            ..Default::default()
        });

    tables_grid_group.end();

    let mut pages_butn = Button::default()
        .with_size(75, 30)
        .below_of(&record_grid_group, 3)
        .with_label("&Page");
    let mut page_input = Input::default().with_size(75, 30).below_of(&pages_butn, 3);
    page_input.set_value("1");
    let mut query_butn = Button::default()
        .with_size(75, 30)
        .right_of(&pages_butn, 4)
        .with_label("&Submit");
    let mut clear_butn = Button::default()
        .with_size(75, 30)
        .below_of(&query_butn, 3)
        .with_label("&Clear");
    let textinput = MultilineInput::default()
        .with_size(836, 63)
        .right_of(&query_butn, 4);
    let mut tables_butn = Button::default()
        .with_size(75, 63)
        .right_of(&textinput, 4)
        .with_label("&Tables");
    let mut save_butn = Button::default()
        .with_size(75, 63)
        .right_of(&tables_butn, 4)
        .with_label("Sa&ve");
    let mut observer_butn = Button::default()
        .with_size(75, 63)
        .right_of(&save_butn, 4)
        .with_label("Observe");

    //create channels
    let (main_app_sender, main_app_receiver) = channel::<Message>();
    let query_butn_sndr = main_app_sender.clone();
    let save_butn_sndr = main_app_sender.clone();
    let tables_butn_sndr = main_app_sender.clone();
    let pages_butn_sndr = main_app_sender.clone();
    let clear_butn_sndr = main_app_sender.clone();
    let observer_butn_sndr = main_app_sender.clone();
    let sql_selector_sndr = main_app_sender.clone();

    query_butn.set_callback({
        move |_| {
            query_butn_sndr.send(Message::Query(textinput.value().clone(), QueryFlag::UserDefined));
            query_butn_sndr.send(Message::FillGrid(1));
        }
    });

    clear_butn.set_callback({
        move |_| {
            clear_butn_sndr.send(Message::ClearGrid);
        }
    });

    save_butn.set_callback({
        move |_| {
            save_butn_sndr.send(Message::Save);
        }
    });

    tables_butn.set_callback({
        move |_| {
            tables_butn_sndr.send(Message::Query(String::from(""), QueryFlag::Tables));
            tables_butn_sndr.send(Message::FillGrid(2));
        }
    });

    pages_butn.set_callback({
        move |_| {
            pages_butn_sndr.send(Message::FillGrid(1));
        }
    });

    observer_butn.set_callback({
        move |_| {
            observer_butn_sndr.send(Message::LaunchObserver);
        }
    });

    main_menu.set_callback(move |m| match m.choice() {
        Some(T) => match &*T {
            "File" => {
                let (x, y): (i32, i32) = center();
                let choice = dialog::choice2(
                    x - 200,
                    y - 100,
                    "Select a server",
                    "Sqlite",
                    "Odbc",
                    "Cancel",
                );
                match choice {
                    Some(0) | Some(1) => sql_selector_sndr.send(Message::SqlServerPacket(choice)),
                    _ => {},
                }

            }
            "Option" => (),
            &_ => (),
        },
        None => (),
    });

    f.fltk_windows[0].end();
    f.fltk_windows[0].show();

    {
        let (x, y): (i32, i32) = center();
        let children_bounds = resize_window_to_children(f.fltk_windows[0].bounds());
        f.fltk_windows[0].resize(
            x - (children_bounds.2 / 2),
            y - (children_bounds.3 / 2),
            children_bounds.2,
            children_bounds.3,
        );
    }

    let mut workers: Vec<JoinHandle<()>> = Vec::<JoinHandle<()>>::new();
    let mut outputs: Vec<MultilineOutput> = Vec::new();

    // enter the event loop which responds to channel messages
    while f.fltk_app.wait() {
        match main_app_receiver.recv() {
            Some(Message::Query(mut qry, query_flag)) => {
                let db_name = match f.conn.connection_type.as_ref() {
                    Some(crate::sql_aux_funcs::ConnectionBase::Odbc) | Some(crate::sql_aux_funcs::ConnectionBase::Sqlite) => f.conn.connection.clone().unwrap(),
                    None => { String::from("None") },
                };
                
                match query_flag {
                    QueryFlag::UserDefined => {},
                    QueryFlag::Tables => qry = match f.conn.connection_type.as_ref() {
                        Some(&ConnectionBase::Sqlite) => { String::from(SQLITE_TABLES) },
                        Some(&ConnectionBase::Odbc) => { String::from(ODBC_TEST_TABLES) },
                        None => { String::from("") },
                    }
                }

                match attempt_query(&qry[..], &db_name[..], f.conn.connection_type.as_ref()) {
                    Ok(value) => {
                        f.conn.assemble_rs(value);
                    }
                    Err(E) => {
                        f.conn.assemble_err(E);
                    }
                }
            }
            Some(Message::Save) => {
                let the_data: String = AuxFuncs::translateStringVecToCSV(&record_grid.data());

                if the_data.len() > 0 {
                    match sqlite3::save_results(the_data) {
                        Ok(_) => {
                            save_successful();
                        }
                        Err(E) => {
                            println!("{E:?}");
                        }
                    };
                };
            }
            Some(Message::FillGrid(table_index)) => {
                if f.conn.result_code == Some(-1) {
                    //check that the query didn't error out
                    match f.conn.result_details {
                        Some(details) => println!("{}", details),
                        None => println!("Empty error message"),
                    }
                    f.conn.result_code = None;
                    f.conn.result_details = None;
                }

                //then fill the grid with the recordset because it passed
                if f.conn.result_code == Some(1) {
                    let page_index: usize = page_input.value().parse::<usize>().unwrap();
                    let table: &mut SmartTable = match table_index {
                        1 => &mut record_grid,
                        2 => &mut table_grid,
                        _ => &mut record_grid,
                    };

                    //slice the recordset into a single page
                    //fill the grid with only <= 50 records
                    let page_of_records: Vec<Record> = match f.conn.record_set {
                        Some(ref rs) => rs.records.clone(),
                        None => { Vec::<Record>::new() },
                    };
                    fill_table(&f.conn.record_set.clone().unwrap(), table, page_of_records);
                }
            }
            Some(Message::ClearGrid) => {
                clear_table(&mut record_grid);
            }
            Some(Message::RandomNumber(indx, num)) => {
                outputs[indx].set_value(&num.to_string()[..]);
            }
            Some(Message::LaunchObserver) => {
                spawn_observer(&mut f, &mut outputs, &mut workers, main_app_sender.clone());
            }
            Some(Message::SqlServerPacket(packet)) => {
                match packet { //sqlite
                    Some(0) => match select_file(&f) {
                        Ok(selected_file) => {
                            f.conn.connection_type = Some(ConnectionBase::Sqlite);
                            f.conn.connection = Some(String::from(selected_file));
                        }
                        Err(E) => println!("Invalid operation during file selection, {E:?}"),
                    }, 
                    Some(1) => { // Odbc
                        let conn_str: String = input_conn_str();
                        f.conn.connection_type = Some(ConnectionBase::Odbc);
                        f.conn.connection = Some(String::from(conn_str));
                        println!("odbc selected");
                        f.conn.record_set = Some(RecordSet::default());
                    }
                    _ => (),
                }
                if f.conn.connection != None {
                    tables_butn.do_callback();
                }
            }
            None => {}
        }
    }
    println!("exited ui event loop");
    Ok(())
}

/*
fn spawn_workers<T, U>(
    count: u64,
    transmitter: Sender<T>) -> JoinHandle<U>
    where
        T: Send + Sync {
    let jhandle: JoinHandle<_> = thread::spawn(|| {
        let generator: rand::rngs::ThreadRng = thread_rng();
        let worker_index: usize = count as usize;
        loop {
            let next_number: u64 = generator.gen();
            transmitter.send(Message::RandomNumber(worker_index, next_number));
            std::thread::sleep(std::time::Duration::from_millis(100 * count));
        }
    });
    jhandle
}
*/

fn save_successful() {
    let center_of_screen: (i32, i32) = center();
    dialog::message(
        center_of_screen.0 - 200,
        center_of_screen.1 - 100,
        "File saved successfully.",
    );
}

fn center() -> (i32, i32) {
    let ss: (f64, f64) = fltk::app::screen_size();
    ((ss.0 / 2.0) as i32, (ss.1 / 2.0) as i32)
}

fn attempt_query(
    textinput: &str,
    db_name: &str,
    db_interface: Option<&ConnectionBase>,
) -> Result<RecordSet, SqlError> {
    let result = match db_interface {
        Some(&ConnectionBase::Odbc) => { crate::odbc_interface::entry_point(String::from(db_name), String::from(textinput)) },
        Some(&ConnectionBase::Sqlite) => { crate::sqlite3_interface::raw_query(String::from(db_name), String::from(textinput)) },
        None => { Err(SqlError::None) },
    };
    result
}

fn clear_table(table: &mut SmartTable) {
    for _ in 0..table.column_count() {
        table.remove_col(0);
    }
    for _ in 0..table.row_count() {
        table.remove_row(0);
    }
}

fn add_columns_to_table<'a>(
    record_set: &'a RecordSet,
    table: &mut SmartTable,
) -> HashMap<&'a String, i32> {
    let mut col_width_map: HashMap<&'a String, i32> =
        HashMap::with_capacity(record_set.column_order.len());
    //add columns
    let mut current_column_index: i32 = 0;
    for column_name in &record_set.column_order {
        table.append_empty_col(&column_name.to_string()[..]);
        let initial_width: i32 = column_name.len() as i32;
        table.set_col_width(current_column_index, initial_width);
        current_column_index += 1;
        col_width_map.insert(column_name, initial_width);
    }
    col_width_map
}

fn resize_columns<'a>(
    record_set: &'a RecordSet,
    col_width_map: &mut HashMap<&'a String, i32>,
    table: &mut SmartTable,
) {
    let mut sort_count: i32 = if record_set.records.len() > 50 {
        50
    } else {
        record_set.records.len() as i32
    };
    let mut current_column_index: i32 = 0;

    for record in &record_set.records {
        for v in &record_set.column_order {
            match record.columns.get(v).unwrap() {
                Some(SqlData::Sqlite(value)) => {
                    match value {
                        sqlite3::SqliteFloat(value) => {
                            let value_stringified = &value.to_string()[..];
                            let previous_size = col_width_map.get(v).copied().unwrap();
                            if (value_stringified.len() as i32) > (previous_size as i32) {
                                col_width_map.insert(v, value_stringified.len().clone() as i32);
                            }
                        }
                        sqlite3::SqliteInteger(value) => {
                            let value_stringified = &value.to_string()[..];
                            let previous_size = col_width_map.get(v).copied().unwrap();
                            if (value_stringified.len() as i32) > (previous_size as i32) {
                                col_width_map.insert(v, value_stringified.len().clone() as i32);
                            }
                        }
                        sqlite3::SqliteString(value) => {
                            let value_stringified = &value.to_string()[..];
                            let previous_size = col_width_map.get(v).copied().unwrap();
                            if (value_stringified.len() as i32) > (previous_size as i32) {
                                col_width_map.insert(v, value_stringified.len().clone() as i32);
                            }
                        }
                        sqlite3::SqliteNull => (),
                        sqlite3::SqliteBinary(value) => {
                            let mut x = String::new();
                            for element in value {
                                x.push_str(&element.to_string()[..])
                            }
                            let value_stringified = &x[..];
                            let previous_size = col_width_map.get(v).copied().unwrap();
                            if (value_stringified.len() as i32) > (previous_size as i32) {
                                col_width_map.insert(v, value_stringified.len().clone() as i32);
                            }
                        }
                    }
                },
                Some(SqlData::Odbc(Value)) => {},
                None => {},
            }
        }
        sort_count -= 1;
        if sort_count == 0 {
            break;
        }
    }

    //resize all columns
    for column_name in &record_set.column_order {
        let col_width: i32 = col_width_map.get(column_name).copied().unwrap();
        table.set_col_width(current_column_index, col_width * 9);
        current_column_index += 1;
    }
}

fn fill_table(
    record_set: &RecordSet,
    table: &mut SmartTable,
    paged_records: Vec<Record>,
) {
    clear_table(table);
    let mut col_width_map: HashMap<&String, i32> = add_columns_to_table(&record_set, table);
    resize_columns(&record_set, &mut col_width_map, table);

    let mut current_record_index: i32 = 0;
    //add rows
    for record in paged_records.iter() {
        table.append_empty_row(&current_record_index.to_string()[..]);
        let mut current_column_index: i32 = 0;
        for v in &record_set.column_order {
            match record.columns.get(v) {
                Some(value) => {
                    match value {
                        Some(data) => {
                            table.set_cell_value(
                            current_record_index,
                            current_column_index,
                            &data.translate()[..],
                            );
                        },
                        None => {},
                    }
                    
                }
                None => {}
            }
            current_column_index += 1;
        }
        current_record_index += 1;
    }
}

fn resize_window_to_children<T>(bounds: Vec<(T, T, T, T)>) -> (T, T, T, T)
where
    T: std::cmp::PartialOrd + std::default::Default,
{
    let mut boundary: (T, T, T, T) = (T::default(), T::default(), T::default(), T::default());

    for current_boundary in bounds {
        if boundary.0 > current_boundary.0 {
            boundary.0 = current_boundary.0
        }
        if boundary.1 > current_boundary.1 {
            boundary.1 = current_boundary.1
        }
        if boundary.2 < current_boundary.2 {
            boundary.2 = current_boundary.2
        }
        if boundary.3 < current_boundary.3 {
            boundary.3 = current_boundary.3
        }
    }

    boundary
}

fn spawn_observer(
    f: &mut FltkHost,
    outputs: &mut Vec<MultilineOutput>,
    workers: &mut Vec<JoinHandle<()>>,
    sndr: Sender<Message>,
) {
    //create hud window
    let hud_window = window::Window::default().with_size(320, 320);
    f.fltk_windows.push(hud_window);

    let mut packer: Pack = Pack::default().size_of_parent();
    packer.set_spacing(5);
    packer.set_frame(fltk::enums::FrameType::ThinUpFrame);
    packer.set_color(fltk::enums::Color::Black);

    for x in 0..=3 {
        //add 4 references to a vector, which will be used in the event loop below
        let output: MultilineOutput = MultilineOutput::default().with_size(500, 500);
        packer.add(&output);
        outputs.push(output);
        let new_sender = sndr.clone();
        let worker_index: usize = x.clone();
        let sleeper_offset: u64 = x.clone().try_into().unwrap();

        //omitted while I try to understand spawn_workers parameter declarations
        //		let jhandle: JoinHandle<()> = spawn_workers(worker_index, sleeper_offset, new_sender);

        // implement workers in the same context instead
        /* 		let jhandle: JoinHandle<_> = thread::spawn(move || {
                    let mut generator: rand::rngs::ThreadRng = thread_rng();
                    loop {
                        let next_number: u64 = generator.gen();
                        new_sender.send(Message::RandomNumber(worker_index, next_number.clone()));
                        std::thread::sleep(std::time::Duration::from_millis((100 + ((sleeper_offset + 1) * 80)).try_into().unwrap()));
                    }
                });
        */
        let builder: Builder = thread::Builder::new().name(worker_index.clone().to_string());
        let handlers: JoinHandle<()> = builder
            .spawn(move || {
                let mut generator: rand::rngs::ThreadRng = thread_rng();
                loop {
                    let next_number: u64 = generator.gen();
                    new_sender.send(Message::RandomNumber(worker_index, next_number.clone()));
                    std::thread::sleep(std::time::Duration::from_millis(
                        (100 + ((sleeper_offset + 1) * 80)).try_into().unwrap(),
                    ));
                }
            })
            .unwrap();

        workers.push(handlers);

        // for some reason, removing this sleep can cause a random panic among threads
        // need to investigate
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    packer.auto_layout();

    f.fltk_windows[1].end();
    f.fltk_windows[1].show();
}

fn select_file(f: &FltkHost) -> Result<String, std::io::Error> {
    let mut fi =
        dialog::FileChooser::new(".", "*.db", dialog::FileChooserType::Single, "Select a DB");
    let center_of_screen: (i32, i32) = center();
    fi.show();
    fi.window().set_pos(center_of_screen.0, center_of_screen.1);
    while fi.shown() {
        f.fltk_app.wait();
    }
    match fi.value(1) {
        Some(choice) => { Ok(choice) },
        None => { Err(std::io::Error::new(ErrorKind::Other, "Cancelled operation")) },
    }
}

fn input_conn_str() -> String {
    let input: String = match fltk::dialog::input(15, 15, "Enter a connection string", "") {
        Some(input) => input,
        None => String::from(""),
    };

    input
}

/* <-- Functions */
