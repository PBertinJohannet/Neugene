use crate::graphics::map::{Map, MapTrait, WIDTH, HEIGHT, zerotable, randomtable};

use gtk::prelude::*;
use gtk::{Window, WindowType, DrawingArea, Button, ScrolledWindow};
use std::thread;
use std::sync::{Arc, Mutex};

const CELL_SIZE: i32 = 4;

struct Shared {
    map : Arc<Mutex<Map>>,
    state : Arc<Mutex<State>>,
    cell_size : Arc<Mutex<i32>>,
}

impl Shared {
    pub fn new () -> Self {
        Shared {
            map : Arc::new(Mutex::new(zerotable())),
            state : Arc::new(Mutex::new(State{
                        run: false,
                        repaint: false,
                        exited: false,
                        next: false,
                     })),
            cell_size : Arc::new(Mutex::new(CELL_SIZE)),
        }
    }
}

// `State` is used for handling events
struct State{
    run: bool,
    repaint: bool,
    exited: bool,
    next: bool,
}

// DrawingArea is wrapped in Area, look here:
// http://stackoverflow.com/questions/25413201/how-do-i-implement-a-trait-i-dont-own-for-a-type-i-dont-own-in-rust
#[derive(Clone)]
struct Area(DrawingArea);

unsafe impl Send for Area {
}

pub struct App {
    shared : Shared,
    view : View,
}

impl App {
    pub fn new() -> Result<Self, String>{
        let shared = Shared::new();
        let view = View::create()?;
        Ok(App {
            shared,
            view,
        })
    }

    pub fn start(&mut self) {
        self.view.pack_all();
        self.connect_randomize();
        self.connect_clear();
        self.connect_close();
        self.connect_draw();
        self.connect_next();
        self.connect_pause();
        self.connect_zoom_in();
        self.connect_zoom_out();
        self.run();
    }

    fn connect_draw(&mut self) {
        // also update the map each time we re-draw
        let map = self.shared.map.clone();
        let state = self.shared.state.clone();
        let cell_size = self.shared.cell_size.clone();
        self.view.area.0.connect_draw( move |this, cr| {
            let mut map = map.lock().unwrap();
            {
                let mut state = state.lock().unwrap();
                if state.run {
                    map.update();
                }else if state.next {
                    map.update();
                    state.next = false;
                }
            }
            (|x: i32| {
                this.set_size_request(WIDTH*x, HEIGHT*x);
                cr.scale(x as f64, x as f64);
            }) (*cell_size.lock().unwrap());
            cr.set_source_rgb(1f64, 1f64, 1f64);
            cr.paint();
            cr.set_source_rgb(0f64, 0f64, 0f64);
            for i in 0..WIDTH as usize { for j in 0..HEIGHT as usize {
                if map[i][j] == 1 {
                    cr.rectangle(i as f64, j as f64, 1.0, 1.0);
                }
            }}
            cr.fill();
            Inhibit(true)
        });
    }

    fn spawn_main_thread(&mut self) -> thread::JoinHandle<()>{
            let state = self.shared.state.clone();
            let area = self.view.area.clone();

            thread::spawn(move || {
                let duration = std::time::Duration::from_millis(50);
                loop {
                    thread::sleep(duration);
                    let mut state = state.lock().unwrap();
                    if state.exited {
                        break;
                    }
                    if state.run {
                        // `queue_draw` will ask gtk to
                        // repaint the widget
                        area.0.queue_draw();
                    } else if state.repaint {
                        state.repaint = false;
                        area.0.queue_draw();
                    }
                }
            })
    }

    pub fn connect_randomize(&mut self){
        let map = self.shared.map.clone();
        let state = self.shared.state.clone();
        self.view.random_button.connect_clicked( move |_| {
            map.lock().unwrap().copy_from(&randomtable());
            state.lock().unwrap().repaint = true;
        });
    }

    pub fn connect_next(&mut self){
        let state = self.shared.state.clone();
        self.view.next_button.connect_clicked( move |_| {
            let mut state = state.lock().unwrap();
            if !state.run {
                state.next = true;
                state.repaint = true;
            } else {
                state.next = false;
            }
        });
    }

