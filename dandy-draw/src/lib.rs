#[cfg(feature = "canvas")]
pub mod canvas;
#[cfg(feature = "egui")]
pub mod egui;
pub mod pos2;

use crate::pos2::{pos2, Pos2};
use dandy::dfa::{Dfa, DfaState};
use dandy::nfa::{Nfa, NfaState};
use paste::paste;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::mem;

pub trait Drawer {
    fn start_drawing(&mut self);
    fn finish_drawing(&mut self);
    fn draw_circle(&mut self, pos: Pos2, radius: f32, thickness: f32);
    fn draw_centered_text(&mut self, pos: Pos2, text: &str);
    fn draw_rect(&mut self, upper_left: Pos2, size: Pos2);
    fn draw_line(&mut self, from: Pos2, to: Pos2, thickness: f32);
    fn set_color(&mut self, _rgb: [u8; 3]) {}
}

struct OffsetScaleDrawer<'a, T> {
    offset: Pos2,
    scale: Pos2,
    drawer: &'a mut T,
}

impl<'a, T: Drawer> Drawer for OffsetScaleDrawer<'a, T> {
    fn start_drawing(&mut self) {
        self.drawer.start_drawing()
    }

    fn finish_drawing(&mut self) {
        self.drawer.finish_drawing()
    }

    fn draw_circle(&mut self, pos: Pos2, radius: f32, thickness: f32) {
        self.drawer.draw_circle(
            (pos + self.offset) * self.scale,
            radius * self.scale.x,
            thickness,
        )
    }

    fn draw_centered_text(&mut self, pos: Pos2, text: &str) {
        self.drawer
            .draw_centered_text((pos + self.offset) * self.scale, text)
    }

    fn draw_rect(&mut self, upper_left: Pos2, size: Pos2) {
        self.drawer
            .draw_rect((upper_left + self.offset) * self.scale, size)
    }

    fn draw_line(&mut self, from: Pos2, to: Pos2, thickness: f32) {
        self.drawer.draw_line(
            (from + self.offset) * self.scale,
            (to + self.offset) * self.scale,
            thickness,
        )
    }

    fn set_color(&mut self, rgb: [u8; 3]) {
        self.drawer.set_color(rgb)
    }
}

macro_rules! define_draw_options {
    ($name:ident {
        $($field:ident : $ty:ty = $def:expr,)*
    }) => {
        pub struct $name {
            $($field: $ty,)*
        }

        impl $name {
            pub fn new($($field: $ty,)*) -> Self {
                Self {
                    $($field,)*
                }
            }

            paste! {
                $(
                pub fn [< with_ $field >](mut self, val: $ty) -> Self {
                    self.$field = val;
                    self
                }
                )*
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new(
                    $($def,)*
                )
            }
        }
    }
}

define_draw_options! {
    DrawOptions {
        x_scale: f32 = 1.0,
        y_scale: f32 = 1.0,
        x_offset: f32 = 0.0,
        y_offset: f32 = 0.0,
        center_line_padding: f32 = 20.0,
        circle_radius: f32 = 30.0,
        circle_width: f32 = 2.0,
        accepting_circle_radius: f32 = 25.0,
        accepting_circle_width: f32 = 2.0,
        init_arrow_length: f32 = 50.0,
        init_arrow_arms_length: f32 = 15.0,
        init_arrow_width: f32 = 3.0,
        trans_arrow_arms_length: f32 = 5.0,
        trans_line_width: f32 = 3.0,
        from_line_offset: f32 = 15.0,
        to_line_offset: f32 = 15.0,
        floor_height: f32 = 25.0,
        text_margin: f32 = 15.0,
        line_circle_margin: f32 = 10.0,
        end_arrow: bool = true,
        middle_arrow: bool = true,
        text_color: [u8; 3] = [255, 255, 255],
        circle_color: [u8; 3] = [150, 255, 255],
        line_color: [u8; 3] = [0, 255, 255],
    }
}

pub fn draw_dfa(dfa: &Dfa, drawer: &mut impl Drawer) {
    draw_dfa_with_opts(dfa, drawer, DrawOptions::default())
}

