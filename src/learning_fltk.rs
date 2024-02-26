/* --> Imports */

use std::{
    collections::HashMap, env::join_paths, sync::{mpsc, Arc, Mutex}, 
    thread::{self, Builder, JoinHandle}, io::ErrorKind,
};

use fltk::{
    app::{channel, App, Receiver, Scheme, Sender, WidgetId, widget_from_id},
    button::Button,
    dialog, enums,
    enums::{Color, Shortcut},
    group::{Flex, Pack},
    input::{Input, MultilineInput},
    menu::{MenuBar, MenuFlag, MenuItem},
    output::MultilineOutput,
    prelude::{GroupExt, InputExt, MenuExt, WidgetBase, WidgetExt, WindowExt, TableExt},
    window,
};

use crate::odbc_interface::*;
use crate::sqlite3_interface as sqlite3;
use fltk_table::{SmartTable, TableOpts};

use crate::sql_aux_funcs::{
    Connection, Record, RecordSet, SqlData, SqlType, Translate,
    ConnectionBase, Request, QueryType,
};
use crate::AuxFuncs;
use rand::{thread_rng, Rng};

/* <-- Imports */
/* --> Enums */

#[derive(Clone)]
pub enum Message {
    Query(QueryType, FetchFlag),
    FillGrid(i32),
    Save,
    ClearGrid,
    RandomNumber(usize, u64),
    LaunchObserver,
    SqlServerPacket(Option<i32>),
}

#[derive(Clone)]
pub enum FetchFlag {
    True,
    False,
}

/* <-- Enums */
/* --> Structs */

struct FltkHost<'a> {
    fltk_app: App,
    fltk_windows: Vec<fltk::window::Window>,
    conn: Connection,
    sender: Option<Sender<Message>>,
    receiver: Option<Receiver<Message>>,
    working_grid: Option<&'a mut SmartTable>,
}

impl<'a> FltkHost<'a> {
    fn new() -> Self {
        FltkHost {
            fltk_app: App::default().with_scheme(Scheme::Oxy),
            fltk_windows: Vec::new(),
            conn: Connection {
                record_set: None,
                connection: None,
                result_code: None,
                result_details: None,
                connection_type: None,
            },
            receiver: None,
            sender: None,
            working_grid: None,
        }
    }

