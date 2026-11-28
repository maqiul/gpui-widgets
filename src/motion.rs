//! Shared motion primitives. Animations only request frames while their value changes.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::{
    AnyElement, App, Bounds, Div, Element, ElementId, GlobalElementId, Hsla, InspectorElementId,
    InteractiveElement, IntoElement, LayoutId, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, Window, div, ease_in_out, linear_color_stop,
    linear_gradient, relative,
};

use crate::theme;

pub fn fast_duration() -> Duration {
    duration_from_seconds(theme::with(|t| t.motion_duration_fast))
}

pub fn mid_duration() -> Duration {
    duration_from_seconds(theme::with(|t| t.motion_duration_mid))
}

pub fn slow_duration() -> Duration {
    duration_from_seconds(theme::with(|t| t.motion_duration_slow))
}

fn duration_from_seconds(seconds: f32) -> Duration {
    if seconds.is_finite() && seconds > 0.0 {
        Duration::from_secs_f32(seconds)
    } else {
        Duration::ZERO
    }
}

pub fn lerp(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}

pub fn mix_color(from: Hsla, to: Hsla, progress: f32) -> Hsla {
    Hsla {
        h: lerp(from.h, to.h, progress),
        s: lerp(from.s, to.s, progress),
        l: lerp(from.l, to.l, progress),
        a: lerp(from.a, to.a, progress),
    }
}

pub fn ease_out_cubic(value: f32) -> f32 {
    1.0 - (1.0 - value).powi(3)
}

pub fn pulse(delta: f32, min: f32, max: f32) -> f32 {
    let phase = if delta < 0.5 {
        delta * 2.0
    } else {
        2.0 - delta * 2.0
    };
    lerp(min, max, phase)
}

pub fn shimmer(delta: f32) -> f32 {
    let phase = (delta * 2.0).fract();
    if phase < 0.5 {
        phase * 2.0
    } else {
        2.0 - phase * 2.0
    }
}

pub fn rotation_degrees(delta: f32) -> f32 {
    delta.fract() * 360.0
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ExitState {
    started_at: Option<Instant>,
}

impl ExitState {
    pub fn begin(&mut self, now: Instant) {
        if self.started_at.is_none() {
            self.started_at = Some(now);
        }
    }

    pub fn cancel(&mut self) {
        self.started_at = None;
    }

    pub fn is_exiting(self) -> bool {
        self.started_at.is_some()
    }

    pub fn is_complete(self, now: Instant, duration: Duration) -> bool {
        self.started_at
            .is_some_and(|started_at| now.duration_since(started_at) >= duration)
    }
}

pub fn interaction_target(hovered: bool, pressed: bool) -> f32 {
    match (hovered, pressed) {
        (_, true) => 1.0,
        (true, false) => 0.5,
        (false, false) => 0.0,
    }
}

pub fn interactive_color(rest: Hsla, hover: Hsla, active: Hsla, value: f32) -> Hsla {
    if value <= 0.5 {
        mix_color(rest, hover, value * 2.0)
    } else {
        mix_color(hover, active, (value - 0.5) * 2.0)
    }
}

pub struct InteractionState {
    value: Cell<f32>,
    hovered: Cell<bool>,
    pressed: Cell<bool>,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            value: Cell::new(0.0),
            hovered: Cell::new(false),
            pressed: Cell::new(false),
        }
    }
}

pub fn keyed_interaction_state(
    window: &mut Window,
    cx: &mut App,
    id: ElementId,
) -> Rc<InteractionState> {
    window
        .use_keyed_state(id, cx, |_, _| Rc::new(InteractionState::default()))
        .read(cx)
        .clone()
}