pub fn draw_dfa_with_opts(dfa: &Dfa, drawer: &mut impl Drawer, opts: DrawOptions) {
    let states = dfa.states().iter().map(Into::into).collect::<Vec<State>>();
    let arrows = dfa_to_arrows(dfa);
    draw(states, arrows, drawer, opts)
}

pub fn draw_nfa(nfa: &Nfa, drawer: &mut impl Drawer) {
    draw_nfa_with_opts(nfa, drawer, DrawOptions::default())
}

pub fn draw_nfa_with_opts(nfa: &Nfa, drawer: &mut impl Drawer, opts: DrawOptions) {
    let states = nfa.states().iter().map(Into::into).collect::<Vec<State>>();
    let arrows = nfa_to_arrows(nfa);
    draw(states, arrows, drawer, opts)
}

fn draw<'a>(
    states: Vec<State<'a>>,
    arrows: Vec<Arrow<'a>>,
    drawer: &mut impl Drawer,
    opts: DrawOptions,
) {
    let offset = pos2(opts.x_offset, opts.y_offset);
    let scale = pos2(opts.x_scale, opts.y_scale);
    let mut drawer = OffsetScaleDrawer {
        offset,
        scale,
        drawer,
    };

    let arrows = group_arrows(arrows);
    let (arrows, levels) = place_arrows(arrows);

    let x_pos = |idx: usize| -> f32 {
        opts.init_arrow_length
            + opts.center_line_padding
            + opts.circle_radius
            + (opts.circle_radius * 2.0 + opts.center_line_padding) * idx as f32
    };

    let x_from_pos = |idx: usize| -> f32 { x_pos(idx) + opts.from_line_offset };

    let x_to_pos = |idx: usize| -> f32 { x_pos(idx) - opts.to_line_offset };

    let line_baseline = (levels + 1) as f32 * opts.floor_height;
    let circle_center = line_baseline + opts.line_circle_margin + opts.circle_radius;

    drawer.start_drawing();
    // draw arrow
    // FIXME: This assumes first is initial, which isn't always the case
    {
        drawer.set_color(opts.line_color);
        let arrow_base = pos2(opts.init_arrow_length, circle_center);
        drawer.draw_line(pos2(0.0, circle_center), arrow_base, opts.init_arrow_width);
        drawer.draw_line(
            pos2(opts.init_arrow_length, circle_center),
            arrow_base - pos2(opts.init_arrow_arms_length, opts.init_arrow_arms_length),
            opts.init_arrow_width,
        );
        drawer.draw_line(
            pos2(opts.init_arrow_length, circle_center),
            arrow_base - pos2(opts.init_arrow_arms_length, -opts.init_arrow_arms_length),
            opts.init_arrow_width,
        );
    }

    // draw states
    for (idx, state) in states.iter().enumerate() {
        let cc = pos2(x_pos(idx), circle_center);
        drawer.set_color(opts.circle_color);
        drawer.draw_circle(cc, opts.circle_radius, opts.circle_width);
        if state.accepting {
            drawer.draw_circle(
                cc,
                opts.accepting_circle_radius,
                opts.accepting_circle_width,
            );
        }
        if state.initial {
            drawer.set_color(opts.line_color);
            let start = cc + Pos2::y(opts.circle_radius + opts.center_line_padding);
            drawer.draw_line(
                start,
                start + Pos2::y(opts.init_arrow_length),
                opts.init_arrow_width,
            );
            drawer.draw_line(
                start,
                start + pos2(opts.init_arrow_arms_length, opts.init_arrow_arms_length),
                opts.init_arrow_width,
            );
            drawer.draw_line(
                start,
                start + pos2(-opts.init_arrow_arms_length, opts.init_arrow_arms_length),
                opts.init_arrow_width,
            );
        }

        drawer.set_color(opts.text_color);
        drawer.draw_centered_text(cc, state.name);
    }

    for arrow in arrows {
        drawer.set_color(opts.line_color);
        let line_height = opts.floor_height * (levels - arrow.level) as f32;

        drawer.draw_line(
            pos2(x_from_pos(arrow.arrow.left), line_baseline),
            pos2(x_from_pos(arrow.arrow.left), line_height),
            opts.trans_line_width,
        );
        drawer.draw_line(
            pos2(x_to_pos(arrow.arrow.right), line_baseline),
            pos2(x_to_pos(arrow.arrow.right), line_height),
            opts.trans_line_width,
        );
        drawer.draw_line(
            pos2(x_from_pos(arrow.arrow.left), line_height),
            pos2(x_to_pos(arrow.arrow.right), line_height),
            opts.trans_line_width,
        );
        let middle = (x_from_pos(arrow.arrow.left) + x_to_pos(arrow.arrow.right)) / 2.0;

        if opts.middle_arrow {
            let mul = if arrow.arrow.direction == Direction::Left {
                1.0
            } else {
                -1.0
            };
            let middle = middle - opts.trans_arrow_arms_length * mul * 0.5;
            drawer.draw_line(
                pos2(middle, line_height),
                pos2(
                    middle + mul * opts.trans_arrow_arms_length,
                    line_height + opts.trans_arrow_arms_length,
                ),
                opts.trans_line_width,
            );
            drawer.draw_line(
                pos2(middle, line_height),
                pos2(
                    middle + mul * opts.trans_arrow_arms_length,
                    line_height - opts.trans_arrow_arms_length,
                ),
                opts.trans_line_width,
            );
        }

        if opts.end_arrow {
            let (x, y) = if arrow.arrow.direction == Direction::Right {
                (x_to_pos(arrow.arrow.right), line_baseline)
            } else {
                (x_from_pos(arrow.arrow.left), line_baseline)
            };
            let base = pos2(x, y);
            drawer.draw_line(
                base,
                base - pos2(opts.trans_arrow_arms_length, opts.trans_arrow_arms_length),
                opts.trans_line_width,
            );
            drawer.draw_line(
                pos2(x, y),
                base - pos2(-opts.trans_arrow_arms_length, opts.trans_arrow_arms_length),
                opts.trans_line_width,
            );
        }

        drawer.set_color(opts.text_color);
        drawer.draw_centered_text(
            pos2(middle, line_height - opts.text_margin),
            &arrow.arrow.label(),
        );
    }
    drawer.finish_drawing();
}