    pub fn connect_zoom_in(&mut self) {
        let state = self.shared.state.clone();
        let cell_size = self.shared.cell_size.clone();
        self.view.zoom_in_button.connect_clicked( move |_| {
            *cell_size.lock().unwrap() += 1;
            state.lock().unwrap().repaint = true;
        });
    }

    pub fn connect_zoom_out(&mut self){
        let state = self.shared.state.clone();
        let cell_size = self.shared.cell_size.clone();
        self.view.zoom_out_button.connect_clicked( move |_| {
            let mut x = cell_size.lock().unwrap();
            if *x > CELL_SIZE {
                *x -= 1;
            }
            state.lock().unwrap().repaint = true;
        });
    }

    pub fn connect_pause (&mut self){
        let state = self.shared.state.clone();
        self.view.pause_button.connect_clicked( move |button| {
            let mut state = state.lock().unwrap();
            state.run = !state.run;
            button.set_label(
                if state.run { "Pause" }
                    else { "Start" }
            );
        });
    }

    pub fn connect_clear(&mut self){
            let map = self.shared.map.clone();
            let state = self.shared.state.clone();
            self.view.clear_button.connect_clicked( move |_| {
                map.lock().unwrap().copy_from(&zerotable());
                state.lock().unwrap().repaint = true;
            });
    }


    pub fn connect_close(&mut self){
        {
            let state = self.shared.state.clone();
            self.view.window.connect_delete_event(move |_, _| {
                let mut state = state.lock().unwrap();
                state.exited = true;
                gtk::main_quit();
                Inhibit(true)
            });
        }
    }

    fn run(&mut self){
        let main_thread = self.spawn_main_thread();
        gtk::main();
        // wait for the thread to stop
        match main_thread.join() {
            Err(_) => println!("Some error ocured"),
            Ok(_) => (),
        }
    }
}

struct View {
    window : Window,
    hbox : gtk::Box,
    button_box : gtk::ButtonBox,
    pause_button : Button,
    random_button : Button,
    next_button : Button,
    clear_button : Button,
    zoom_in_button : Button,
    zoom_out_button : Button,
    area : Area,
    scroller : ScrolledWindow,
}

impl View {
    pub fn create() -> Result<Self, String>{
        gtk::init().map_err(|e| String::from("gtk::init failed"))?;
        let window = Window::new(WindowType::Toplevel);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let button_box = gtk::ButtonBox::new(gtk::Orientation::Vertical);
        let pause_button = Button::new_with_label("Start");
        let random_button = Button::new_with_label("Randomize");
        let next_button = Button::new_with_label("Next");
        let clear_button = Button::new_with_label("Clear");
        let zoom_in_button = Button::new_with_label("Zoom in");
        let zoom_out_button = Button::new_with_label("Zoom out");
        let area = Area(DrawingArea::new());
        let scroller = ScrolledWindow::new(None, None);
        Ok(View {
            window,
            hbox,
            button_box,
            pause_button,
            random_button,
            next_button,
            clear_button,
            zoom_in_button,
            zoom_out_button,
            area,
            scroller,
        })
    }

    pub fn pack_all(&mut self) {
        self.scroller.set_size_request(WIDTH*CELL_SIZE, HEIGHT*CELL_SIZE);
        // disable auto-hide scrollbar
        self.scroller.set_overlay_scrolling(false);

        self.button_box.set_layout(gtk::ButtonBoxStyle::Start);
        self.button_box.pack_start(&self.pause_button, false, false, 0);
        self.button_box.pack_start(&self.next_button, false, false, 0);
        self.button_box.pack_start(&self.random_button, false, false, 0);
        self.button_box.pack_start(&self.clear_button, false, false, 0);
        self.button_box.pack_start(&self.zoom_in_button, false, false, 0);
        self.button_box.pack_start(&self.zoom_out_button, false, false, 0);
        self.scroller.add(&self.area.0);
        self.hbox.pack_start(&self.scroller, false, false, 0);
        self.hbox.pack_start(&self.button_box, false, false, 0);
        self.window.add(&self.hbox);
        self.window.set_title("Game of Life");
        self.window.show_all();
    }
}