pub fn bind_hover_refresh<E: InteractiveElement + StatefulInteractiveElement>(
    element: E,
    state: Rc<InteractionState>,
) -> E {
    let hover_state = state.clone();
    let mut element = element.on_hover(move |hovered, window, _cx| {
        hover_state.hovered.set(*hovered);
        hover_state
            .value
            .set(interaction_target(*hovered, hover_state.pressed.get()));
        window.refresh();
    });
    let down_state = state.clone();
    element = element.on_mouse_down(gpui::MouseButton::Left, move |_, window, _cx| {
        down_state.pressed.set(true);
        down_state
            .value
            .set(interaction_target(down_state.hovered.get(), true));
        window.refresh();
    });
    let up_state = state;
    element.on_mouse_up(gpui::MouseButton::Left, move |_, window, _cx| {
        up_state.pressed.set(false);
        up_state
            .value
            .set(interaction_target(up_state.hovered.get(), false));
        window.refresh()
    })
}

pub fn hover_transition(
    id: impl Into<ElementId>,
    element: Div,
    apply: impl Fn(Stateful<Div>, f32) -> Stateful<Div> + 'static,
) -> AnimatedValue<Stateful<Div>> {
    let id = id.into();
    AnimatedValue::new(id.clone(), element.id(id), 0.0, apply)
        .hover_refresh(bind_hover_refresh)
        .duration(fast_duration())
}

#[derive(Clone)]
enum Target {
    Value(f32),
    Interaction,
    Shared(Rc<InteractionState>),
}

enum Apply<E> {
    Value(Rc<dyn Fn(E, f32) -> E>),
    WithInteraction(Rc<dyn Fn(E, f32, f32) -> E>),
}

impl<E> Clone for Apply<E> {
    fn clone(&self) -> Self {
        match self {
            Self::Value(apply) => Self::Value(apply.clone()),
            Self::WithInteraction(apply) => Self::WithInteraction(apply.clone()),
        }
    }
}

struct MotionState {
    value: f32,
    target: f32,
    from: f32,
    started_at: Instant,
    interaction_value: f32,
    interaction_target: f32,
    interaction_from: f32,
    interaction_started_at: Instant,
    interaction: Rc<InteractionState>,
}

pub struct AnimatedValue<E> {
    id: ElementId,
    element: Option<E>,
    target: Target,
    duration: Duration,
    initial: Option<f32>,
    easing: fn(f32) -> f32,
    apply: Apply<E>,
    bind_interaction: Option<Rc<dyn Fn(E, Rc<InteractionState>) -> E>>,
}

impl<E> AnimatedValue<E> {
    pub fn new(
        id: impl Into<ElementId>,
        element: E,
        target: f32,
        apply: impl Fn(E, f32) -> E + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            element: Some(element),
            target: Target::Value(target),
            duration: mid_duration(),
            initial: None,
            easing: ease_out_cubic,
            apply: Apply::Value(Rc::new(apply)),
            bind_interaction: None,
        }
    }

    pub fn new_with_interaction(
        id: impl Into<ElementId>,
        element: E,
        target: f32,
        apply: impl Fn(E, f32, f32) -> E + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            element: Some(element),
            target: Target::Value(target),
            duration: mid_duration(),
            initial: None,
            easing: ease_out_cubic,
            apply: Apply::WithInteraction(Rc::new(apply)),
            bind_interaction: None,
        }
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn initial(mut self, value: f32) -> Self {
        self.initial = Some(value);
        self
    }

    pub fn easing(mut self, easing: fn(f32) -> f32) -> Self {
        self.easing = easing;
        self
    }

    pub fn target_shared(mut self, state: Rc<InteractionState>) -> Self {
        self.target = Target::Shared(state);
        self
    }

    pub fn hover_refresh(mut self, bind: impl Fn(E, Rc<InteractionState>) -> E + 'static) -> Self {
        self.target = Target::Interaction;
        self.bind_interaction = Some(Rc::new(bind));
        self
    }

    pub fn hover_refresh_value(
        mut self,
        value: f32,
        bind: impl Fn(E, Rc<InteractionState>) -> E + 'static,
    ) -> Self {
        self.target = Target::Value(value);
        self.bind_interaction = Some(Rc::new(bind));
        self
    }
}