pub fn dfa_ascii_art(dfa: &Dfa) -> String {
    let states = dfa.states().iter().map(Into::into).collect::<Vec<State>>();
    let arrows = dfa_to_arrows(dfa);
    ascii_art(states, arrows)
}

pub fn nfa_ascii_art(nfa: &Nfa) -> String {
    let states = nfa.states().iter().map(Into::into).collect::<Vec<State>>();
    let arrows = nfa_to_arrows(nfa);
    ascii_art(states, arrows)
}

fn ascii_art<'a>(states: Vec<State<'a>>, arrows: Vec<Arrow<'a>>) -> String {
    let widest_state_name = states.iter().map(|s| s.name.chars().count()).max().unwrap();

    // optional grouping
    let arrows = group_arrows(arrows);
    let (arrows, levels) = place_arrows(arrows);

    let left_x_idx = |idx: usize| -> usize {
        //-> ((a))
        5 + (7 + widest_state_name) * idx
    };
    let right_x_idx = |idx: usize| -> usize {
        //-> ((a))
        6 + widest_state_name + (7 + widest_state_name) * idx
    };
    let art_width = right_x_idx(states.len()) - 1;

    let last_line = {
        let pad = |s: &str, l: usize| {
            let cs = s.chars().count();
            if cs < l {
                let amnt = l - cs;
                format!("{}{}", s, &" ".repeat(amnt))
            } else {
                s.to_string()
            }
        };

        let mut acc = String::with_capacity(art_width);
        acc.push_str("-> ");
        states.iter().for_each(|state| {
            if state.accepting {
                acc.push_str(&format!("(( {} )) ", pad(state.name, widest_state_name)))
            } else {
                acc.push_str(&format!("(  {}  ) ", pad(state.name, widest_state_name)))
            }
        });
        acc
    };

    let mut lines = Vec::with_capacity(levels * 2 + 1);
    let mut bars = HashSet::new();
    for level in (0..levels).rev() {
        let mut top_line = " ".repeat(art_width);
        let mut bot_line = " ".repeat(art_width);

        bars.iter().for_each(|&bar| {
            unsafe { top_line.as_bytes_mut()[bar] = b'|' }
            unsafe { bot_line.as_bytes_mut()[bar] = b'|' }
        });

        arrows
            .iter()
            .filter(|arrow| arrow.level == level)
            .for_each(|arrow| {
                let (leftmost, rightmost) = if arrow.arrow.direction == Direction::Spot {
                    (left_x_idx(arrow.arrow.left), right_x_idx(arrow.arrow.right))
                } else {
                    (right_x_idx(arrow.arrow.left), left_x_idx(arrow.arrow.right))
                };

                // SAFETY: valid utf8 since only ascii is used and string is initially only ascii
                for x in leftmost..=rightmost {
                    unsafe { top_line.as_bytes_mut()[x] = b'-' }
                }
                match arrow.arrow.direction {
                    Direction::Left => unsafe {
                        top_line.as_bytes_mut()[left_x_idx(arrow.arrow.right) - 1] = b'<'
                    },
                    Direction::Right => unsafe {
                        top_line.as_bytes_mut()[right_x_idx(arrow.arrow.left) + 2] = b'>'
                    },
                    Direction::Spot => unsafe {
                        top_line.as_bytes_mut()[left_x_idx(arrow.arrow.left) + 1] = b'>'
                    },
                }

                bars.insert(right_x_idx(arrow.arrow.left));
                bars.insert(left_x_idx(arrow.arrow.right));
                unsafe { bot_line.as_bytes_mut()[right_x_idx(arrow.arrow.left)] = b'|' }
                unsafe { bot_line.as_bytes_mut()[left_x_idx(arrow.arrow.right)] = b'|' }
            });
        // We do this in a second for loop to
        // * make sure all shapes have been drawn out
        // * to draw the labels "on top" of those shapes
        // * to be able to disable label drawing
        arrows
            .iter()
            .filter(|arrow| arrow.level == level)
            .for_each(|arrow| {
                // copy label
                if arrow.arrow.left != arrow.arrow.right {
                    // Note that label can be non-ascii (as for ε), so make sure we replace correct amnt of spaces
                    let label = arrow.arrow.label();
                    let range = {
                        // We need to replace the nth *char* which is not necessarily same as the nth byte (we could
                        // have inserted labels to the left)
                        let start = right_x_idx(arrow.arrow.left);
                        let start = bot_line.char_indices().nth(start).unwrap().0;
                        // We need to replace equally many spaces as the label has (visible) chars
                        start + 1..start + label.chars().count() + 1
                    };
                    bot_line.replace_range(range, &label);
                } else {
                    // Space is tight, so don't print this
                    // let label = arrow.arrow.label();
                    // let range = {
                    //     let start = left_x_idx(arrow.arrow.left);
                    //     start + 1..start + label.len() + 1
                    // };
                    // // SAFETY: beforehand only ascii, and label is valid string
                    // unsafe { bot_line.as_bytes_mut()[range].copy_from_slice(label.as_bytes()) }
                }
            });
        lines.push(top_line);
        lines.push(bot_line);
    }

    lines.push(last_line);
    lines.join("\n")
}

