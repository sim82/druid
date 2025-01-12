// Copyright 2019 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A slider widget.

use crate::kurbo::{Circle, Shape};
use crate::widget::prelude::*;
use crate::{theme, LinearGradient, Point, Rect, UnitPoint};
use tracing::{instrument, trace, warn};

const TRACK_THICKNESS: f64 = 4.0;
const BORDER_WIDTH: f64 = 2.0;
const KNOB_STROKE_WIDTH: f64 = 2.0;

/// A slider, allowing interactive update of a numeric value.
///
/// This slider implements `Widget<f64>`, and works on values clamped
/// in the range `min..max`.
#[derive(Debug, Clone, Default)]
pub struct Slider {
    min: f64,
    max: f64,
    step: Option<f64>,
    knob_pos: Point,
    knob_hovered: bool,
    x_offset: f64,
}

impl Slider {
    /// Create a new `Slider`.
    pub fn new() -> Slider {
        Slider {
            min: 0.,
            max: 1.,
            step: None,
            knob_pos: Default::default(),
            knob_hovered: Default::default(),
            x_offset: Default::default(),
        }
    }

    /// Builder-style method to set the range covered by this slider.
    ///
    /// The default range is `0.0..1.0`.
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Builder-style method to set the stepping.
    ///
    /// The default step size is `0.0` (smooth).
    pub fn with_step(mut self, step: f64) -> Self {
        if step < 0.0 {
            warn!("bad stepping (must be positive): {}", step);
            return self;
        }
        self.step = if step > 0.0 {
            Some(step)
        } else {
            // A stepping value of 0.0 would yield an infinite amount of steps.
            // Enforce no stepping instead.
            None
        };
        self
    }

    /// check self.min <= self.max, if not swaps the values.
    fn check_range(&mut self) {
        if self.max < self.min {
            warn!(
                "min({}) should be less than max({}), swaping the values",
                self.min, self.max
            );
            std::mem::swap(&mut self.max, &mut self.min);
        }
    }
}

impl Slider {
    fn knob_hit_test(&self, knob_width: f64, mouse_pos: Point) -> bool {
        let knob_circle = Circle::new(self.knob_pos, knob_width / 2.);
        knob_circle.winding(mouse_pos) > 0
    }

    fn calculate_value(&self, mouse_x: f64, knob_width: f64, slider_width: f64) -> f64 {
        let scalar = ((mouse_x + self.x_offset - knob_width / 2.) / (slider_width - knob_width))
            .max(0.0)
            .min(1.0);
        let mut value = self.min + scalar * (self.max - self.min);
        if let Some(step) = self.step {
            let max_step_value = ((self.max - self.min) / step).floor() * step + self.min;
            if value > max_step_value {
                // edge case: make sure max is reachable
                let left_dist = value - max_step_value;
                let right_dist = self.max - value;
                value = if left_dist < right_dist {
                    max_step_value
                } else {
                    self.max
                };
            } else {
                // snap to discrete intervals
                value = (((value - self.min) / step).round() * step + self.min).min(self.max);
            }
        }
        value
    }

    fn normalize(&self, data: f64) -> f64 {
        (data.max(self.min).min(self.max) - self.min) / (self.max - self.min)
    }
}

