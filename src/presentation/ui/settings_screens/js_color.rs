//! The colour picker of the settings screens: a port of the `jscolor` widget the original uses.
//!
//! Same mechanism: a hex box that is tinted with the colour it holds (its text turns black or white,
//! whichever reads better), and a popup panel opened by pressing the box, with a hue x saturation pad
//! and a brightness slider. Picking a colour writes its hex into the box; typing 3 or 6 hex digits
//! moves the colour live. Every number below is `jscolor`'s own default or the option the original
//! passes (`width:240, height:160, position:'bottom', borderColor:'#212121', insetColor:'#161616',
//! backgroundColor:'#212121'`), and the pad and slider are drawn on `<canvas>` with the same gradients,
//! so the pixels come out the same.

use crate::application::app_state::AppContext;
use dioxus::prelude::*;
use std::collections::HashMap;
use wasm_bindgen::JsCast;

// --- the picker's geometry (jscolor defaults + the original's options) ---
const PAD_WIDTH: f64 = 240.0;
const PAD_HEIGHT: f64 = 160.0;
const PADDING: f64 = 12.0;
const SLIDER_WIDTH: f64 = 16.0;
const INSET: f64 = 1.0;
const BORDER: f64 = 1.0;
const RADIUS: f64 = 8.0;
const SHADOW_BLUR: f64 = 15.0;
const POINTER_BORDER: f64 = 1.0;
const POINTER_THICKNESS: f64 = 2.0;
const CROSS_SIZE: f64 = 8.0;
const SLIDER_POINTER_SPACE: f64 = 3.0;
/// `max(padding, 1.5 * (2 * pointerBorderWidth + pointerThickness))`
const PAD_TO_SLIDER: f64 = 12.0;
/// Inside the border: 2*inset + 2*padding + pad, then the slider column.
const BOX_WIDTH: f64 = 2.0 * INSET + 2.0 * PADDING + PAD_WIDTH + (2.0 * INSET + PAD_TO_SLIDER + SLIDER_WIDTH);
const BOX_HEIGHT: f64 = 2.0 * INSET + 2.0 * PADDING + PAD_HEIGHT;
const OUTER_WIDTH: f64 = BOX_WIDTH + 2.0 * BORDER;
const OUTER_HEIGHT: f64 = BOX_HEIGHT + 2.0 * BORDER;
const CROSS_OUTER: f64 = 2.0 * POINTER_BORDER + POINTER_THICKNESS + 2.0 * CROSS_SIZE;
const PAD_ID: &str = "jsc-pad-canvas";
const SLIDER_ID: &str = "jsc-slider-canvas";

// --- colour maths, exactly `jscolor`'s HSV_RGB / RGB_HSV / isLight ---

/// Hue 0-360, saturation and value 0-100 -> red, green, blue as floats 0-255.
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> [f64; 3] {
    let u = 255.0 * (v / 100.0);
    let h = h / 60.0;
    let s = s / 100.0;
    let i = h.floor();
    let f = if (i as i64) % 2 != 0 { h - i } else { 1.0 - (h - i) };
    let m = u * (1.0 - s);
    let n = u * (1.0 - s * f);
    match i as i64 {
        6 | 0 => [u, n, m],
        1 => [n, u, m],
        2 => [m, u, n],
        3 => [m, n, u],
        4 => [n, m, u],
        _ => [u, m, n],
    }
}

/// Red, green, blue 0-255 -> (hue or `None` for a grey, saturation, value).
fn rgb_to_hsv(r: f64, g: f64, b: f64) -> (Option<f64>, f64, f64) {
    let (r, g, b) = (r / 255.0, g / 255.0, b / 255.0);
    let low = r.min(g).min(b);
    let high = r.max(g).max(b);
    let spread = high - low;
    if spread == 0.0 {
        return (None, 0.0, 100.0 * high);
    }
    let h = if r == low { 3.0 + (b - g) / spread } else if g == low { 5.0 + (r - b) / spread } else { 1.0 + (g - r) / spread };
    (Some(60.0 * if h == 6.0 { 0.0 } else { h }), 100.0 * (spread / high), 100.0 * high)
}