fn dfa_to_arrows(dfa: &Dfa) -> Vec<Arrow> {
    dfa.states()
        .iter()
        .enumerate()
        .flat_map(|(from, state)| {
            state
                .transitions()
                .iter()
                .enumerate()
                .map(move |(idx, to)| Arrow::new(from, *to, &dfa.alphabet()[idx]))
        })
        .collect()
}

fn nfa_to_arrows(nfa: &Nfa) -> Vec<Arrow> {
    nfa.states()
        .iter()
        .enumerate()
        .flat_map(|(from, state)| {
            state
                .transitions()
                .iter()
                .enumerate()
                .flat_map(move |(idx, tos)| {
                    tos.iter()
                        .map(move |to| Arrow::new(from, *to, &nfa.alphabet()[idx]))
                })
                .chain(
                    state
                        .epsilon_transitions()
                        .iter()
                        .map(move |to| Arrow::new(from, *to, "ε")),
                )
        })
        .collect()
}

fn place_arrows<'a, T: ArrowLike<'a>>(arrows: Vec<T>) -> (Vec<PlacedArrow<'a, T>>, usize) {
    let mut unplaced = arrows;
    unplaced.sort_by_key(|arrow| usize::MAX - arrow.right()); // sort in reverse order
    let mut current_depth = 0;
    let mut end_of_last = 0;
    let mut placed = Vec::with_capacity(unplaced.len());

    while !unplaced.is_empty() {
        // do one pass and place as many arrows as possible
        let mut old_unplaced = mem::take(&mut unplaced);
        while let Some(arrow) = old_unplaced.pop() {
            if arrow.left() >= end_of_last {
                if arrow.left() == arrow.right() {
                    end_of_last = arrow.right() + 1;
                } else {
                    end_of_last = arrow.right();
                }
                placed.push(PlacedArrow {
                    arrow,
                    level: current_depth,
                    phantom: PhantomData,
                });
            } else {
                unplaced.push(arrow);
            }
        }
        current_depth += 1;
        unplaced.reverse();
        end_of_last = 0;
    }

    (placed, current_depth)
}

