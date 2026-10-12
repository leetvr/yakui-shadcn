//! Time-based motion using Yakui's retained widgets. Values belong to the app;
//! transitions retain only their visual interpolation at a stable call site.
//!
//! ```no_run
//! # use yakui_adequate_ui as ui;
//! # let expanded = false;
//! ui::animation::animate(if expanded { 320. } else { 64. }, Default::default(), |width| {
//!     ui::width(width, || ui::label("Navigation"));
//! });
//! ```
use std::{
    cell::RefCell,
    fmt::Debug,
    rc::Rc,
    time::{Duration, Instant},
};
use yakui::{Color, Vec2, widgets::Scope};
use yakui_core::{
    WidgetId, context,
    navigation::NavDirection,
    widget::{LayoutContext, NavigateContext, PaintContext, Widget},
};

/// CSS timing functions. `Standard` is Tailwind's default (0.4, 0, 0.2, 1).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Easing {
    Linear,
    #[default]
    Standard,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier([f32; 4]),
}
impl Easing {
    pub fn sample(self, progress: f32) -> f32 {
        let progress = progress.clamp(0., 1.);
        if progress == 0. || progress == 1. || self == Self::Linear {
            return progress;
        }
        let [x1, y1, x2, y2] = match self {
            Self::Standard => [0.4, 0., 0.2, 1.],
            Self::EaseIn => [0.4, 0., 1., 1.],
            Self::EaseOut => [0., 0., 0.2, 1.],
            Self::EaseInOut => [0.4, 0., 0.6, 1.],
            Self::CubicBezier(points) => points,
            Self::Linear => unreachable!(),
        };
        let curve = |t: f32, a: f32, b: f32| {
            let inverse = 1. - t;
            3. * inverse * inverse * t * a + 3. * inverse * t * t * b + t * t * t
        };
        // CSS gives x as elapsed time, not the Bezier parameter. Solve the
        // monotonic x curve first; sampling y at elapsed time produces wrong easing.
        let (mut low, mut high) = (0., 1.);
        for _ in 0..20 {
            let middle = (low + high) * 0.5;
            if curve(middle, x1, x2) < progress {
                low = middle;
            } else {
                high = middle;
            }
        }
        curve((low + high) * 0.5, y1, y2)
    }
}
/// Duration and timing curve, independent of the display's frame rate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Motion {
    pub duration: Duration,
    pub easing: Easing,
}
impl Default for Motion {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(150),
            easing: Easing::Standard,
        }
    }
}
impl Motion {
    pub const fn new(duration: Duration, easing: Easing) -> Self {
        Self { duration, easing }
    }
}
/// Values that can be interpolated by a retained transition.
pub trait Interpolate: Copy + Debug + PartialEq + 'static {
    fn interpolate(self, target: Self, progress: f32) -> Self;
}
impl Interpolate for f32 {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        self + (target - self) * progress
    }
}
impl Interpolate for Vec2 {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        self.lerp(target, progress)
    }
}
impl Interpolate for Color {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        self.lerp(&target, progress)
    }
}
impl<A: Interpolate, B: Interpolate> Interpolate for (A, B) {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        (
            self.0.interpolate(target.0, progress),
            self.1.interpolate(target.1, progress),
        )
    }
}
impl<A: Interpolate, B: Interpolate, C: Interpolate> Interpolate for (A, B, C) {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        (
            self.0.interpolate(target.0, progress),
            self.1.interpolate(target.1, progress),
            self.2.interpolate(target.2, progress),
        )
    }
}
/// An interruptible transition. Retargeting starts at the current sampled value.
/// The caller supplies a monotonic elapsed time, allowing deterministic captures.
#[derive(Debug, Clone, Copy)]
pub struct Transition<T: Interpolate> {
    from: T,
    target: T,
    started: Duration,
    motion: Motion,
}
impl<T: Interpolate> Transition<T> {
    pub fn new(value: T) -> Self {
        Self {
            from: value,
            target: value,
            started: Duration::ZERO,
            motion: Motion::default(),
        }
    }
    pub fn set_target(&mut self, target: T, motion: Motion, now: Duration) {
        if self.target != target {
            self.from = self.value(now);
            self.target = target;
            self.started = now;
            self.motion = motion;
        }
    }
    pub fn snap(&mut self, value: T) {
        self.from = value;
        self.target = value;
    }
    pub fn value(&self, now: Duration) -> T {
        if !self.is_active(now) {
            return self.target;
        }
        let progress =
            now.saturating_sub(self.started).as_secs_f32() / self.motion.duration.as_secs_f32();
        self.from
            .interpolate(self.target, self.motion.easing.sample(progress))
    }
    pub fn is_active(&self, now: Duration) -> bool {
        self.from != self.target && now.saturating_sub(self.started) < self.motion.duration
    }
}
#[derive(Debug, Clone, Copy)]
struct AnimationTime(Duration);
#[derive(Debug, Clone, Copy)]
struct MotionEnabled(bool);
#[derive(Debug)]
pub(crate) struct FrameClock {
    origin: Instant,
    frame: Instant,
    pub(crate) active: bool,
}
pub(crate) fn frame_clock() -> Rc<RefCell<FrameClock>> {
    clock_for(&context::dom())
}
fn clock_for(dom: &yakui_core::dom::Dom) -> Rc<RefCell<FrameClock>> {
    let frame = dom.frame_time();
    let clock = dom.get_global_or_init(|| {
        Rc::new(RefCell::new(FrameClock {
            origin: frame,
            frame,
            active: false,
        }))
    });
    if clock.borrow().frame != frame {
        let mut clock = clock.borrow_mut();
        clock.frame = frame;
        clock.active = false;
    }
    clock
}
/// Shared elapsed frame time. Normally captured once by `Yakui::start`.
pub fn now() -> Duration {
    if let Some(time) = yakui::util::read_scope::<AnimationTime>() {
        return time.0;
    }
    let clock = frame_clock();
    let clock = clock.borrow();
    clock.frame.saturating_duration_since(clock.origin)
}
/// Whether this frame declared active motion, for diagnostics and capture checks.
/// Games continue declaring and painting UI each vsync frame regardless. Call
/// after `Yakui::finish`, so transitions resolved during layout are included.
pub fn animations_active(yak: &yakui::Yakui) -> bool {
    clock_for(yak.dom()).borrow().active
}
pub(crate) fn request_frame() {
    frame_clock().borrow_mut().active = true;
}
pub(crate) fn enabled() -> bool {
    yakui::util::read_scope::<MotionEnabled>().is_none_or(|settings| settings.0)
}
/// Override time for deterministic tests, screenshots and motion sequences.
#[track_caller]
pub fn with_animation_time(time: Duration, children: impl FnOnce()) {
    Scope::new(AnimationTime(time)).show(children);
}
/// Disable motion within a subtree, settling all transitions immediately.
#[track_caller]
pub fn with_motion_enabled(enabled: bool, children: impl FnOnce()) {
    Scope::new(MotionEnabled(enabled)).show(children);
}