fn hex_of(rgb: [f64; 3]) -> String {
    format!("{:02X}{:02X}{:02X}", rgb[0].round() as u8, rgb[1].round() as u8, rgb[2].round() as u8)
}

fn is_light(rgb: [f64; 3]) -> bool {
    0.213 * rgb[0] + 0.715 * rgb[1] + 0.072 * rgb[2] > 255.0 / 2.0
}

/// `jscolor`'s `fromString` for what people type: 3 or 6 hex digits, with any non-word characters around
/// them (a `#`, spaces). `None` while the text is not (yet) a colour.
pub fn parse_hex(text: &str) -> Option<[f64; 3]> {
    let digits = text.trim_matches(|c: char| !(c.is_alphanumeric() || c == '_'));
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let value = |slice: &str| u8::from_str_radix(slice, 16).ok().map(f64::from);
    match digits.len() {
        6 => Some([value(&digits[0..2])?, value(&digits[2..4])?, value(&digits[4..6])?]),
        3 => {
            let doubled: Vec<String> = digits.chars().map(|c| format!("{c}{c}")).collect();
            Some([value(&doubled[0])?, value(&doubled[1])?, value(&doubled[2])?])
        }
        _ => None,
    }
}

/// `jscolor.fromRGB`: the new hsv for a colour, keeping the old hue for a grey and the old saturation for black.
fn hsv_after_rgb(previous: [f64; 3], rgb: [f64; 3]) -> [f64; 3] {
    let (h, s, v) = rgb_to_hsv(rgb[0], rgb[1], rgb[2]);
    let mut hsv = previous;
    if let Some(h) = h {
        hsv[0] = h.clamp(0.0, 360.0);
    }
    if v != 0.0 {
        hsv[1] = s.clamp(0.0, 100.0);
    }
    hsv[2] = v.clamp(0.0, 100.0);
    hsv
}

// --- shared state ---

#[derive(Clone, Copy, PartialEq)]
enum Control {
    Pad,
    Slider,
}

#[derive(Clone, Copy, PartialEq)]
struct Placement {
    x: f64,
    y: f64,
    /// jscolor draws the drop shadow flat (no vertical offset) when the panel overlaps its own input.
    contract_shadow: bool,
}

/// What the one shared picker panel is doing. Provided once, by the root component.
#[derive(Clone, Copy)]
pub struct ColorPickerState {
    owner: Signal<Option<String>>,
    hsv_by_owner: Signal<HashMap<String, [f64; 3]>>,
    placement: Signal<Placement>,
    dragging: Signal<Option<Control>>,
}

impl ColorPickerState {
    pub fn new() -> Self {
        Self {
            owner: Signal::new(None),
            hsv_by_owner: Signal::new(HashMap::new()),
            placement: Signal::new(Placement { x: 0.0, y: 0.0, contract_shadow: false }),
            dragging: Signal::new(None),
        }
    }

    pub fn hide(mut self) {
        self.owner.set(None);
        self.dragging.set(None);
    }

    fn hsv_of(&self, id: &str) -> [f64; 3] {
        self.hsv_by_owner.read().get(id).copied().unwrap_or([0.0, 0.0, 100.0])
    }
}

/// Where the panel goes: below its input (`position:'bottom'`, `smartPosition`): flipped above when it
/// would run off the bottom, and against the input's right edge when it would run off the right.
fn place(input: (f64, f64, f64, f64), view: (f64, f64)) -> Placement {
    let ((left, top, width, height), (view_w, view_h)) = (input, view);
    let l = (height + OUTER_HEIGHT) / 2.0;
    let x = if left + OUTER_WIDTH > view_w {
        if left + width / 2.0 > view_w / 2.0 && left + width - OUTER_WIDTH >= 0.0 { left + width - OUTER_WIDTH } else { left }
    } else {
        left
    };
    let below = top + height;
    let y = if below + OUTER_HEIGHT > view_h {
        if top + height / 2.0 > view_h / 2.0 && below - 2.0 * l >= 0.0 { below - 2.0 * l } else { below }
    } else if below >= 0.0 {
        below
    } else {
        below - 2.0 * l
    };
    let contract_shadow = (x + OUTER_WIDTH > left || x < left + width) && (y + OUTER_HEIGHT < top + height);
    Placement { x, y, contract_shadow }
}

