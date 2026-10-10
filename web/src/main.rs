//! Browser front end. The game prints onto green-bar paper, keeping the
//! original pacing: the game's pauses become timers between printed text.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{self, Write};
use std::rc::Rc;
use std::time::Duration;

use coffee_maker_tables::{Game, Pacer};
use gloo_timers::future::sleep;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, HtmlButtonElement, HtmlElement, HtmlInputElement, KeyboardEvent};

/// Something the game produced: text to print, or a pause before the rest.
enum Event {
    Text(Vec<u8>),
    Pause(Duration),
}

/// Collects the game's output and pauses, in order, for the page to play
/// back. The game writes to one clone and paces through another.
#[derive(Clone, Default)]
struct Feed(Rc<RefCell<Vec<Event>>>);

impl Feed {
    fn take(&self) -> Vec<Event> {
        std::mem::take(&mut self.0.borrow_mut())
    }
}

impl Write for Feed {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut events = self.0.borrow_mut();
        match events.last_mut() {
            Some(Event::Text(text)) => text.extend_from_slice(buf),
            _ => events.push(Event::Text(buf.to_vec())),
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Pacer for Feed {
    fn pause(&mut self, d: Duration) {
        self.0.borrow_mut().push(Event::Pause(d));
    }
}

/// The elements the page works with.
#[derive(Clone)]
struct Page {
    document: Document,
    paper: HtmlElement,
    out: HtmlElement,
    input: HtmlInputElement,
    again: HtmlButtonElement,
}

impl Page {
    fn find() -> Page {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .expect("no document");
        let get = |id: &str| {
            document
                .get_element_by_id(id)
                .unwrap_or_else(|| panic!("missing #{id}"))
        };
        Page {
            paper: get("paper").unchecked_into(),
            out: get("out").unchecked_into(),
            input: get("cmd").unchecked_into(),
            again: get("again").unchecked_into(),
            document,
        }
    }

    /// Print text at the print head, just before the input.
    fn print(&self, text: &str) {
        let node = self.document.create_text_node(text);
        self.out
            .insert_before(&node, Some(&self.input))
            .expect("insert text");
        self.scroll_to_end();
    }

    /// Print what the player typed, in red, as a terminal would echo it.
    fn echo(&self, line: &str) {
        let span = self.document.create_element("span").expect("span");
        span.set_class_name("typed");
        span.set_text_content(Some(line));
        self.out
            .insert_before(&span, Some(&self.input))
            .expect("insert echo");
        self.print("\n");
    }

    fn clear(&self) {
        while let Some(child) = self.out.first_child() {
            if child.is_same_node(Some(&self.input)) {
                break;
            }
            self.out.remove_child(&child).expect("remove child");
        }
    }

    fn scroll_to_end(&self) {
        self.paper.set_scroll_top(self.paper.scroll_height());
    }
}

struct App {
    page: Page,
    feed: Feed,
    game: Game<Feed, Feed>,
    /// Output is being printed; commands entered meanwhile wait their turn.
    printing: bool,
    /// Lines entered while printing, oldest first, like a terminal's
    /// type-ahead buffer.
    typed_ahead: VecDeque<String>,
}

type Shared = Rc<RefCell<App>>;

fn new_game(app: &Shared) {
    {
        let mut a = app.borrow_mut();
        a.page.clear();
        a.page.again.set_hidden(true);
        a.page.input.set_hidden(false);
        a.typed_ahead.clear();
        a.feed.take();
        a.game = Game::new(a.feed.clone(), a.feed.clone()).with_numbered_prompts();
        a.game.start_without_banner().expect("Feed never fails");
    }
    spawn_local(play(app.clone()));
}

/// Echo a line and hand it to the game.
fn run_line(app: &Shared, line: &str) {
    let mut a = app.borrow_mut();
    a.page.echo(line);
    a.game.handle_line(line).expect("Feed never fails");
}

/// Play back everything the game has produced, running any commands typed
/// ahead in the meantime, then hand control back to the player.
async fn play(app: Shared) {
    let page = app.borrow().page.clone();
    loop {
        let events = {
            let mut a = app.borrow_mut();
            a.printing = true;
            a.feed.take()
        };
        for event in events {
            match event {
                Event::Text(bytes) => page.print(&String::from_utf8_lossy(&bytes)),
                Event::Pause(d) => sleep(d).await,
            }
        }
        let next = {
            let mut a = app.borrow_mut();
            a.printing = false;
            if a.game.is_finished() {
                a.typed_ahead.clear();
            }
            a.typed_ahead.pop_front()
        };
        match next {
            Some(line) => run_line(&app, &line),
            None => break,
        }
    }
    if app.borrow().game.is_finished() {
        page.input.set_hidden(true);
        page.again.set_hidden(false);
        page.again.focus().ok();
    } else {
        page.input.focus().ok();
    }
    page.scroll_to_end();
}

fn submit(app: &Shared) {
    let line = {
        let mut a = app.borrow_mut();
        if a.game.is_finished() {
            return;
        }
        let line = a.page.input.value();
        a.page.input.set_value("");
        if a.printing {
            a.typed_ahead.push_back(line);
            return;
        }
        line
    };
    run_line(app, &line);
    spawn_local(play(app.clone()));
}

fn main() {
    let feed = Feed::default();
    let page = Page::find();
    let app: Shared = Rc::new(RefCell::new(App {
        game: Game::new(feed.clone(), feed.clone()),
        page: page.clone(),
        feed,
        printing: false,
        typed_ahead: VecDeque::new(),
    }));

    let on_key = {
        let app = app.clone();
        Closure::<dyn FnMut(KeyboardEvent)>::new(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                e.prevent_default();
                submit(&app);
            }
        })
    };
    page.input
        .add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref())
        .expect("keydown listener");
    on_key.forget();

    // Clicking anywhere on the paper puts the cursor back at the prompt.
    let on_paper_click = {
        let input = page.input.clone();
        Closure::<dyn FnMut()>::new(move || {
            if !input.hidden() {
                input.focus().ok();
            }
        })
    };
    page.paper
        .add_event_listener_with_callback("click", on_paper_click.as_ref().unchecked_ref())
        .expect("click listener");
    on_paper_click.forget();

    let on_again = {
        let app = app.clone();
        Closure::<dyn FnMut()>::new(move || new_game(&app))
    };
    page.again
        .add_event_listener_with_callback("click", on_again.as_ref().unchecked_ref())
        .expect("click listener");
    on_again.forget();

    new_game(&app);
}
