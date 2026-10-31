//! Scrolling with a wheel, a trackpad or a finger. Scrolling goes by pixels,
//! so the page moves as far as the hand does, a line at a time once a line's
//! worth adds up. Sideways, the wide code block under the pointer, or where
//! a finger first touched, moves a column at a time the same way.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use nixpulvis::App;
use wasm_bindgen::prelude::*;
use web_sys::{AddEventListenerOptions, Element, EventTarget, TouchEvent, WheelEvent, Window};

use crate::Shared;

/// Slowest a flick glides, in pixels a millisecond.
const MIN_GLIDE: f64 = 0.05;

/// How far a finger goes before its touch picks a way to move, in pixels.
const LOCK_DISTANCE: i32 = 6;

/// How long between wheel events before the next starts a new gesture, in
/// milliseconds.
const WHEEL_GESTURE: f64 = 200.0;

/// What wheels and fingers move, for the view of an app drawn in an
/// element.
pub(crate) struct Gestures {
    /// Down the page.
    scroll: Rc<Mover>,
    /// The wide code block at the pointer, sideways.
    shift: Rc<Mover>,
    /// Where the pointer, or the finger, last was in the window, in pixels.
    pointer: Rc<Cell<(f64, f64)>>,
    /// The element the view is drawn in.
    screen: Element,
    /// How wide and tall the view's cells are, in pixels.
    cell: (f64, f64),
}

impl Gestures {
    /// Starts following the wheel and touches for the view of `app` drawn
    /// in `screen`, with cells `cell` pixels wide and tall.
    pub(crate) fn listen(app: &Shared, screen: &Element, cell: (f64, f64)) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("no window")?;
        let pointer = Rc::new(Cell::new((0.0, 0.0)));
        let shift = Mover::new(app, cell.0, {
            let (pointer, screen) = (pointer.clone(), screen.clone());
            move |app: &mut App, columns| {
                let (x, y) = cell_at(&screen, cell, pointer.get());
                app.shift_at(x, y, columns)
            }
        });
        let gestures = Rc::new(Self {
            scroll: Mover::new(app, cell.1, App::scroll_by),
            shift,
            pointer,
            screen: screen.clone(),
            cell,
        });
        gestures.listen_wheel(&window)?;
        gestures.listen_touches(&window)
    }

    fn along(&self, axis: Axis) -> Rc<Mover> {
        match axis {
            Axis::Down => self.scroll.clone(),
            Axis::Sideways => self.shift.clone(),
        }
    }

    /// A wheel gesture, like a swipe on a trackpad, goes the way its first
    /// event mostly goes.
    fn listen_wheel(self: &Rc<Self>, window: &Window) -> Result<(), JsValue> {
        let gestures = self.clone();
        let mut gesture: Option<(Axis, f64)> = None;
        let on_wheel = Closure::<dyn FnMut(WheelEvent)>::new(move |e: WheelEvent| {
            e.prevent_default();
            gestures
                .pointer
                .set((f64::from(e.client_x()), f64::from(e.client_y())));
            let (x, y) = gestures.wheel_pixels(&e);
            let axis = match gesture {
                Some((axis, at)) if e.time_stamp() - at < WHEEL_GESTURE => axis,
                _ => {
                    let axis = Axis::of(x, y);
                    gestures.along(axis).start();
                    axis
                }
            };
            // Once it runs into the end, like a sideways flick's momentum at
            // the end of the code, the next event is free to go the other
            // way.
            let moved = gestures.along(axis).by(axis.part(x, y));
            gesture = moved.then_some((axis, e.time_stamp()));
        });
        window.add_event_listener_with_callback_and_add_event_listener_options(
            "wheel",
            on_wheel.as_ref().unchecked_ref(),
            &not_passive(),
        )?;
        on_wheel.forget();
        Ok(())
    }

    /// How far a wheel event goes, sideways and down, in pixels.
    fn wheel_pixels(&self, e: &WheelEvent) -> (f64, f64) {
        let pixels = |delta: f64| match e.delta_mode() {
            WheelEvent::DOM_DELTA_LINE => delta * self.cell.1,
            WheelEvent::DOM_DELTA_PAGE => delta * f64::from(self.screen.client_height()),
            _ => delta,
        };
        (pixels(e.delta_x()), pixels(e.delta_y()))
    }

    /// A finger drags the page along, and a flick keeps it gliding for a
    /// bit, slowing down, like a page in the browser does.
    fn listen_touches(self: &Rc<Self>, window: &Window) -> Result<(), JsValue> {
        let touch = Rc::new(RefCell::new(Touch::default()));

        let (gestures, moving) = (self.clone(), touch.clone());
        let on_move = Closure::<dyn FnMut(TouchEvent)>::new(move |e: TouchEvent| {
            e.prevent_default();
            let Some(at) = first_touch(&e) else { return };
            let Some(step) = moving.borrow_mut().moved(at, e.time_stamp()) else {
                return;
            };
            let mover = gestures.along(step.axis);
            if step.first {
                mover.start();
            }
            let _ = mover.by(step.pixels);
        })
        .into_js_value();

        let (gestures, ending) = (self.clone(), touch.clone());
        let on_end = Closure::<dyn FnMut(TouchEvent)>::new(move |_: TouchEvent| {
            if let Some(axis) = ending.borrow_mut().end() {
                glide(ending.clone(), gestures.along(axis), None);
            }
        })
        .into_js_value();

        // A touch's events all go to the element it started on, even once a
        // redraw has taken that element out of the page, when they no
        // longer bubble up to the window. So the rest of the touch is
        // followed on the element itself, the innermost one, inside the
        // view's shadow root.
        let gestures = self.clone();
        let on_start = Closure::<dyn FnMut(TouchEvent)>::new(move |e: TouchEvent| {
            let at = first_touch(&e);
            if let Some((x, y)) = at {
                gestures.pointer.set((f64::from(x), f64::from(y)));
            }
            touch.borrow_mut().begin(at, e.time_stamp());
            let Ok(target) = e.composed_path().get(0).dyn_into::<EventTarget>() else {
                return;
            };
            let _ = target.add_event_listener_with_callback_and_add_event_listener_options(
                "touchmove",
                on_move.unchecked_ref(),
                &not_passive(),
            );
            for kind in ["touchend", "touchcancel"] {
                let _ = target.add_event_listener_with_callback(kind, on_end.unchecked_ref());
            }
        });
        window.add_event_listener_with_callback("touchstart", on_start.as_ref().unchecked_ref())?;
        on_start.forget();
        Ok(())
    }
}

