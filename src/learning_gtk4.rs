/* --> Summary of contents


<-- Summary of contents */
/* --> imports */

use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Button, self, Orientation};
use std::cell::Cell;
use std::rc::Rc;
use gtk::subclass::prelude::*;
use glib::clone;

/* <--  imports */
/* -->  Constants */

const APP_ID: &str = "org.example.HelloWorld";

/* <--  Constants */
/* -->  Functions */

pub fn entry_point() -> glib::ExitCode {
	let app = Application::builder()
		.application_id(APP_ID)
		.build();
	app.connect_activate(build_ui);	
	app.run()
}

fn build_ui(app: &Application) {
	let button_increase = Button::builder()
		.label("Increase")
		.margin_top(12)
		.margin_bottom(2)
		.margin_start(12)
		.margin_end(12)
		.build();
		
		let button_decrease = Button::builder()
		.label("Decrease")
		.margin_top(2)
		.margin_bottom(2)
		.margin_start(12)
		.margin_end(12)
		.build();
		
	let number = Rc::new(Cell::new(0));

	button_increase.connect_clicked(clone!(@weak number, @weak button_decrease => 
		move |_| { 
			number.set(number.get() + 1); 
			button_decrease.set_label(&number.get().to_string());
	} ));
	button_decrease.connect_clicked(clone!(@weak button_increase =>
		move |_| { 
			number.set(number.get() - 1);
			button_increase.set_label(&number.get().to_string());
		} 
	));

	let gtk_box = gtk::Box::builder()
		.orientation(Orientation::Vertical)
		.build();
	gtk_box.append(&button_increase);
	gtk_box.append(&button_decrease);

	let window = ApplicationWindow::builder()
		.application(app)
		.default_width(320)
		.default_height(200)
		.title("Hello World!")
		.child(&gtk_box)
		.build();
		
	window.present();
}

/* <--  Functions */
/* -->  mods */
mod imp {
	use gtk::glib;
	use gtk::subclass::prelude::*;
	
	#[derive(Default)]
	pub struct CustomButton;
	
	#[glib::object_subclass]
	impl ObjectSubclass for CustomButton {
		const NAME: &'static str = "MyGtkAppCustomButton";
		type Type = CustomButton;
		type ParentType = gtk::Button;
	}
	
	impl ObjectImpl for CustomButton {}
	impl WidgetImpl for CustomButton {}
	impl ButtonImpl for CustomButton {}
	
	mod imp_mod {
		use glib::Object;
		use gtk::glib;
		
		glib::wrapper! {
			pub struct CustomButton(ObjectSubclass<super::CustomButton>)
				@extends gtk::Button, gtk::Widget,
				@implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
		}
		
		impl super::CustomButton {
			pub fn new() -> Self {
				Object::builder().build()
			}
			
			pub fn with_label(label: &str) -> Self {
				Object::builder().property("label", label).build()
			}
		}
		
		impl Default for CustomButton {
			fn default() -> Self {
				Self::new()
			}
		}
	}
}
/* <--  mods */