impl<E> Element for AnimatedValue<E>
where
    E: IntoElement + 'static,
{
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let target = self.target.clone();
        let duration = self.duration;
        let easing = self.easing;
        let initial = self.initial;
        let element = self.element.take().expect("animated element is single-use");
        let apply = self.apply.clone();
        let animate_interaction = matches!(apply, Apply::WithInteraction(_));
        let bind_interaction = self.bind_interaction.take();
        window.with_element_state(global_id.unwrap(), |state, window| {
            let now = Instant::now();
            let mut state = state.unwrap_or_else(|| MotionState {
                value: initial.unwrap_or(0.0),
                target: 0.0,
                from: initial.unwrap_or(0.0),
                started_at: now,
                interaction_value: 0.0,
                interaction_target: 0.0,
                interaction_from: 0.0,
                interaction_started_at: now,
                interaction: Rc::new(InteractionState::default()),
            });
            let target = match target {
                Target::Value(value) => value,
                Target::Interaction => state.interaction.value.get(),
                Target::Shared(interaction) => interaction.value.get(),
            };
            if (state.target - target).abs() > f32::EPSILON {
                state.from = state.value;
                state.target = target;
                state.started_at = now;
            }
            let interaction_target = state.interaction.value.get();
            if (state.interaction_target - interaction_target).abs() > f32::EPSILON {
                state.interaction_from = state.interaction_value;
                state.interaction_target = interaction_target;
                state.interaction_started_at = now;
            }
            let element = if let Some(bind) = bind_interaction {
                bind(element, state.interaction.clone())
            } else {
                element
            };

            let mut done = true;
            if cx.reduce_motion() || duration.is_zero() {
                state.value = target;
                state.interaction_value = interaction_target;
            } else {
                let progress = (now.duration_since(state.started_at).as_secs_f32()
                    / duration.as_secs_f32())
                .clamp(0.0, 1.0);
                state.value = lerp(state.from, target, easing(progress));
                done = (state.from - target).abs() <= f32::EPSILON || progress >= 1.0;
                if animate_interaction {
                    let interaction_progress = (now
                        .duration_since(state.interaction_started_at)
                        .as_secs_f32()
                        / duration.as_secs_f32())
                    .clamp(0.0, 1.0);
                    state.interaction_value = lerp(
                        state.interaction_from,
                        interaction_target,
                        easing(interaction_progress),
                    );
                    done &= (state.interaction_from - interaction_target).abs() <= f32::EPSILON
                        || interaction_progress >= 1.0;
                }
            }

            let mut element = match apply {
                Apply::Value(apply) => apply(element, state.value),
                Apply::WithInteraction(apply) => {
                    apply(element, state.value, state.interaction_value)
                }
            }
            .into_any_element();
            if !done {
                window.request_animation_frame();
            }
            ((element.request_layout(window, cx), element), state)
        })
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<gpui::Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<gpui::Pixels>,
        element: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopEffect {
    Pulse,
    Shimmer,
}

pub struct MotionLoop {
    id: ElementId,
    child: Option<AnyElement>,
    effect: LoopEffect,
    period: Duration,
}

impl MotionLoop {
    pub fn new(id: impl Into<ElementId>, child: impl IntoElement, effect: LoopEffect) -> Self {
        Self {
            id: id.into(),
            child: Some(child.into_any_element()),
            effect,
            period: Duration::from_millis(1200),
        }
    }

    pub fn period(mut self, period: Duration) -> Self {
        self.period = if period.is_zero() {
            Duration::from_millis(1)
        } else {
            period
        };
        self
    }
}

impl Element for MotionLoop {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let effect = self.effect;
        let child = self.child.take().expect("motion loop is single-use");
        let period = self.period;
        window.with_element_state(global_id.unwrap(), |state, window| {
            let mut started_at = state.unwrap_or_else(Instant::now);
            let now = Instant::now();
            let reduced = cx.reduce_motion();
            let delta = if reduced {
                0.5
            } else {
                now.duration_since(started_at).as_secs_f32() / period.as_secs_f32()
            };
            let mut element = LoopSurface {
                child: Some(child),
                effect,
                delta,
            }
            .into_any_element();
            if reduced {
                started_at = now;
            } else {
                window.request_animation_frame();
            }
            ((element.request_layout(window, cx), element), started_at)
        })
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}

struct LoopSurface {
    child: Option<AnyElement>,
    effect: LoopEffect,
    delta: f32,
}

impl IntoElement for LoopSurface {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let mut root = div()
            .relative()
            .overflow_hidden()
            .child(self.child.expect("motion loop is single-use"));
        match self.effect {
            LoopEffect::Pulse => root.opacity(pulse(self.delta, 0.45, 1.0)),
            LoopEffect::Shimmer => {
                let left = shimmer(self.delta);
                root = root.child(
                    div()
                        .absolute()
                        .left(relative(left))
                        .top_0()
                        .bottom_0()
                        .w(relative(0.35))
                        .bg(linear_gradient(
                            90.0,
                            linear_color_stop(gpui::transparent_white(), 0.0),
                            linear_color_stop(gpui::white(), 1.0),
                        ))
                        .opacity(0.35),
                );
                root
            }
        }
    }
}