    fn construct(&mut self) -> Result<(), String> {
        self.fltk_windows.push(window::Window::default()
        .with_id("sql_window")
        .with_size(1280, 760)
        .center_screen());

        MenuBar::default().with_id("main_menu").with_size(1280, 30).with_pos(0, 0); //main menu
        fltk::app::widget_from_id::<fltk::menu::MenuBar>("main_menu").as_mut().unwrap().add_choice("File|Option");

        Flex::default().with_id("record_grid_group").with_size(1000, 650).below_of(fltk::app::widget_from_id::<fltk::menu::MenuBar>("main_menu").as_ref().unwrap(), 5);
        fltk::app::widget_from_id::<fltk::group::Flex>("record_grid_group").as_ref().unwrap().begin();

        SmartTable::default()
        .size_of_parent()
        .with_opts(TableOpts {
            rows: 0,
            cols: 0,
            editable: false,
            cell_font_size: 9,
            header_font_size: 10,
            cell_border_color: enums::Color::Light2,
            ..Default::default()
        }).set_id("record_grid");

        fltk::app::widget_from_id::<fltk::group::Flex>("record_grid_group").as_ref().unwrap().end();

        Flex::default()
        .with_id("tables_grid_group")
        .with_size(276, 325)
        .right_of(fltk::app::widget_from_id::<fltk::group::Flex>("record_grid_group").as_ref().unwrap(), 5);
    
        fltk::app::widget_from_id::<fltk::group::Flex>("tables_grid_group").as_ref().unwrap().begin();
    
        SmartTable::default()
        .with_size(276, 325)
        .with_opts(TableOpts {
            rows: 20,
            cols: 1,
            editable: false,
            cell_font_size: 9,
            header_font_size: 10,
            cell_border_color: enums::Color::Light2,
            ..Default::default()
        }).set_id("tables_grid");
    
        fltk::app::widget_from_id::<fltk::group::Flex>("tables_grid_group").as_ref().unwrap().end();

        Flex::default()
        .with_id("columns_grid_group")
        .with_size(276, 325)
        .below_of(fltk::app::widget_from_id::<fltk::group::Flex>("tables_grid_group").as_ref().unwrap(), 5);

        fltk::app::widget_from_id::<fltk::group::Flex>("columns_grid_group").as_ref().unwrap().begin();

        SmartTable::default()
        .with_size(276, 325)
        .with_opts(TableOpts {
            rows: 20,
            cols: 1,
            editable: false,
            cell_font_size: 9,
            header_font_size: 10,
            cell_border_color: enums::Color::Light2,
            ..Default::default()
        }).set_id("columns_grid");

        fltk::app::widget_from_id::<fltk::group::Flex>("columns_grid_group").as_ref().unwrap().end();

        Button::default()
            .with_size(75, 30)
            .below_of(fltk::app::widget_from_id::<fltk::group::Flex>("record_grid_group").as_ref().unwrap(), 3)
            .with_label("&Page")
            .with_id("pages_butn");
        
        Input::default().with_id("pages_input").with_size(75, 30).below_of(fltk::app::widget_from_id::<fltk::button::Button>("pages_butn").as_ref().unwrap(), 3);
        fltk::app::widget_from_id::<fltk::input::Input>("pages_input").as_mut().unwrap().set_value("1");
        
        Button::default()
            .with_id("query_butn")
            .with_size(75, 30)
            .right_of(fltk::app::widget_from_id::<fltk::input::Input>("pages_input").as_ref().unwrap(), 4)
            .with_label("&Submit");
        Button::default()
            .with_id("clear_butn")
            .with_size(75, 30)
            .below_of(fltk::app::widget_from_id::<fltk::button::Button>("query_butn").as_ref().unwrap(), 3)
            .with_label("&Clear");
        MultilineInput::default()
            .with_id("text_input")
            .with_size(836, 63)
            .right_of(fltk::app::widget_from_id::<fltk::button::Button>("query_butn").as_ref().unwrap(), 4);
        Button::default()
            .with_id("tables_butn")
            .with_size(75, 63)
            .right_of(fltk::app::widget_from_id::<fltk::input::MultilineInput>("text_input").as_ref().unwrap(), 4)
            .with_label("&Tables");
        Button::default()
            .with_id("save_butn")
            .with_size(75, 63)
            .right_of(fltk::app::widget_from_id::<fltk::button::Button>("tables_butn").as_ref().unwrap(), 4)
            .with_label("Sa&ve");
        Button::default()
            .with_id("observer_butn")
            .with_size(75, 63)
            .right_of(fltk::app::widget_from_id::<fltk::button::Button>("save_butn").as_ref().unwrap(), 4)
            .with_label("Observe");


        // event handling, message passing

        {
            let (a, b) = channel::<Message>();
            self.sender = Some(a);
            self.receiver = Some(b);
        }

        let query_butn_sndr:    Sender<Message> = self.sender.as_ref().unwrap().clone();
        let save_butn_sndr:     Sender<Message> = self.sender.as_ref().unwrap().clone();
        let tables_butn_sndr:   Sender<Message> = self.sender.as_ref().unwrap().clone();
        let pages_butn_sndr:    Sender<Message> = self.sender.as_ref().unwrap().clone();
        let clear_butn_sndr:    Sender<Message> = self.sender.as_ref().unwrap().clone();
        let observer_butn_sndr: Sender<Message> = self.sender.as_ref().unwrap().clone();
        let sql_selector_sndr:  Sender<Message> = self.sender.as_ref().unwrap().clone();
        let tables_grid_sndr:   Sender<Message> = self.sender.as_ref().unwrap().clone();

        fltk::app::widget_from_id::<fltk::button::Button>("query_butn")
        .as_mut()
        .unwrap()
        .set_callback({
            move |_| {
                query_butn_sndr.send(Message::Query(QueryType::UserDefined(fltk::app::widget_from_id::<fltk::input::Input>("text_input").as_ref().unwrap().value().clone()), FetchFlag::False));
                query_butn_sndr.send(Message::FillGrid(1));
            }
        });
    
        fltk::app::widget_from_id::<fltk::button::Button>("clear_butn")
        .as_mut()
        .unwrap()
        .set_callback({
            move |_| {
                clear_butn_sndr.send(Message::ClearGrid);
            }
        });
    
        fltk::app::widget_from_id::<fltk::button::Button>("save_butn")
        .as_mut()
        .unwrap()
        .set_callback({
            move |_| {
                save_butn_sndr.send(Message::Save);
            }
        });
    
        fltk::app::widget_from_id::<fltk::button::Button>("tables_butn")
        .as_mut()
        .unwrap()
        .handle(move |tables_butn, ev: fltk::enums::Event| {
            match ev {
                fltk::enums::Event::Push => {
                    tables_butn_sndr.send(Message::Query(QueryType::SqlFunction(Request::Tables(2)), FetchFlag::False));
                    tables_butn_sndr.send(Message::FillGrid(2));
                    true
                },
                _ => false,
            }
        });
    
        fltk::app::widget_from_id::<fltk::button::Button>("pages_butn")
        .as_mut()
        .unwrap()
        .set_callback({
            move |_| {
                pages_butn_sndr.send(Message::FillGrid(1));
            }
        });
    
        fltk::app::widget_from_id::<fltk::button::Button>("observer_butn")
        .as_mut()
        .unwrap()
        .set_callback({
            move |_| {
                observer_butn_sndr.send(Message::LaunchObserver);
            }
        });

        fltk::app::widget_from_id::<fltk_table::SmartTable>("tables_grid")
        .as_mut()
        .unwrap()
        .handle(move |tables_grid_ref, ev: fltk::enums::Event| {
            match ev {
                fltk::enums::Event::Push => {
                    if tables_grid_ref.get_selection() != (-1, -1, -1, -1) {
                        tables_grid_sndr.send(Message::Query(QueryType::SqlFunction(Request::Columns(String::new())), FetchFlag::True));
                        tables_grid_sndr.send(Message::FillGrid(3));
                    }
                    true
                },
                _ => false,
            }
        });
    
        fltk::app::widget_from_id::<fltk::menu::MenuBar>("main_menu")
        .as_mut()
        .unwrap()
        .set_callback(move |m| match m.choice() {
            Some(T) => match &*T {
                "File" => {
                    let (x, y): (i32, i32) = center();
                    let choice = dialog::choice2(
                        x - 200,
                        y - 100,
                        "Select a server type",
                        "Cancel",
                        "Odbc",
                        "Sqlite",
                    );
                    match choice {
                        Some(2) | Some(1) => sql_selector_sndr.send(Message::SqlServerPacket(choice)),
                        _ => {},
                    }
    
                }
                "Option" => (),
                &_ => (),
            },
            None => (),
        });
    
        self.fltk_windows[0].end();
        self.fltk_windows[0].show();


        {
            let (x, y): (i32, i32) = center();
            let children_bounds = resize_window_to_children(self.fltk_windows[0].bounds());
            self.fltk_windows[0].resize(
                x - (children_bounds.2 / 2),
                y - (children_bounds.3 / 2),
                children_bounds.2,
                children_bounds.3,
            );
        }

//        let mut workers: Vec<JoinHandle<()>> = Vec::<JoinHandle<()>>::new();
//        let mut outputs: Vec<MultilineOutput> = Vec::new();
        Ok(())
    }