/// The hex box. Tinted with the colour it holds; pressing it opens the picker.
#[derive(Props, Clone, PartialEq)]
pub struct JsColorInputProps {
    pub id: String,
    /// The colour setting's current hex (6 digits).
    pub hex: String,
}

#[component]
pub fn JsColorInput(props: JsColorInputProps) -> Element {
    let context = use_context::<AppContext>();
    let mut picker = use_context::<ColorPickerState>();
    let id = props.id.clone();
    let mut text = use_signal(|| props.hex.clone());
    let mut mounted = use_signal(|| None::<std::rc::Rc<MountedData>>);

    // The colour itself always comes from the setting; the text is only what is typed in the box.
    let hex = props.hex.clone();
    use_effect(use_reactive!(|hex| {
        // (case-insensitive: the setting keeps the text exactly as it was typed)
        if parse_hex(&text.peek()).map(hex_of).map(|typed| typed.to_uppercase()) != Some(hex.to_uppercase()) {
            text.set(hex.clone());
        }
    }));
    let rgb = parse_hex(&props.hex).unwrap_or([255.0, 255.0, 255.0]);
    // jscolor starts every box with hsv [0, 0, 100] and folds the colour in.
    {
        let mut map = picker.hsv_by_owner;
        if !map.peek().contains_key(&props.id) {
            map.write().insert(props.id.clone(), hsv_after_rgb([0.0, 0.0, 100.0], rgb));
        }
    }
    let foreground = if is_light(rgb) { "#000" } else { "#FFF" };
    let style = format!("text-align:center;background-image:none;background-color:#{};color:{foreground}", props.hex);
    let is_open = picker.owner.read().as_deref() == Some(props.id.as_str());

    let open_id = id.clone();
    let type_id = id.clone();
    let blur_hex = props.hex.clone();
    rsx! {
        input {
            class: "shadow inputEff jscolor",
            class: if is_open { "jscolor-active" },
            style: "{style}",
            maxlength: "6",
            autocomplete: "off",
            value: "{text}",
            onmounted: move |event| mounted.set(Some(event.data())),
            onmousedown: move |event| {
                // Not the root's "press anywhere else closes the picker".
                event.stop_propagation();
                let id = open_id.clone();
                spawn(async move {
                    let Some(mounted) = mounted.peek().clone() else { return };
                    let Ok(rect) = mounted.get_client_rect().await else { return };
                    let window = web_sys::window();
                    let view = window
                        .map(|w| (w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1100.0), w.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(640.0)))
                        .unwrap_or((1100.0, 640.0));
                    picker.placement.set(place((rect.origin.x, rect.origin.y, rect.size.width, rect.size.height), view));
                    picker.owner.set(Some(id));
                });
            },
            oninput: move |event| {
                let typed = event.value();
                text.set(typed.clone());
                if let Some(rgb) = parse_hex(&typed) {
                    let mut map = picker.hsv_by_owner;
                    let previous = map.peek().get(&type_id).copied().unwrap_or([0.0, 0.0, 100.0]);
                    map.write().insert(type_id.clone(), hsv_after_rgb(previous, rgb));
                    // The original stores what was typed, as typed ("ff8000" stays lowercase). Three digits are
                    // stored in full: the original stores them raw and then draws them wrongly (a bug not copied).
                    let digits = if typed.trim().len() == 6 { typed.trim().to_string() } else { hex_of(rgb) };
                    let id = type_id.clone();
                    context.edit_settings(move |settings| settings.set_color_hex(&id, &digits));
                }
            },
            // Leaving the box shows the current colour in full again, in capitals (jscolor's `importColor`
            // writes the field but not the stored setting).
            onblur: move |_| text.set(blur_hex.to_uppercase()),
        }
    }
}