fn group_arrows(arrows: Vec<Arrow>) -> Vec<GroupedArrow> {
    arrows
        .into_iter()
        .fold(HashMap::<_, Vec<Arrow>>::new(), |mut map, arrow| {
            map.entry((arrow.left, arrow.right, arrow.direction))
                .or_default()
                .push(arrow);
            map
        })
        .drain()
        .map(|((left, right, direction), arrows)| GroupedArrow {
            left,
            right,
            direction,
            labels: arrows.into_iter().map(|arrow| arrow.label).collect(),
        })
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
struct PlacedArrow<'a, T: ArrowLike<'a>> {
    arrow: T,
    level: usize,
    phantom: PhantomData<&'a T>,
}

#[derive(Debug, PartialEq, Eq)]
struct GroupedArrow<'a> {
    left: usize,
    right: usize,
    direction: Direction,
    labels: Vec<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
struct Arrow<'a> {
    left: usize,
    right: usize,
    direction: Direction,
    label: &'a str,
}

impl<'a> Arrow<'a> {
    fn new(from: usize, to: usize, label: &'a str) -> Self {
        use std::cmp::Ordering::*;
        use Direction::*;
        match from.cmp(&to) {
            Less => Arrow {
                left: from,
                right: to,
                direction: Right,
                label,
            },
            Equal => Arrow {
                left: from,
                right: to,
                direction: Spot,
                label,
            },
            Greater => Arrow {
                left: to,
                right: from,
                direction: Left,
                label,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
enum Direction {
    Left,
    Right,
    Spot,
}

trait ArrowLike<'a> {
    fn left(&self) -> usize;
    fn right(&self) -> usize;
    fn direction(&self) -> Direction;
    fn label(&self) -> Cow<'a, str>;
}

impl<'a> ArrowLike<'a> for Arrow<'a> {
    fn left(&self) -> usize {
        self.left
    }

    fn right(&self) -> usize {
        self.right
    }

    fn direction(&self) -> Direction {
        self.direction
    }

    fn label(&self) -> Cow<'a, str> {
        Cow::Borrowed(self.label)
    }
}

impl<'a> ArrowLike<'a> for GroupedArrow<'a> {
    fn left(&self) -> usize {
        self.left
    }

    fn right(&self) -> usize {
        self.right
    }

    fn direction(&self) -> Direction {
        self.direction
    }

    fn label(&self) -> Cow<'static, str> {
        Cow::Owned(self.labels.join(", "))
    }
}

struct State<'a> {
    name: &'a str,
    accepting: bool,
    #[allow(dead_code)]
    initial: bool,
}

impl<'a> From<&'a DfaState> for State<'a> {
    fn from(value: &'a DfaState) -> Self {
        State {
            name: value.name(),
            accepting: value.is_accepting(),
            initial: value.is_initial(),
        }
    }
}

impl<'a> From<&'a NfaState> for State<'a> {
    fn from(value: &'a NfaState) -> Self {
        State {
            name: value.name(),
            accepting: value.is_accepting(),
            initial: value.is_initial(),
        }
    }
}
