use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, TreeView,
	Button, CellRendererText, Label, ListStore,
	Orientation, TreeViewColumn, WindowPosition,
	gio, glib};

pub fn entry_point() -> gtk::Application {
/*
    let app = Application::builder()
        .application_id("com.init.DaediVictus")
        .build();

    app.connect_activate(|app| {
        // We create the main window.
        let win = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .title("SQLite Query Tool!")
            .build();

				let tv = TreeView::new();
				win.add(&tv);

        // Don't forget to make all widgets visible.
        win.show_all();
    });

    app.run();
*/
	let application = init();
	application
}

pub fn create_and_fill_model() -> ListStore {
	let model = ListStore::new(&[u32::static_type(), String::static_type()]);
	let entries = &["Michel", "Sara", "Liam", "Zelda", "Neo", "Octopus Master"];
	for (i, entry) in entries.iter().enumerate() {
		model.insert_with_values(None, &[(0, &(i as u32 + 1)), (1, &entry)]);
	}
	model
}

pub fn add_column_data(tree: &TreeView, id: i32) {
	let column = TreeViewColumn::new();
	let cell = CellRendererText::new();

	TreeViewColumnExt::pack_start(&column, &cell, true);
	TreeViewColumnExt::add_attribute(&column, &cell, "text", id);
	tree.append_column(&column);
}

pub fn remove_column(tree: &TreeView, column: &TreeViewColumn) {
	tree.remove_column(&column);
}

// add # of columns
fn add_columns_to_tree(tree: TreeView, column_count: u32) -> TreeView {
	tree.set_headers_visible(true);
	let mut i = 0;

	while i < column_count {
		append_column(&tree, i);
		i = i + 1;
	}
	tree
}

fn init() -> gtk::Application {
	let application = gtk::Application::new(
		Some("com.DaedriVictus.Sqlite.UI"),
		Default::default(),
	);

	let window = ApplicationWindow::new(application);

	window.set_title("SQLite Query Tool");
	window.set_position(WindowPosition::Center);

	application.run();
	application
}


pub fn rebuild_trees(application: &gtk::Application, record_set: String) {
	let vertical_layout = gtk::Box::new(Orientation::Vertical, 0);
	let label = Label::new(None);
	let tree = create_and_setup_view(&record_set);
	let model = create_and_fill_model(&record_set);

	tree.set_model(Some(&model));
	vertical_layout.add(&tree);
	vertical_layout.add(&label);

	tree.connect_cursor_changed(move |tree_view| {
		let selection = tree_view.selection();
		if let Some((model, iter)) = selection.selected() {
			label.set_text(&format!(
				"Hello '{}' from row {}",
				model
					.value(&iter, 1)
					.get::<String>()
					.expect("Treeview selection, column 1"),
				model
					.value(&iter, 0)
					.get::<u32>()
					.expect("Treeview selection, column 0"),
			));
		}
	});

	window.add(&vertical_layout);

	window.show_all();
}