/// The way a gesture moves the view, down the page or wide code sideways.
/// A gesture only ever moves one way.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Axis {
    Down,
    Sideways,
}

impl Axis {
    /// The way a move of `x`, `y` pixels mostly goes.
    fn of(x: f64, y: f64) -> Self {
        if x.abs() > y.abs() {
            Axis::Sideways
        } else {
            Axis::Down
        }
    }

    /// The part of a move of `x`, `y` pixels that goes this way.
    fn part(self, x: f64, y: f64) -> f64 {
        match self {
            Axis::Down => y,
            Axis::Sideways => x,
        }
    }
}

/// Moves the view a whole cell at a time, `cell` pixels each, as far as a
/// gesture goes, keeping the part of a cell left over for the gesture's
/// next event.
struct Mover {
    app: Shared,
    cell: f64,
    step: Step,
    carried: Cell<f64>,
}

/// Takes a mover's steps, and says whether they moved, which they don't at
/// the end.
type Step = Box<dyn Fn(&mut App, isize) -> bool>;

impl Mover {
    fn new(app: &Shared, cell: f64, step: impl Fn(&mut App, isize) -> bool + 'static) -> Rc<Self> {
        Rc::new(Self {
            app: app.clone(),
            cell,
            step: Box::new(step),
            carried: Cell::new(0.0),
        })
    }

    /// Starts a gesture, dropping what the last one left over, so this one
    /// doesn't start part of a cell in.
    fn start(&self) {
        self.carried.set(0.0);
    }

    /// Moves `pixels` further, and says whether the view moved, which it
    /// doesn't at the end.
    fn by(&self, pixels: f64) -> bool {
        let total = self.carried.get() + pixels;
        let steps = (total / self.cell).trunc();
        self.carried.set(total - steps * self.cell);
        steps == 0.0 || (self.step)(&mut self.app.borrow_mut(), steps as isize)
    }
}

/// A finger on the screen, and how fast it was going.
#[derive(Default)]
struct Touch {
    /// Where the finger came down, and where it was last, in pixels.
    start: Option<(i32, i32)>,
    last: Option<(i32, i32)>,
    /// The way this touch moves the view, once it's gone far enough to tell.
    axis: Option<Axis>,
    /// When the last touch event came, in milliseconds.
    at: f64,
    /// In pixels a millisecond, down or right when positive.
    speed: f64,
    gliding: bool,
}

/// How far a finger's move takes the view.
struct TouchStep {
    axis: Axis,
    pixels: f64,
    /// Whether it's the touch's first, when it picked its way.
    first: bool,
}

impl Touch {
    /// A finger coming down at `at`, catching any glide.
    fn begin(&mut self, at: Option<(i32, i32)>, time: f64) {
        *self = Self {
            start: at,
            last: at,
            at: time,
            ..Self::default()
        };
    }