/// Applies a colour picked in the panel: remembered per box, saved as a setting.
fn apply_hsv(context: AppContext, mut picker: ColorPickerState, id: &str, hsv: [f64; 3]) {
    picker.hsv_by_owner.write().insert(id.to_string(), hsv);
    let digits = hex_of(hsv_to_rgb(hsv[0], hsv[1], hsv[2]));
    let id = id.to_string();
    context.edit_settings(move |settings| settings.set_color_hex(&id, &digits));
}

/// Where the pad's or slider's pointer is: the mouse position relative to the drawn canvas.
fn pick_from_pointer(context: AppContext, picker: ColorPickerState, control: Control, client_x: f64, client_y: f64) {
    let Some(id) = picker.owner.peek().clone() else { return };
    let placement = *picker.placement.peek();
    let mut hsv = picker.hsv_of(&id);
    // the canvases sit at (border + padding + inset) inside the panel; the slider is at the right
    let pad_left = placement.x + BORDER + PADDING + INSET;
    let top = placement.y + BORDER + PADDING + INSET;
    let y_value = 100.0 - (client_y - top) * (100.0 / (PAD_HEIGHT - 1.0));
    match control {
        Control::Pad => {
            let x_value = (client_x - pad_left) * (360.0 / (PAD_WIDTH - 1.0));
            hsv[0] = x_value.clamp(0.0, 360.0);
            hsv[1] = y_value.clamp(0.0, 100.0);
        }
        Control::Slider => hsv[2] = y_value.clamp(0.0, 100.0),
    }
    apply_hsv(context, picker, &id, hsv);
}

fn draw_canvases(hsv: [f64; 3]) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else { return };
    let context_of = |id: &str| -> Option<(web_sys::HtmlCanvasElement, web_sys::CanvasRenderingContext2d)> {
        let canvas = document.get_element_by_id(id)?.dyn_into::<web_sys::HtmlCanvasElement>().ok()?;
        let context = canvas.get_context("2d").ok()??.dyn_into::<web_sys::CanvasRenderingContext2d>().ok()?;
        Some((canvas, context))
    };
    if let Some((canvas, ctx)) = context_of(PAD_ID) {
        canvas.set_width(PAD_WIDTH as u32);
        canvas.set_height(PAD_HEIGHT as u32);
        let hue = ctx.create_linear_gradient(0.0, 0.0, PAD_WIDTH, 0.0);
        for (offset, color) in [(0.0, "#F00"), (1.0 / 6.0, "#FF0"), (2.0 / 6.0, "#0F0"), (3.0 / 6.0, "#0FF"), (4.0 / 6.0, "#00F"), (5.0 / 6.0, "#F0F"), (1.0, "#F00")] {
            let _ = hue.add_color_stop(offset as f32, color);
        }
        ctx.set_fill_style_canvas_gradient(&hue);
        ctx.fill_rect(0.0, 0.0, PAD_WIDTH, PAD_HEIGHT);
        let white = ctx.create_linear_gradient(0.0, 0.0, 0.0, PAD_HEIGHT);
        let _ = white.add_color_stop(0.0, "rgba(255,255,255,0)");
        let _ = white.add_color_stop(1.0, "rgba(255,255,255,1)");
        ctx.set_fill_style_canvas_gradient(&white);
        ctx.fill_rect(0.0, 0.0, PAD_WIDTH, PAD_HEIGHT);
    }
    if let Some((canvas, ctx)) = context_of(SLIDER_ID) {
        canvas.set_width(SLIDER_WIDTH as u32);
        canvas.set_height(PAD_HEIGHT as u32);
        let full = hsv_to_rgb(hsv[0], hsv[1], 100.0);
        let top = format!("rgb({},{},{})", full[0].round(), full[1].round(), full[2].round());
        let gradient = ctx.create_linear_gradient(0.0, 0.0, 0.0, PAD_HEIGHT);
        let _ = gradient.add_color_stop(0.0, &top);
        let _ = gradient.add_color_stop(1.0, "#000");
        ctx.set_fill_style_canvas_gradient(&gradient);
        ctx.fill_rect(0.0, 0.0, SLIDER_WIDTH, PAD_HEIGHT);
    }
}

