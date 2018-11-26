use cairo::Context;
use crate::graphics::SingleStepDrawable;
use crate::graphics::{learn_back, DrawInstruction, Entity, ToDraw};
use crate::params::*;
use crate::problems;
use gtk::prelude::*;
use gtk::{Button, DrawingArea, ScrolledWindow, Window, WindowType};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::thread;

struct Shared {
    current_frames: Arc<Mutex<ToDraw>>,
    next_frames: Arc<Mutex<ToDraw>>,
    state: Arc<Mutex<State>>,
    world_size: Arc<Mutex<[usize; 2]>>,
}

impl Shared {
    pub fn new() -> Self {
        Shared {
            current_frames: Arc::new(Mutex::new(ToDraw(Vec::new()))),
            next_frames: Arc::new(Mutex::new(ToDraw(Vec::new()))),
            state: Arc::new(Mutex::new(State {
                run: true,
                repaint: false,
                exited: false,
            })),
            world_size: Arc::new(Mutex::new([1, 1])),
        }
    }
}

// `State` is used for handling events
struct State {
    run: bool,
    repaint: bool,
    exited: bool,
}

// DrawingArea is wrapped in Area, look here:
// http://stackoverflow.com/questions/25413201/how-do-i-implement-a-trait-i-dont-own-for-a-type-i-dont-own-in-rust
#[derive(Clone)]
struct Area(DrawingArea);

unsafe impl Send for Area {}

pub struct App<T: SingleStepDrawable + Clone> {
    shared: Shared,
    view: View,
    phantom: PhantomData<T>,
}

impl<T: SingleStepDrawable + Clone> App<T>
where
    <T as problems::SingleStepProblem>::Sol: std::clone::Clone,
    <T as problems::GenericProblem>::ProblemConfig: 'static,
{
    pub fn new() -> Result<Self, String> {
        let shared = Shared::new();
        let view = View::create()?;
        Ok(App {
            shared,
            view,
            phantom: PhantomData,
        })
    }

    pub fn start(&mut self, conf: T::ProblemConfig) {
        self.view.pack_all();
        self.connect_close();
        self.connect_draw();
        self.run(conf);
    }

    /// Draws the given entities on the area using the given world size.
    fn draw_entities(
        _this: &DrawingArea,
        cr: &Context,
        frame: Vec<Entity>,
        world_size: &[usize; 2],
    ) -> Inhibit {
        //println!("draw instruction entities! {}", frame.len());
        cr.set_source_rgb(1f64, 1f64, 1f64);
        cr.paint();
        let (coef_x, coef_y) = (
            (WIDTH as f64) / (world_size[0] as f64),
            (HEIGHT as f64) / (world_size[1] as f64),
        );
        for [x, y, size_x, size_y, red, green, blue] in frame {
            cr.set_source_rgb(red, green, blue);
            cr.rectangle(coef_x * x, coef_y * y, coef_x * size_x, coef_y * size_y);
            /*println!(
                "draw ent : {}, {}, {}, {}, {}, {}, {}",
                coef_x * x,
                coef_y * y,
                coef_x * size_x,
                coef_y * size_y,
                red,
                green,
                blue
            );
            println!("draw orig : {}, {}, {}, {}", x, y, size_x, size_y);*/
            cr.fill();
        }
        Inhibit(true)
    }

    fn draw_instruction(
        this: &DrawingArea,
        cr: &Context,
        current: &mut ToDraw,
        next: &mut ToDraw,
        world_size: &mut [usize; 2],
    ) -> Inhibit {
        let front = current.0.pop();
        match front {
            Some(DrawInstruction::WorldSize([x, y])) => {
                *world_size = [x, y];
                Self::draw_instruction(this, cr, current, next, world_size)
            }
            Some(DrawInstruction::Frame(frame)) => Self::draw_entities(this, cr, frame, world_size),
            None => {
                current.0 = next.0.clone().into_iter().rev().collect();
                if current.0.len() > 0 {
                    Self::draw_instruction(this, cr, current, next, world_size)
                } else {
                    Inhibit(true)
                }
            }
        }
    }

    fn connect_draw(&mut self) {
        // also update the map each time we re-draw
        let current_frames = self.shared.current_frames.clone();
        let next_frames = self.shared.next_frames.clone();
        let world_size = self.shared.world_size.clone();
        self.view.area.0.connect_draw(move |this, cr| {
            let mut current = current_frames.lock().unwrap();
            let mut next = next_frames.lock().unwrap();
            let mut world = world_size.lock().unwrap();
            this.set_size_request(WIDTH, HEIGHT);
            Self::draw_instruction(this, cr, &mut current, &mut next, &mut world)
        });
    }

    fn spawn_main_thread(&mut self) -> thread::JoinHandle<()> {
        let state = self.shared.state.clone();
        let area = self.view.area.clone();

        thread::spawn(move || {
            let duration = std::time::Duration::from_millis(200);
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

    pub fn spawn_learning_thread(&mut self, conf: T::ProblemConfig) -> thread::JoinHandle<()> {
        let next = self.shared.next_frames.clone();
        let arc_conf = Arc::new(Mutex::new(conf));
        thread::spawn(move || learn_back::<T>(next, arc_conf))
    }

    pub fn connect_close(&mut self) {
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

    fn run(&mut self, conf: T::ProblemConfig) {
        let main_thread = self.spawn_main_thread();
        let _learning_thread = self.spawn_learning_thread(conf);
        gtk::main();
        // wait for the thread to stop
        match main_thread.join() {
            Err(_) => println!("Some error ocured"),
            Ok(_) => (),
        }
    }
}

struct View {
    window: Window,
    hbox: gtk::Box,
    button_box: gtk::ButtonBox,
    pause_button: gtk::Button,
    area: Area,
    scroller: ScrolledWindow,
}

impl View {
    pub fn create() -> Result<Self, String> {
        gtk::init().map_err(|_| String::from("gtk::init failed"))?;
        let window = Window::new(WindowType::Toplevel);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let button_box = gtk::ButtonBox::new(gtk::Orientation::Vertical);
        let pause_button = Button::new_with_label("Start");
        let area = Area(DrawingArea::new());
        let scroller = ScrolledWindow::new(None, None);
        Ok(View {
            window,
            hbox,
            button_box,
            pause_button,
            area,
            scroller,
        })
    }

    pub fn pack_all(&mut self) {
        println!("pack all");
        self.scroller.set_size_request(WIDTH, HEIGHT);
        // disable auto-hide scrollbar
        self.scroller.set_overlay_scrolling(false);

        self.button_box.set_layout(gtk::ButtonBoxStyle::Start);
        self.button_box
            .pack_start(&self.pause_button, false, false, 0);
        self.scroller.add(&self.area.0);
        self.hbox.pack_start(&self.scroller, false, false, 0);
        self.hbox.pack_start(&self.button_box, false, false, 0);
        self.window.add(&self.hbox);
        self.window.set_title("Neugene");
        self.window.show_all();
    }
}