impl Widget<f64> for Slider {
    #[instrument(name = "Slider", level = "trace", skip(self, ctx, event, data, env))]
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut f64, env: &Env) {
        let knob_size = env.get(theme::BASIC_WIDGET_HEIGHT);
        let slider_width = ctx.size().width;

        match event {
            Event::MouseDown(mouse) => {
                if !ctx.is_disabled() {
                    ctx.set_active(true);
                    if self.knob_hit_test(knob_size, mouse.pos) {
                        self.x_offset = self.knob_pos.x - mouse.pos.x
                    } else {
                        self.x_offset = 0.;
                        *data = self.calculate_value(mouse.pos.x, knob_size, slider_width);
                    }
                    ctx.request_paint();
                }
            }
            Event::MouseUp(mouse) => {
                if ctx.is_active() && !ctx.is_disabled() {
                    *data = self.calculate_value(mouse.pos.x, knob_size, slider_width);
                    ctx.request_paint();
                }
                ctx.set_active(false);
            }
            Event::MouseMove(mouse) => {
                if !ctx.is_disabled() {
                    if ctx.is_active() {
                        *data = self.calculate_value(mouse.pos.x, knob_size, slider_width);
                        ctx.request_paint();
                    }
                    if ctx.is_hot() {
                        let knob_hover = self.knob_hit_test(knob_size, mouse.pos);
                        if knob_hover != self.knob_hovered {
                            self.knob_hovered = knob_hover;
                            ctx.request_paint();
                        }
                    }
                } else {
                    ctx.set_active(false);
                }
            }
            _ => (),
        }
    }

    #[instrument(name = "Slider", level = "trace", skip(self, ctx, event, _data, _env))]
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &f64, _env: &Env) {
        match event {
            // checked in LifeCycle::WidgetAdded because logging may not be setup in with_range
            LifeCycle::WidgetAdded => self.check_range(),
            LifeCycle::DisabledChanged(_) => ctx.request_paint(),
            _ => (),
        }
    }

    #[instrument(
        name = "Slider",
        level = "trace",
        skip(self, ctx, _old_data, _data, _env)
    )]
    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &f64, _data: &f64, _env: &Env) {
        ctx.request_paint();
    }

    #[instrument(name = "Slider", level = "trace", skip(self, ctx, bc, _data, env))]
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &f64, env: &Env) -> Size {
        bc.debug_check("Slider");
        let height = env.get(theme::BASIC_WIDGET_HEIGHT);
        let width = env.get(theme::WIDE_WIDGET_WIDTH);
        let baseline_offset = (height / 2.0) - TRACK_THICKNESS;
        ctx.set_baseline_offset(baseline_offset);
        let size = bc.constrain((width, height));
        trace!(
            "Computed layout: size={}, baseline_offset={:?}",
            size,
            baseline_offset
        );
        size
    }

    #[instrument(name = "Slider", level = "trace", skip(self, ctx, data, env))]
    fn paint(&mut self, ctx: &mut PaintCtx, data: &f64, env: &Env) {
        let clamped = self.normalize(*data);
        let rect = ctx.size().to_rect();
        let knob_size = env.get(theme::BASIC_WIDGET_HEIGHT);

        //Paint the background
        let background_width = rect.width() - knob_size;
        let background_origin = Point::new(knob_size / 2., (knob_size - TRACK_THICKNESS) / 2.);
        let background_size = Size::new(background_width, TRACK_THICKNESS);
        let background_rect = Rect::from_origin_size(background_origin, background_size)
            .inset(-BORDER_WIDTH / 2.)
            .to_rounded_rect(2.);

        let background_gradient = LinearGradient::new(
            UnitPoint::TOP,
            UnitPoint::BOTTOM,
            (
                env.get(theme::BACKGROUND_LIGHT),
                env.get(theme::BACKGROUND_DARK),
            ),
        );

        ctx.stroke(background_rect, &env.get(theme::BORDER_DARK), BORDER_WIDTH);

        ctx.fill(background_rect, &background_gradient);

        //Get ready to paint the knob
        let is_active = ctx.is_active();
        let is_hovered = self.knob_hovered;

        let knob_position = (rect.width() - knob_size) * clamped + knob_size / 2.;
        self.knob_pos = Point::new(knob_position, knob_size / 2.);
        let knob_circle = Circle::new(self.knob_pos, (knob_size - KNOB_STROKE_WIDTH) / 2.);

        let knob_gradient = if ctx.is_disabled() {
            LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (
                    env.get(theme::DISABLED_FOREGROUND_LIGHT),
                    env.get(theme::DISABLED_FOREGROUND_DARK),
                ),
            )
        } else if ctx.is_active() {
            LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (
                    env.get(theme::FOREGROUND_DARK),
                    env.get(theme::FOREGROUND_LIGHT),
                ),
            )
        } else {
            LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (
                    env.get(theme::FOREGROUND_LIGHT),
                    env.get(theme::FOREGROUND_DARK),
                ),
            )
        };

        //Paint the border
        let border_color = if (is_hovered || is_active) && !ctx.is_disabled() {
            env.get(theme::FOREGROUND_LIGHT)
        } else {
            env.get(theme::FOREGROUND_DARK)
        };

        ctx.stroke(knob_circle, &border_color, KNOB_STROKE_WIDTH);

        //Actually paint the knob
        ctx.fill(knob_circle, &knob_gradient);
    }
}