    /// The finger moving to `now`. Nothing until it's gone far enough to
    /// pick which way it moves the view.
    fn moved(&mut self, now: (i32, i32), time: f64) -> Option<TouchStep> {
        let (start, last) = (self.start?, self.last?);
        let first = self.axis.is_none();
        let axis = match self.axis {
            Some(axis) => axis,
            None => {
                let (x, y) = (now.0 - start.0, now.1 - start.1);
                if x.abs().max(y.abs()) < LOCK_DISTANCE {
                    return None;
                }
                *self.axis.insert(Axis::of(f64::from(x), f64::from(y)))
            }
        };
        self.last = Some(now);
        let pixels = axis.part(f64::from(last.0 - now.0), f64::from(last.1 - now.1));
        let ms = (time - self.at).max(1.0);
        // Smoothed, so one jittery event doesn't decide a flick.
        self.speed = 0.8 * pixels / ms + 0.2 * self.speed;
        self.at = time;
        Some(TouchStep {
            axis,
            pixels,
            first,
        })
    }

    /// The finger lifting. Returns the way to glide, when it was a flick.
    fn end(&mut self) -> Option<Axis> {
        self.start = None;
        self.last = None;
        let axis = self.axis.take().filter(|_| self.speed.abs() > MIN_GLIDE)?;
        self.gliding = true;
        Some(axis)
    }
}

/// Keeps the view going after a flick, a frame at a time, until it slows
/// to a stop, runs into the end, or another touch catches it.
fn glide(touch: Rc<RefCell<Touch>>, mover: Rc<Mover>, last: Option<f64>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let frame = Closure::once_into_js(move |now: f64| {
        let mut state = touch.borrow_mut();
        if !state.gliding {
            return;
        }
        let ms = last.map_or(16.0, |last| (now - last).clamp(1.0, 100.0));
        let pixels = state.speed * ms;
        // About 5% slower every 16 milliseconds.
        state.speed *= 0.95_f64.powf(ms / 16.0);
        state.gliding = state.speed.abs() > MIN_GLIDE;
        drop(state);
        if !mover.by(pixels) {
            touch.borrow_mut().gliding = false;
        }
        glide(touch, mover, Some(now));
    });
    let _ = window.request_animation_frame(frame.unchecked_ref());
}

/// Where the first finger of a touch is, in pixels in the window.
fn first_touch(e: &TouchEvent) -> Option<(i32, i32)> {
    e.touches().get(0).map(|t| (t.client_x(), t.client_y()))
}

/// The cell of the view drawn in `screen`, with cells `cell` pixels wide
/// and tall, at `x`, `y` pixels in the window.
fn cell_at(screen: &Element, cell: (f64, f64), (x, y): (f64, f64)) -> (u16, u16) {
    let rect = screen.get_bounding_client_rect();
    let at = |pixels: f64, size: f64| (pixels / size).max(0.0) as u16;
    (at(x - rect.left(), cell.0), at(y - rect.top(), cell.1))
}

/// Listener options that let the view take the event over from the
/// browser's own scrolling.
fn not_passive() -> AddEventListenerOptions {
    let options = AddEventListenerOptions::new();
    options.set_passive(false);
    options
}

#[cfg(test)]
mod tests {
    use nixpulvis::Host;

    use super::*;

    /// A mover with 10 pixel cells, and the steps it takes.
    fn mover() -> (Rc<Mover>, Rc<RefCell<Vec<isize>>>) {
        let app = Rc::new(RefCell::new(App::new(Host::Browser, "", "/")));
        let steps = Rc::new(RefCell::new(Vec::new()));
        let taken = steps.clone();
        let mover = Mover::new(&app, 10.0, move |_, n| {
            taken.borrow_mut().push(n);
            true
        });
        (mover, steps)
    }

    #[test]
    fn moves_a_cell_at_a_time() {
        let (mover, steps) = mover();
        for pixels in [4.0, 4.0, 25.0, -30.0] {
            mover.by(pixels);
        }
        // 8 pixels isn't a cell yet, 33 is 3 with 3 over, and -27 is -2.
        assert_eq!(*steps.borrow(), [3, -2]);
        // A new gesture starts from nothing.
        mover.start();
        mover.by(5.0);
        assert_eq!(*steps.borrow(), [3, -2]);
    }

    #[test]
    fn touches_move_one_way() {
        let mut touch = Touch::default();
        touch.begin(Some((100, 100)), 0.0);
        // Too short a move to tell which way.
        assert!(touch.moved((103, 102), 10.0).is_none());
        let step = touch.moved((90, 104), 20.0).unwrap();
        assert_eq!(
            (step.axis, step.pixels, step.first),
            (Axis::Sideways, 10.0, true)
        );
        // Still sideways, though now it's mostly down.
        let step = touch.moved((88, 140), 30.0).unwrap();
        assert_eq!(
            (step.axis, step.pixels, step.first),
            (Axis::Sideways, 2.0, false)
        );
        // Held still before lifting, so it doesn't glide.
        touch.moved((88, 140), 500.0);
        assert_eq!(touch.end(), None);

        // A flick glides on the way it went.
        touch.begin(Some((100, 300)), 0.0);
        for (i, y) in [280, 240, 200].into_iter().enumerate() {
            touch.moved((100, y), 10.0 * (i + 1) as f64);
        }
        assert_eq!(touch.end(), Some(Axis::Down));
        assert!(touch.gliding);
    }
}