/// The one shared panel. Rendered by the root component, outside everything else, so it can sit anywhere.
#[component]
pub fn JsColorPicker() -> Element {
    let context = use_context::<AppContext>();
    let picker = use_context::<ColorPickerState>();
    let owner = picker.owner.read().clone();
    let hsv = owner.as_deref().map(|id| picker.hsv_of(id)).unwrap_or([0.0, 0.0, 100.0]);
    // the canvases exist only while the panel is open; draw them again whenever the colour moves
    use_effect(move || {
        if picker.owner.read().is_some() {
            let hsv = picker.owner.read().as_deref().map(|id| picker.hsv_of(id)).unwrap_or([0.0, 0.0, 100.0]);
            draw_canvases(hsv);
        }
    });
    if owner.is_none() {
        return rsx! {};
    }
    let placement = *picker.placement.read();
    let dragging = *picker.dragging.read();

    let cross_x = (hsv[0] / 360.0 * (PAD_WIDTH - 1.0)).round() - (CROSS_OUTER / 2.0).floor();
    let cross_y = ((1.0 - hsv[1] / 100.0) * (PAD_HEIGHT - 1.0)).round() - (CROSS_OUTER / 2.0).floor();
    let slider_y = ((1.0 - hsv[2] / 100.0) * (PAD_HEIGHT - 1.0)).round();
    let slider_pointer_top = slider_y - (2.0 * POINTER_BORDER + POINTER_THICKNESS) - (SLIDER_POINTER_SPACE / 2.0).floor();
    let shadow_offset = if placement.contract_shadow { 0.0 } else { SHADOW_BLUR };
    let cross_line = (CROSS_OUTER / 2.0).floor() - (POINTER_THICKNESS / 2.0).floor();
    let cross_line_border = cross_line - POINTER_BORDER;
    let cross_long_inner = CROSS_OUTER - 2.0 * POINTER_BORDER;
    let thick_outer = 2.0 * POINTER_BORDER + POINTER_THICKNESS;

    rsx! {
        div {
            style: "clear:both;width:{OUTER_WIDTH}px;height:{OUTER_HEIGHT}px;z-index:1000;position:absolute;left:{placement.x}px;top:{placement.y}px",
            onmousedown: move |event| event.stop_propagation(),
            div { style: "position:absolute;left:0;top:0;width:100%;height:100%;border-radius:{RADIUS}px;box-shadow:0px {shadow_offset}px {SHADOW_BLUR}px 0px rgba(0,0,0,0.2)" }
            div { style: "position:relative;border:{BORDER}px solid;border-color:var(--color-panel);background:var(--color-panel);border-radius:{RADIUS}px",
                div { style: "width:{BOX_WIDTH}px;height:{BOX_HEIGHT}px",
                    // the hue x saturation pad
                    div { style: "position:absolute;left:{PADDING}px;top:{PADDING}px;border:{INSET}px solid;border-color:var(--color-panel-inset)",
                        div { style: "position:relative;width:{PAD_WIDTH}px;height:{PAD_HEIGHT}px", canvas { id: PAD_ID } }
                        div { style: "position:absolute;left:0;top:0;width:{CROSS_OUTER}px;height:{CROSS_OUTER}px;left:{cross_x}px;top:{cross_y}px",
                            div { style: "position:absolute;background:#FFF;width:{thick_outer}px;height:{CROSS_OUTER}px;left:{cross_line_border}px;top:0" }
                            div { style: "position:absolute;background:#FFF;height:{thick_outer}px;width:{CROSS_OUTER}px;top:{cross_line_border}px;left:0" }
                            div { style: "position:absolute;background:var(--color-pointer);height:{cross_long_inner}px;width:{POINTER_THICKNESS}px;left:{cross_line}px;top:{POINTER_BORDER}px" }
                            div { style: "position:absolute;background:var(--color-pointer);width:{cross_long_inner}px;height:{POINTER_THICKNESS}px;top:{cross_line}px;left:{POINTER_BORDER}px" }
                        }
                    }
                    div {
                        style: "position:absolute;left:0;top:0;width:{PADDING + 2.0 * INSET + PAD_WIDTH + PAD_TO_SLIDER / 2.0}px;height:{BOX_HEIGHT}px;cursor:crosshair;background:#FFF;opacity:0",
                        onmousedown: move |event| {
                            event.stop_propagation();
                            let point = event.client_coordinates();
                            pick_from_pointer(context, picker, Control::Pad, point.x, point.y);
                            let mut dragging = picker.dragging;
                            dragging.set(Some(Control::Pad));
                        },
                    }
                    // the brightness slider
                    div { style: "display:block;position:absolute;right:{PADDING}px;top:{PADDING}px;border:{INSET}px solid;border-color:var(--color-panel-inset)",
                        div { style: "overflow:hidden;width:{SLIDER_WIDTH}px;height:{PAD_HEIGHT}px", canvas { id: SLIDER_ID } }
                        div { style: "position:absolute;left:-{thick_outer}px;top:{slider_pointer_top}px;border:{POINTER_BORDER}px solid #FFF",
                            div { style: "border:{POINTER_THICKNESS}px solid var(--color-pointer)",
                                div { style: "border:{POINTER_BORDER}px solid #FFF",
                                    div { style: "width:{SLIDER_WIDTH}px;height:{SLIDER_POINTER_SPACE}px" }
                                }
                            }
                        }
                    }
                    div {
                        style: "display:block;position:absolute;right:0;top:0;width:{SLIDER_WIDTH + PAD_TO_SLIDER / 2.0 + PADDING + 2.0 * INSET}px;height:{BOX_HEIGHT}px;cursor:default;background:#FFF;opacity:0",
                        onmousedown: move |event| {
                            event.stop_propagation();
                            let point = event.client_coordinates();
                            pick_from_pointer(context, picker, Control::Slider, point.x, point.y);
                            let mut dragging = picker.dragging;
                            dragging.set(Some(Control::Slider));
                        },
                    }
                }
            }
        }
        // While a control is being dragged, one transparent layer over the whole window follows the mouse
        // (jscolor listens on the document for the same thing).
        if let Some(control) = dragging {
            div {
                style: if control == Control::Pad { "position:fixed;left:0;top:0;width:100%;height:100%;z-index:1001;cursor:crosshair" } else { "position:fixed;left:0;top:0;width:100%;height:100%;z-index:1001;cursor:default" },
                onmousemove: move |event| {
                    let point = event.client_coordinates();
                    pick_from_pointer(context, picker, control, point.x, point.y);
                },
                onmouseup: move |_| {
                    let mut dragging = picker.dragging;
                    dragging.set(None);
                },
                onmouseleave: move |_| {
                    let mut dragging = picker.dragging;
                    dragging.set(None);
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every expected value below was produced by running the original's own `HSV_RGB` / `RGB_HSV` / `isLight`
    // from jscolor.js, not derived here.

    #[test]
    fn hsv_to_hex_matches_jscolor() {
        let hex = |h, s, v| hex_of(hsv_to_rgb(h, s, v));
        assert_eq!(hex(0.0, 100.0, 100.0), "FF0000");
        assert_eq!(hex(60.0, 100.0, 100.0), "FFFF00");
        assert_eq!(hex(120.0, 50.0, 50.0), "408040");
        assert_eq!(hex(200.0, 80.0, 60.0), "1F7099");
        assert_eq!(hex(300.0, 25.0, 90.0), "E6ACE6");
        assert_eq!(hex(359.0, 100.0, 40.0), "660002");
        assert_eq!(hex(360.0, 100.0, 100.0), "FF0000"); // the pad's far-right edge wraps back to red
        assert_eq!(hex(30.0, 0.0, 70.0), "B3B3B3");
    }

    #[test]
    fn rgb_to_hsv_matches_jscolor() {
        let close = |a: f64, b: f64| (a - b).abs() < 1e-5;
        let (h, s, v) = rgb_to_hsv(255.0, 128.0, 0.0);
        assert!(close(h.unwrap(), 30.117647) && close(s, 100.0) && close(v, 100.0));
        let (h, s, v) = rgb_to_hsv(3.0, 169.0, 244.0);
        assert!(close(h.unwrap(), 198.672199) && close(s, 98.770492) && close(v, 95.686275));
        let (h, s, v) = rgb_to_hsv(18.0, 52.0, 86.0);
        assert!(close(h.unwrap(), 210.0) && close(s, 79.069767) && close(v, 33.72549));
        // a grey has no hue
        let (h, s, v) = rgb_to_hsv(33.0, 33.0, 33.0);
        assert!(h.is_none() && s == 0.0 && close(v, 12.941176));
    }

    #[test]
    fn text_is_black_on_light_colours_and_white_on_dark_ones() {
        assert!(is_light([255.0, 128.0, 0.0]));
        assert!(is_light([3.0, 169.0, 244.0]));
        assert!(is_light([128.0, 128.0, 128.0]));
        assert!(is_light([255.0, 255.0, 255.0]));
        assert!(!is_light([33.0, 33.0, 33.0]));
        assert!(!is_light([18.0, 52.0, 86.0]));
        assert!(!is_light([0.0, 0.0, 0.0]));
    }

    #[test]
    fn typed_text_is_a_colour_only_with_three_or_six_hex_digits() {
        assert_eq!(parse_hex("ff8000"), Some([255.0, 128.0, 0.0]));
        assert_eq!(parse_hex("0f0"), Some([0.0, 255.0, 0.0]));
        assert_eq!(parse_hex("#03A9F4"), Some([3.0, 169.0, 244.0])); // a leading # is allowed
        assert_eq!(parse_hex(" 123456 "), Some([18.0, 52.0, 86.0]));
        for partial in ["", "f", "ff", "ffff", "fffff", "ggg", "12345g", "1234567"] {
            assert_eq!(parse_hex(partial), None, "{partial:?}");
        }
    }

    #[test]
    fn a_grey_keeps_the_previous_hue_and_black_keeps_the_previous_saturation() {
        let previous = [200.0, 60.0, 80.0];
        assert_eq!(hsv_after_rgb(previous, [128.0, 128.0, 128.0])[0], 200.0);
        let black = hsv_after_rgb(previous, [0.0, 0.0, 0.0]);
        assert_eq!((black[0], black[1], black[2]), (200.0, 60.0, 0.0));
    }

    #[test]
    fn the_panel_opens_below_its_input_and_flips_when_it_would_run_off_the_window() {
        let input = (645.0, 319.0, 440.0, 32.0);
        let below = place(input, (1100.0, 640.0));
        assert_eq!((below.x, below.y), (645.0, 351.0)); // straight under the box
        // near the bottom of the window it goes above instead
        let low = place((645.0, 560.0, 440.0, 32.0), (1100.0, 640.0));
        assert_eq!(low.y, 560.0 - OUTER_HEIGHT);
        // too far right: its right edge lines up with the box's
        let right = place((900.0, 100.0, 150.0, 32.0), (1000.0, 640.0));
        assert_eq!(right.x, 900.0 + 150.0 - OUTER_WIDTH);
    }
}