    fn event_loop(&mut self) -> Result<(), String> {
        while self.fltk_app.wait() {
            match self.receiver.as_ref().unwrap().recv() {
                Some(Message::Query(mut query, fetch_flag)) => {
                    let db_name = match self.conn.connection_type.as_ref() {
                        Some(crate::sql_aux_funcs::ConnectionBase::Odbc) | Some(crate::sql_aux_funcs::ConnectionBase::Sqlite) => self.conn.connection.clone().unwrap(),
                        None => { String::from("None") },
                    };
                    if fltk::app::widget_from_id::<fltk_table::SmartTable>("tables_grid").as_ref().unwrap().get_selection() > (-1, -1, -1, -1) {
                        let mut table_name: String = String::new();
                        table_name = match fetch_flag {
                            FetchFlag::True => {
                                let (row, col, _, _) = fltk::app::widget_from_id::<fltk_table::SmartTable>("tables_grid").as_ref().unwrap().get_selection();
                                fltk::app::widget_from_id::<fltk_table::SmartTable>("tables_grid").as_ref().unwrap().cell_value(row, col)
                            },
                            FetchFlag::False => { String::new() }
                        };
                        query = QueryType::SqlFunction(Request::Columns(table_name));
                        fltk::app::widget_from_id::<fltk_table::SmartTable>("tables_grid").as_mut().unwrap().unset_selection();
                    };
                    match attempt_query(query, &db_name[..], self.conn.connection_type.as_ref()) {
                        Ok(value) => {
                            self.conn.assemble_rs(value);
                        }
                        Err(E) => {
                            println!("{}", E);
                        }
                    }
                },
                Some(Message::Save) => {
                    let the_data: String = AuxFuncs::translateStringVecToCSV(&fltk::app::widget_from_id::<fltk_table::SmartTable>("record_grid").as_ref().unwrap().data());
    
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
                },
                Some(Message::FillGrid(table_index)) => {
                    if self.conn.result_code == Some(-1) {
                        //check that the query didn't error out
                        match self.conn.result_details.as_ref() {
                            Some(details) => println!("{}", details),
                            None => println!("Empty error message"),
                        }
                        self.conn.result_code = None;
                        self.conn.result_details = None;
                    }
    
                    //then fill the grid with the recordset because it passed
                    if self.conn.result_code == Some(1) {
                       /*
                        let page_index: usize = fltk::app::widget_from_id::<fltk::input::Input>("pages_input")
                            .unwrap()
                            .value()
                            .parse::<usize>()
                            .unwrap(); */
                        
                        let mut x = match table_index {
                            1 => { fltk::app::widget_from_id::<fltk_table::SmartTable>("record_grid").unwrap() },
                            2 => { fltk::app::widget_from_id::<fltk_table::SmartTable>("tables_grid").unwrap() },
                            3 => { fltk::app::widget_from_id::<fltk_table::SmartTable>("columns_grid").unwrap() },
                            _ => { fltk::app::widget_from_id::<fltk_table::SmartTable>("record_grid").unwrap() },
                        };
    
                        //slice the recordset into a single page
                        //fill the grid with only <= 50 records
                        let page_of_records: Vec<Record> = match self.conn.record_set {
                            Some(ref rs) => rs.records.clone(),
                            None => { Vec::<Record>::new() },
                        };
                        fill_table(&self.conn.record_set.clone().unwrap(), &mut x, page_of_records);
                    }
                },
                Some(Message::ClearGrid) => {
                    clear_table(&mut fltk::app::widget_from_id::<fltk_table::SmartTable>("record_grid").unwrap());
                },
                Some(Message::RandomNumber(indx, num)) => {
//                    outputs[indx].set_value(&num.to_string()[..]);
                },
                Some(Message::LaunchObserver) => {
//                    spawn_observer(&mut self, &mut outputs, &mut workers, main_app_sender.clone());
                },
                Some(Message::SqlServerPacket(packet)) => {
                    match packet { //sqlite
                        Some(2) => match self.select_file() {
                            Ok(selected_file) => {
                                self.conn.connection_type = Some(ConnectionBase::Sqlite);
                                self.conn.connection = Some(String::from(selected_file));
                            }
                            Err(E) => println!("Invalid operation during file selection, {E:?}"),
                        }, 
                        Some(1) => { // Odbc
                            let conn_str: String = self.input_conn_str();
                            self.conn.connection_type = Some(ConnectionBase::Odbc);
                            self.conn.connection = Some(String::from(conn_str));
                            println!("odbc selected");
                            self.conn.record_set = Some(RecordSet::default());
                        }
                        _ => (),
                    }
                    if self.conn.connection != None {
                        fltk::app::widget_from_id::<fltk::button::Button>("tables_butn").unwrap().handle_event(fltk::enums::Event::Push);
                    }
                },
                None => {},
            }
        }
        println!("exited ui event loop");
        Ok(())
    }

