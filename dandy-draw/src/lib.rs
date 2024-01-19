use std::borrow::Cow;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;
use dandy::dfa::Dfa;
use dandy::nfa::Nfa;

pub fn draw_demo(drawer: &mut impl Drawer) {
    drawer.start_drawing();
    drawer.draw_circle((30.0, 30.0), 20.0, 2.0);
    drawer.draw_circle((30.0, 30.0), 16.0, 2.0);
    drawer.finish_drawing();
}

pub trait Drawer {
    fn start_drawing(&mut self);
    fn finish_drawing(&mut self);
    fn draw_circle(&mut self, pos: (f32, f32), radius: f32, thickness: f32);
    fn draw_centered_text(&mut self, pos: (f32, f32), text: &str);
    fn draw_rect(&mut self, upper_left: (f32, f32), size: (f32, f32));
    fn draw_line(&mut self, from: (f32, f32), to: (f32, f32), thickness: f32);
}

pub fn ascii_art(dfa: &Dfa) -> String {
    let widest_state_name = dfa.states().iter().map(|s| s.name().chars().count()).max().unwrap();
    let left_x_idx = |idx: usize| -> usize {
        //-> ((a))
        5 + (7 + widest_state_name) * idx
    };
    let right_x_idx = |idx: usize| -> usize {
        //-> ((a))
        6 + widest_state_name + (7 + widest_state_name) * idx
    };
    let art_width = right_x_idx(dfa.states().len()) - 1;

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
        dfa.states().iter().for_each(|state|
            if state.is_accepting() {
                acc.push_str(&format!("(( {} )) ", pad(state.name(), widest_state_name)))
            } else {
                acc.push_str(&format!("(  {}  ) ", pad(state.name(), widest_state_name)))
            }
        );
        acc
    };

    let arrows = dfa_to_arrows(dfa);
    // optional grouping
    let arrows = group_arrows(arrows);

    let (arrows, levels) = dbg!(place_arrows(arrows));
    let mut lines = Vec::with_capacity(levels * 2 + 1);
    for level in (0..levels).rev() {
        let mut top_line = " ".repeat(art_width);
        let mut bot_line = " ".repeat(art_width);
        arrows.iter()
            .filter(|arrow| arrow.level == level)
            .for_each(|arrow| {
                let (leftmost, rightmost) = if arrow.arrow.direction == Direction::Spot {
                    (left_x_idx(arrow.arrow.left), right_x_idx(arrow.arrow.right))
                } else {
                    (right_x_idx(arrow.arrow.left), left_x_idx(arrow.arrow.right))
                };

                // SAFETY: valid utf8 since only ascii is used and string is initially only ascii
                for x in leftmost..=rightmost
                {
                    unsafe { top_line.as_bytes_mut()[x] = b'-' }
                }
                match arrow.arrow.direction {
                    Direction::Left => unsafe {
                        top_line.as_bytes_mut()[
                            left_x_idx(arrow.arrow.right) - 1
                            ] = b'<'
                    },
                    Direction::Right => unsafe {
                        top_line.as_bytes_mut()[
                            right_x_idx(arrow.arrow.left) + 2
                            ] = b'>'
                    },
                    Direction::Spot => unsafe {
                        top_line.as_bytes_mut()[
                            left_x_idx(arrow.arrow.left) + 1
                            ] = b'>'
                    }
                }
                unsafe { bot_line.as_bytes_mut()[right_x_idx(arrow.arrow.left)] = b'|' }
                unsafe { bot_line.as_bytes_mut()[left_x_idx(arrow.arrow.right)] = b'|' }
                // copy label
                if arrow.arrow.left != arrow.arrow.right {
                    let label = arrow.arrow.label();
                    let range = {
                        let start = right_x_idx(arrow.arrow.left);
                        start + 1..start + label.len() + 1
                    };
                    // SAFETY: beforehand only ascii, and label is valid string
                    unsafe { bot_line.as_bytes_mut()[range].copy_from_slice(label.as_bytes()) }
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

fn dfa_to_arrows<'a>(dfa: &'a Dfa) -> Vec<Arrow<'a>> {
    dfa.states().iter().enumerate().flat_map(|(from, state)|
        state.transitions().iter().enumerate().map(move |(idx, to)| Arrow::new(from, *to, dfa.states()[idx].name()))
    ).collect()
}

fn nfa_to_arrows(nfa: &Nfa) -> Vec<Arrow> {
    nfa.states().iter().enumerate().flat_map(|(from, state)|
        state.transitions().iter().enumerate().flat_map(move |(idx, tos)|
            tos.iter().map(move |to| Arrow::new(from, *to, nfa.states()[idx].name()))
        )
    ).collect()
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

fn group_arrows<'a>(arrows: Vec<Arrow<'a>>) -> Vec<GroupedArrow<'a>> {
    arrows
        .into_iter()
        .fold(HashMap::<_, Vec<Arrow>>::new(), |mut map, arrow|
            {
                map.entry((arrow.left, arrow.right, arrow.direction))
                    .or_default()
                    .push(arrow);
                map
            },
        )
        .drain()
        .map(|((left, right, direction), arrows)|
            GroupedArrow {
                left,
                right,
                direction,
                labels: arrows.into_iter().map(|arrow| arrow.label).collect(),
            }
        ).collect()
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
            Less =>
                Arrow {
                    left: from,
                    right: to,
                    direction: Right,
                    label,
                },
            Equal =>
                Arrow {
                    left: from,
                    right: to,
                    direction: Spot,
                    label,
                },
            Greater =>
                Arrow {
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