#[derive(Debug)]
struct Animate<T: Interpolate> {
    transition: Option<Transition<T>>,
}
impl<T: Interpolate> Widget for Animate<T> {
    type Props<'a> = (T, Motion);
    type Response = T;
    fn new() -> Self {
        Self { transition: None }
    }
    fn update(&mut self, (target, motion): Self::Props<'_>) -> T {
        update_transition(&mut self.transition, target, motion)
    }
}
pub(crate) fn update_transition<T: Interpolate>(
    state: &mut Option<Transition<T>>,
    target: T,
    motion: Motion,
) -> T {
    let now = now();
    let transition = state.get_or_insert_with(|| Transition::new(target));
    if enabled() {
        transition.set_target(target, motion, now);
    } else {
        transition.snap(target);
    }
    if transition.is_active(now) {
        request_frame();
    }
    transition.value(now)
}
/// Interpolate a value at this call site and declare its children with the sample.
/// Initial values render settled. This wrapper occupies its children's layout,
/// so animation state does not introduce empty siblings or extra list spacing.
#[track_caller]
pub fn animate<T: Interpolate, R>(target: T, motion: Motion, children: impl FnOnce(T) -> R) -> R {
    let dom = context::dom();
    let response = dom.begin_widget::<Animate<T>>((target, motion));
    let id = response.id;
    let result = children(response.into_inner());
    dom.end_widget::<Animate<T>>(id);
    result
}
/// Subtree paint effects. Transforms are visual; layout/hit bounds stay unchanged.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionStyle {
    pub opacity: f32,
    pub scale: Vec2,
    /// Clockwise rotation in radians about the subtree's centre.
    pub rotation: f32,
    pub translation: Vec2,
    /// Translation as a fraction of this subtree's laid-out dimensions.
    pub relative_translation: Vec2,
}
impl Default for MotionStyle {
    fn default() -> Self {
        Self {
            opacity: 1.,
            scale: Vec2::ONE,
            rotation: 0.,
            translation: Vec2::ZERO,
            relative_translation: Vec2::ZERO,
        }
    }
}
impl Interpolate for MotionStyle {
    fn interpolate(self, target: Self, progress: f32) -> Self {
        Self {
            opacity: self.opacity.interpolate(target.opacity, progress),
            scale: self.scale.interpolate(target.scale, progress),
            rotation: self.rotation.interpolate(target.rotation, progress),
            translation: self.translation.interpolate(target.translation, progress),
            relative_translation: self
                .relative_translation
                .interpolate(target.relative_translation, progress),
        }
    }
}
#[derive(Debug, Default)]
struct Effect {
    style: MotionStyle,
}
impl Widget for Effect {
    type Props<'a> = MotionStyle;
    type Response = ();
    fn new() -> Self {
        Self::default()
    }
    fn update(&mut self, style: MotionStyle) {
        self.style = style;
    }
    fn paint(&self, ctx: PaintContext<'_>) {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let transform = yakui_core::paint::PaintTransform {
            scale: self.style.scale,
            rotation: self.style.rotation,
            origin: rect.center(),
            translation: self.style.translation + self.style.relative_translation * rect.size(),
        };
        ctx.paint.with_transform(transform, |paint| {
            paint.with_opacity(self.style.opacity, |paint| {
                self.default_paint(PaintContext {
                    dom: ctx.dom,
                    layout: ctx.layout,
                    paint,
                })
            })
        });
    }
}
#[track_caller]
pub fn motion(style: MotionStyle, children: impl FnOnce()) {
    yakui::util::widget_children::<Effect, _>(children, style);
}
/// Presence keeps closed content mounted while its exit transition finishes.
#[derive(Debug, Clone, Copy)]
pub struct PresenceSample {
    pub progress: f32,
    pub present: bool,
    pub interactive: bool,
}
#[derive(Debug)]
struct Presence {
    transition: Transition<f32>,
    sample: PresenceSample,
}
impl Widget for Presence {
    type Props<'a> = (bool, Motion);
    type Response = PresenceSample;
    fn new() -> Self {
        Self {
            transition: Transition::new(0.),
            sample: PresenceSample {
                progress: 0.,
                present: false,
                interactive: false,
            },
        }
    }
    fn update(&mut self, (visible, motion): Self::Props<'_>) -> PresenceSample {
        let now = now();
        let target = if visible { 1. } else { 0. };
        if enabled() {
            self.transition.set_target(target, motion, now);
        } else {
            self.transition.snap(target);
        }
        let active = self.transition.is_active(now);
        if active {
            request_frame();
        }
        let progress = self.transition.value(now);
        self.sample = PresenceSample {
            progress,
            present: visible || active || progress > 0.,
            interactive: visible,
        };
        self.sample
    }
    fn layout(&self, ctx: LayoutContext<'_>, constraints: yakui::Constraints) -> Vec2 {
        let size = self.default_layout(
            LayoutContext {
                dom: ctx.dom,
                input: ctx.input,
                layout: ctx.layout,
                paint: ctx.paint,
            },
            constraints,
        );
        if !self.sample.interactive {
            ctx.layout.disable_input(ctx.dom, ctx.dom.current());
        }
        if self.sample.present {
            size
        } else {
            Vec2::ZERO
        }
    }
    fn navigate(&self, ctx: NavigateContext<'_>, direction: NavDirection) -> Option<WidgetId> {
        self.sample
            .interactive
            .then(|| self.default_navigate(ctx, direction))
            .flatten()
    }
}
/// The callback always runs so focus scopes can observe logical closure. Declare
/// visual children only when `sample.present`; closed content ignores input.
#[track_caller]
pub fn presence<R>(visible: bool, motion: Motion, children: impl FnOnce(PresenceSample) -> R) -> R {
    let dom = context::dom();
    let response = dom.begin_widget::<Presence>((visible, motion));
    let id = response.id;
    let result = children(response.into_inner());
    dom.end_widget::<Presence>(id);
    result
}