    fn select_file(&mut self) -> Result<String, std::io::Error> {
        let mut fi =
            dialog::FileChooser::new(".", "*.db", dialog::FileChooserType::Single, "Select a DB");
        let center_of_screen: (i32, i32) = center();
        fi.show();
        fi.window().set_pos(center_of_screen.0, center_of_screen.1);
        while fi.shown() {
            self.fltk_app.wait();
        }
        match fi.value(1) {
            Some(choice) => { Ok(choice) },
            None => { Err(std::io::Error::new(ErrorKind::Other, "Cancelled operation")) },
        }
    }
    
    fn input_conn_str(&mut self) -> String {
        let input: String = match fltk::dialog::input(15, 15, "Enter a connection string", "") {
            Some(input) => input,
            None => String::from(""),
        };
    
        input
    }
}

/* <-- Structs */
/* --> Const */
/*
static ODBC_TEST_TABLES: &str = "ZXY";
static ODBC_TEST_COLUMNS: &str = "ZXZ";
static SQLITE_TABLES: &str = "select name from sqlite_schema where type = 'table' and name not like 'sqlite_%';";
 */
/* <-- Const */
/* --> Functions */

pub fn entry_point() -> Result<(), String> {
    let mut f: FltkHost = FltkHost::new();

    f.construct();

    match f.event_loop() {
        Ok(_) => { Ok(()) },
        Err(E) => { Err(E) },
    }
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
    request: QueryType,
    db_name: &str,
    db_interface: Option<&ConnectionBase>,
) -> Result<RecordSet, String> {
    let result = match db_interface {
        Some(&ConnectionBase::Odbc) => { crate::odbc_interface::entry_point(String::from(db_name), request).unwrap() },
        Some(&ConnectionBase::Sqlite) => { crate::sqlite3_interface::raw_query(String::from(db_name), request).unwrap() },
        None => { return Err(String::from("Oops")) },
    };
    Ok(result)
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
    mut table: &mut SmartTable,
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



/* <-- Functions */
