use druid::{
    im::HashSet,
    widget::{prelude::*, Button, Controller, ControllerHost, Either, Flex, Label},
    AppLauncher, Data, LocalizedString, MouseButton, MouseButtons, MouseEvent, Widget, WidgetExt,
    WindowDesc,
};
use rand::{thread_rng, Rng};
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Data)]
struct Field {
    x: i32,
    y: i32,
}

#[derive(Clone, Data)]
struct AppState {
    revealed: HashSet<Field>,
    mines: HashSet<Field>,
    flagged: HashSet<Field>,
    width: i32,
    height: i32,
}

const WIDTH: i32 = 15;
const HEIGHT: i32 = 15;
const DIV: i32 = 10;

impl AppState {
    pub fn get_neighbors(&self, field: &Field) -> impl Iterator<Item = Field> {
        let width = self.width;
        let height = self.height;
        let field = *field;
        (-1..=1)
            .map(move |y| {
                (-1..=1).filter_map(move |x| {
                    let n = Field {
                        x: field.x + x,
                        y: field.y + y,
                    };
                    if x == 0 && y == 0 || !(0..width).contains(&n.x) || !(0..height).contains(&n.y)
                    {
                        None
                    } else {
                        Some(n)
                    }
                })
            })
            .flatten()
    }
    pub fn reveal_inc(&mut self, start: &Field) {
        // non-recursive reveal becomes necessary at about 200x200 fields
        // (which druid still can handle without too much drama :P)
        let mut stack = vec![*start];
        while !stack.is_empty() {
            let field = stack.pop().expect("stack is not empty");
            if self.revealed.contains(&field) || self.mines.contains(&field) {
                continue;
            }
            self.revealed.insert(field);
            if !self.get_neighbors(&field).any(|n| self.mines.contains(&n)) {
                stack.extend(self.get_neighbors(&field))
            }
        }
    }

    // pub fn reveal_rec(&mut self, field: &Field) {
    //     if self.revealed.contains(field) || self.mines.contains(field) {
    //         return;
    //     }
    //     self.revealed.insert(*field);
    //     if !self.get_neighbors(field).any(|n| self.mines.contains(&n)) {
    //         self.get_neighbors(field).for_each(|n| self.reveal_rec(&n));
    //     }
    // }

    pub fn count_mines(&self, field: &Field) -> usize {
        self.get_neighbors(field)
            .filter(|f| self.mines.contains(f))
            .count()
    }
}

pub fn main() {
    let main_window = WindowDesc::new(make_ui).title(LocalizedString::new("View Switcher"));

    let mut mines = HashSet::new();
    let mut rng = thread_rng();
    while mines.len() < (WIDTH * HEIGHT / DIV) as usize {
        let x: i32 = rng.gen_range(0, WIDTH);
        let y: i32 = rng.gen_range(0, HEIGHT);
        mines.insert(Field { x, y });
    }

    let data = AppState {
        revealed: HashSet::new(),
        mines,
        flagged: HashSet::new(),
        width: WIDTH,
        height: HEIGHT,
    };
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}

fn make_ui() -> impl Widget<AppState> {
    let mut col = Flex::column().main_axis_alignment(druid::widget::MainAxisAlignment::SpaceEvenly);

    let width = WIDTH;
    let height = HEIGHT;

    col.add_flex_child(
        Button::new("reset").on_click(|_ctx, data: &mut AppState, _env| {
            data.flagged.clear();
            data.revealed.clear();
            data.mines.clear();
            let mut rng = thread_rng();
            while data.mines.len() < (WIDTH * HEIGHT / DIV) as usize {
                let x: i32 = rng.gen_range(0, WIDTH);
                let y: i32 = rng.gen_range(0, HEIGHT);
                data.mines.insert(Field { x, y });
            }
        }),
        1.0,
    );

    for y in 0..height {
        let mut row = Flex::row().main_axis_alignment(druid::widget::MainAxisAlignment::Center); //.main_axis_alignment(druid::widget::MainAxisAlignment::SpaceEvenly);
        for x in 0..width {
            let field = Field { x, y };

            let button = Button::new(move |data: &AppState, _env: &Env| {
                if data.flagged.contains(&field) {
                    "F".to_string()
                } else {
                    "".to_string()
                }
            });
            let button = ControllerHost::new(
                button,
                AllClick::new(move |ctx, data: &mut AppState, _env, event| {
                    // if ctx.
                    let do_flag = event.button == MouseButton::Right
                        || (event.button == MouseButton::Left && event.mods.ctrl());
                    if !do_flag {
                        if !data.flagged.contains(&field) {
                            if data.mines.contains(&field) {
                                data.revealed = data.revealed.clone().union(data.mines.clone())
                            } else {
                                data.reveal_inc(&field);
                            }
                        }
                    } else {
                        if !data.flagged.contains(&field) {
                            data.flagged.insert(field);
                        } else {
                            data.flagged.remove(&field);
                        }
                    }
                    if !data
                        .revealed
                        .clone()
                        .intersection(data.mines.clone())
                        .is_empty()
                    {
                        println!("BOOOOMMMM");
                    } else if data.revealed.len() == (WIDTH * HEIGHT) as usize - data.mines.len() {
                        println!("win!!!1111eleven")
                    }
                }),
            );
            let either = Either::<AppState>::new(
                move |data, _env| data.revealed.contains(&field),
                Label::new(move |data: &AppState, _env: &Env| {
                    if data.mines.contains(&field) {
                        "X".to_string()
                    } else {
                        let c = data.count_mines(&field);
                        if c > 0 {
                            format!("{}", c)
                        } else {
                            "".to_string()
                        }
                    }
                })
                .center(),
                button,
            )
            .expand();

            row.add_flex_child(either, 1.0);
        }
        col.add_flex_child(row, 1.0)
    }
    col
}

pub struct AllClick<T> {
    /// A closure that will be invoked when the child widget is clicked.
    action: Box<dyn Fn(&mut EventCtx, &mut T, &Env, &MouseEvent)>,
}

impl<T: Data> AllClick<T> {
    /// Create a new clickable [`Controller`] widget.
    pub fn new(action: impl Fn(&mut EventCtx, &mut T, &Env, &MouseEvent) + 'static) -> Self {
        AllClick {
            action: Box::new(action),
        }
    }
}

impl<T: Data, W: Widget<T>> Controller<T, W> for AllClick<T> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_paint();
            }
            Event::MouseUp(mouse_event) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        (self.action)(ctx, data, env, mouse_event);
                    }
                    ctx.request_paint();
                }
            }
            _ => {}
        }

        child.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &T,
        env: &Env,
    ) {
        if let LifeCycle::HotChanged(_) | LifeCycle::FocusChanged(_) = event {
            ctx.request_paint();
        }

        child.lifecycle(ctx, event, data, env);
    }
}

#[test]
fn test_app_state() {
    let app_state = AppState {
        revealed: HashSet::new(),
        mines: HashSet::new(),
        flagged: HashSet::new(),
        width: 10,
        height: 10,
    };
    let field = Field { x: 5, y: 5 };

    let ns: Vec<Field> = app_state.get_neighbors(&field).collect();
    println!("ns: {:?}", ns);
}