#[derive(Debug)]
struct Reveal {
    visible: bool,
    trigger: Option<WidgetId>,
    now: Duration,
    enabled: bool,
    height: RefCell<Option<Transition<f32>>>,
    clock: Option<Rc<RefCell<FrameClock>>>,
}
impl Widget for Reveal {
    type Props<'a> = (bool, Option<WidgetId>);
    type Response = bool;
    fn new() -> Self {
        Self {
            visible: false,
            trigger: None,
            now: Duration::ZERO,
            enabled: true,
            height: RefCell::new(None),
            clock: None,
        }
    }
    fn update(&mut self, (visible, trigger): Self::Props<'_>) -> bool {
        self.visible = visible;
        self.trigger = trigger;
        self.now = now();
        self.enabled = enabled();
        self.clock = Some(frame_clock());
        // Once exit is settled the content callback no longer runs. While exit
        // is active its children stay retained, preserving their paint and state.
        if let Some(transition) = self.height.borrow_mut().as_mut() {
            if !visible {
                if self.enabled {
                    transition.set_target(
                        0.,
                        Motion::new(Duration::from_millis(200), Easing::EaseOut),
                        self.now,
                    );
                } else {
                    transition.snap(0.);
                }
            }
            visible || transition.is_active(self.now) || transition.value(self.now) > 0.
        } else {
            visible
        }
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: yakui::Constraints) -> Vec2 {
        ctx.layout.enable_clipping(ctx.dom);
        let mut natural = Vec2::ZERO;
        for &child in &ctx.dom.get_current().children {
            natural = natural.max(ctx.calculate_layout(
                child,
                yakui::Constraints::loose(Vec2::new(constraints.max.x, f32::INFINITY)),
            ));
        }
        let target = if self.visible { natural.y } else { 0. };
        let mut state = self.height.borrow_mut();
        let transition = state.get_or_insert_with(|| Transition::new(target));
        if self.enabled {
            transition.set_target(
                target,
                Motion::new(Duration::from_millis(200), Easing::EaseOut),
                self.now,
            );
        } else {
            transition.snap(target);
        }
        if transition.is_active(self.now) {
            self.clock.as_ref().unwrap().borrow_mut().active = true;
        }
        let height = transition.value(self.now);
        if !self.visible {
            // Logical closure removes input immediately while layout clips the
            // retained content through its final height. Return child focus to
            // the header before disabling this subtree's event registrations.
            let mut current = ctx.input.focus();
            while let Some(id) = current {
                if id == ctx.dom.current() {
                    ctx.input.set_focus(self.trigger, false);
                    break;
                }
                current = ctx.dom.get(id).and_then(|node| node.parent);
            }
            ctx.layout.disable_input(ctx.dom, ctx.dom.current());
        }
        constraints.constrain(Vec2::new(natural.x, height))
    }
    fn navigate(&self, ctx: NavigateContext<'_>, direction: NavDirection) -> Option<WidgetId> {
        self.visible
            .then(|| self.default_navigate(ctx, direction))
            .flatten()
    }
}
#[track_caller]
pub(crate) fn reveal(visible: bool, trigger: Option<WidgetId>, children: impl FnOnce()) {
    let dom = context::dom();
    let response = dom.begin_widget::<Reveal>((visible, trigger));
    let id = response.id;
    if response.into_inner() {
        children();
    }
    dom.end_widget::<Reveal>(id);
}