pub fn enter<E>(
    id: impl Into<ElementId>,
    element: E,
    apply: impl Fn(E, f32) -> E + 'static,
) -> AnimatedValue<E>
where
    E: IntoElement + 'static,
{
    AnimatedValue::new(id, element, 1.0, apply)
        .initial(0.0)
        .duration(mid_duration())
        .easing(ease_out_cubic)
}

pub fn breathe(delta: f32) -> f32 {
    pulse(delta, 0.45, 1.0)
}

pub fn breathe_easing(delta: f32) -> f32 {
    ease_in_out(delta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interaction_target_prioritizes_pressed() {
        assert_eq!(interaction_target(false, false), 0.0);
        assert_eq!(interaction_target(true, false), 0.5);
        assert_eq!(interaction_target(false, true), 1.0);
        assert_eq!(interaction_target(true, true), 1.0);
    }

    #[test]
    fn interpolation_clamps_at_call_sites() {
        assert_eq!(lerp(10.0, 20.0, 0.0), 10.0);
        assert_eq!(lerp(10.0, 20.0, 0.5), 15.0);
        assert_eq!(lerp(10.0, 20.0, 1.0), 20.0);
    }

    #[test]
    fn pulse_is_symmetric_and_bounded() {
        assert_eq!(pulse(0.0, 0.45, 1.0), 0.45);
        assert_eq!(pulse(0.5, 0.45, 1.0), 1.0);
        assert_eq!(pulse(1.0, 0.45, 1.0), 0.45);
    }

    #[test]
    fn shimmer_wraps_once_per_cycle() {
        assert_eq!(shimmer(0.0), 0.0);
        assert_eq!(shimmer(0.25), 1.0);
        assert_eq!(shimmer(0.5), 0.0);
        assert_eq!(shimmer(0.75), 1.0);
    }

    #[test]
    fn rotation_is_normalized() {
        assert_eq!(rotation_degrees(0.0), 0.0);
        assert_eq!(rotation_degrees(0.25), 90.0);
        assert_eq!(rotation_degrees(1.25), 90.0);
    }

    #[test]
    fn exit_starts_once_and_completes_at_duration() {
        let now = Instant::now();
        let duration = Duration::from_millis(20);
        let mut exit = ExitState::default();
        assert!(!exit.is_exiting());
        assert!(!exit.is_complete(now, duration));
        exit.begin(now);
        exit.begin(now + duration);
        assert!(!exit.is_complete(now + duration - Duration::from_millis(1), duration));
        assert!(exit.is_complete(now + duration, duration));
    }

    #[test]
    fn exit_can_be_revived_before_removal() {
        let now = Instant::now();
        let mut exit = ExitState::default();
        exit.begin(now);
        exit.cancel();
        assert!(!exit.is_exiting());
        exit.begin(now);
        assert_eq!(exit.started_at, Some(now));
    }
}
