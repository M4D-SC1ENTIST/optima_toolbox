use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::egui;
use bevy_egui::egui::{Align2, Color32, Context, Id, Pos2, Response, Ui, Visuals};
use bevy_egui::egui::panel::{Side, TopBottomSide};
use optima_file::traits::{FromRonString, ToRonString};

#[derive(Resource)]
pub struct OEguiEngineWrapper(pub Mutex<OEguiEngine>);
impl OEguiEngineWrapper {
    pub fn get_mutex_guard(&self) -> MutexGuard<OEguiEngine> {
        self.0.lock().unwrap()
    }
}

pub struct OEguiEngine {
    ui_contains_pointer: bool,
    window_states: HashMap<String, OEguiWindowState>,
    side_panel_states: HashMap<String, OEguiSidePanelState>,
    top_bottom_panel_states: HashMap<String, OEguiTopBottomPanelState>,
    button_responses: HashMap<String, OEguiButtonResponse>,
    slider_responses: HashMap<String, OEguiSliderResponse>,
    checkbox_responses: HashMap<String, OEguiCheckboxResponse>,
    radiobutton_responses: HashMap<String, OEguiRadiobuttonResponse>,
    selector_responses: HashMap<String, OEguiSelectorResponse>
}
impl OEguiEngine {
    pub fn new() -> Self {
        Self {
            ui_contains_pointer: false,
            window_states: Default::default(),
            side_panel_states: Default::default(),
            top_bottom_panel_states: Default::default(),
            button_responses: Default::default(),
            slider_responses: Default::default(),
            checkbox_responses: Default::default(),
            radiobutton_responses: Default::default(),
            selector_responses: Default::default(),
        }
    }
    pub fn reset_on_frame(&mut self) {
        self.ui_contains_pointer = false;
        self.window_states.values_mut().for_each(|x| x.change_position = false);
    }
    pub fn ui_contains_pointer(&self) -> bool {
        self.ui_contains_pointer
    }
    pub fn open_window(&mut self, id_str: &str) {
        let window_state = self.window_states.get_mut(id_str);
        match window_state {
            None => {
                self.window_states.insert(id_str.to_string(), OEguiWindowState::new(true, Pos2::default(), false));
            }
            Some(window_state) => {
                window_state.open = true;
            }
        }
    }
    pub fn close_window(&mut self, id_str: &str) {
        let window_state = self.window_states.get_mut(id_str);
        match window_state {
            None => {
                self.window_states.insert(id_str.to_string(), OEguiWindowState::new(false, Pos2::default(), false));
            }
            Some(window_state) => {
                window_state.open = false;
            }
        }
    }
    pub fn set_position_of_window(&mut self, id_str: &str, pos: egui::Pos2) {
        let window_state = self.window_states.get_mut(id_str);
        match window_state {
            None => {
                self.window_states.insert(id_str.to_string(), OEguiWindowState::new(true, pos, true));
            }
            Some(window_state) => {
                window_state.position = pos;
                window_state.change_position = true;
            }
        }
    }
    pub fn open_side_panel(&mut self, id_str: &str) {
        let state = self.side_panel_states.get_mut(id_str);
        match state {
            None => {
                self.side_panel_states.insert(id_str.to_string(), OEguiSidePanelState { open: true });
            }
            Some(state) => {
                state.open = true;
            }
        }
    }
    pub fn close_side_panel(&mut self, id_str: &str) {
        let state = self.side_panel_states.get_mut(id_str);
        match state {
            None => {
                self.side_panel_states.insert(id_str.to_string(), OEguiSidePanelState { open: false });
            }
            Some(state) => {
                state.open = false;
            }
        }
    }
    pub fn open_top_bottom_panel(&mut self, id_str: &str) {
        let state = self.side_panel_states.get_mut(id_str);
        match state {
            None => {
                self.top_bottom_panel_states.insert(id_str.to_string(), OEguiTopBottomPanelState { open: true });
            }
            Some(state) => {
                state.open = true;
            }
        }
    }
    pub fn close_top_bottom_panel(&mut self, id_str: &str) {
        let state = self.side_panel_states.get_mut(id_str);
        match state {
            None => {
                self.top_bottom_panel_states.insert(id_str.to_string(), OEguiTopBottomPanelState { open: false });
            }
            Some(state) => {
                state.open = false;
            }
        }
    }
    pub fn set_style(&self, ctx: &Context) {
        let mut visuals = Visuals::dark();
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgba_premultiplied(150, 20, 20, 10);
        ctx.set_visuals(visuals);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! egui_engine_widget_helpers {
    ($fn_name: tt, $field_name: tt, $r: tt) => {
        #[allow(dead_code)]
        impl OEguiEngine {
            pub fn $fn_name(&self, id_str: &str) -> Option<&$r> {
                let response = self.$field_name.get(id_str);
                return response;
            }
        }
    }
}

egui_engine_widget_helpers!(get_button_response, button_responses, OEguiButtonResponse);
egui_engine_widget_helpers!(get_slider_response, slider_responses, OEguiSliderResponse);
egui_engine_widget_helpers!(get_checkbox_response, checkbox_responses, OEguiCheckboxResponse);
egui_engine_widget_helpers!(get_radiobutton_response, radiobutton_responses, OEguiRadiobuttonResponse);
egui_engine_widget_helpers!(get_selector_response, selector_responses, OEguiSelectorResponse);

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait OEguiWidgetTrait {
    type Args;

    fn show(&self, id_str: &str, ui: &mut Ui, egui_engine: &Res<OEguiEngineWrapper>, args: &Self::Args);
}

pub struct OEguiButton {
    text: String
}
impl OEguiButton {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string()
        }
    }
}
impl OEguiWidgetTrait for OEguiButton {
    type Args = ();

    fn show(&self, id_str: &str, ui: &mut Ui, egui_engine: &Res<OEguiEngineWrapper>, _args: &()) {
        let mut egui_engine = egui_engine.0.lock().unwrap();
        let response = ui.add(egui::widgets::Button::new(self.text.as_str()));
        egui_engine.button_responses.insert( id_str.to_string(), OEguiButtonResponse { widget_response: response } );
    }
}

pub struct OEguiButtonResponse {
    widget_response: Response
}
impl OEguiButtonResponse {
    pub fn widget_response(&self) -> &Response {
        &self.widget_response
    }
}

pub struct OEguiSlider {
    lower_range: f64,
    upper_range: f64
}
impl OEguiSlider {
    pub fn new(lower_range: f64, upper_range: f64) -> Self {
        Self {
            lower_range,
            upper_range,
        }
    }
}
impl OEguiWidgetTrait for OEguiSlider {
    type Args = ();

    fn show(&self, id_str: &str, ui: &mut Ui, egui_engine: &Res<OEguiEngineWrapper>, _args: &()) {
        let mut mutex_guard = egui_engine.get_mutex_guard();
        let stored_response = mutex_guard.slider_responses.get(id_str);
        let mut slider_value = match stored_response {
            None => { 0.0 }
            Some(stored_response) => { stored_response.slider_value }
        };
        let response = ui.add(egui::widgets::Slider::new(&mut slider_value, self.lower_range..=self.upper_range));
        mutex_guard.slider_responses.insert(id_str.to_string(), OEguiSliderResponse { widget_response: response, slider_value });
    }
}

pub struct OEguiSliderResponse {
    widget_response: Response,
    slider_value: f64,
}
impl OEguiSliderResponse {
    pub fn widget_response(&self) -> &Response {
        &self.widget_response
    }
    pub fn slider_value(&self) -> f64 {
        self.slider_value
    }
}

pub struct OEguiCheckbox { text: String }
impl OEguiCheckbox {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}
impl OEguiWidgetTrait for OEguiCheckbox {
    type Args = ();

    fn show(&self, id_str: &str, ui: &mut Ui, egui_engine: &Res<OEguiEngineWrapper>, _args: &()) {
        let mut mutex_guard = egui_engine.get_mutex_guard();
        let stored_response = mutex_guard.checkbox_responses.get_mut(id_str);
        let mut currently_selected = match stored_response {
            None => { false }
            Some(stored_response) => { stored_response.currently_selected }
        };
        let response = ui.add(egui::widgets::Checkbox::new(&mut currently_selected, self.text.as_str()));
        mutex_guard.checkbox_responses.insert(id_str.to_string(), OEguiCheckboxResponse { widget_response: response, currently_selected });
    }
}

pub struct OEguiCheckboxResponse {
    widget_response: Response,
    currently_selected: bool
}
impl OEguiCheckboxResponse {
    pub fn widget_response(&self) -> &Response {
        &self.widget_response
    }
    pub fn currently_selected(&self) -> bool {
        self.currently_selected
    }
}

pub struct OEguiRadiobutton { text: String }
impl OEguiRadiobutton {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string()
        }
    }
}
impl OEguiWidgetTrait for OEguiRadiobutton {
    type Args = ();

    fn show(&self, id_str: &str, ui: &mut Ui, egui_engine: &Res<OEguiEngineWrapper>, _immut_args: &Self::Args) {
        let mut mutex_guard = egui_engine.get_mutex_guard();
        let stored_response = mutex_guard.radiobutton_responses.get_mut(id_str);
        let currently_selected = match stored_response {
            None => { false }
            Some(stored_response) => { stored_response.currently_selected }
        };
        let response = ui.add(egui::widgets::RadioButton::new(currently_selected, self.text.as_str()));
        mutex_guard.radiobutton_responses.insert( id_str.to_string(), OEguiRadiobuttonResponse { widget_response: response, currently_selected } );
    }
}

pub struct OEguiRadiobuttonResponse {
    widget_response: Response,
    currently_selected: bool
}
impl OEguiRadiobuttonResponse {
    pub fn widget_response(&self) -> &Response {
        &self.widget_response
    }
    pub fn currently_selected(&self) -> bool {
        self.currently_selected
    }
}

pub struct OEguiSelector {
    egui_selector_mode: OEguiSelectorMode,
    selection_choices_as_ron_strings: Vec<String>,
    selection_display_strings: Option<Vec<String>>,
    allow_multiple_selections: bool,
}
impl OEguiSelector {
    pub fn new<S: ToRonString>(egui_selection_mode: OEguiSelectorMode,
                               selection_choices: Vec<S>,
                               selection_display_strings: Option<Vec<String>>,
                               allow_multiple_selections: bool) -> Self {
        Self {
            egui_selector_mode: egui_selection_mode,
            selection_choices_as_ron_strings: selection_choices.iter().map(|x| x.to_ron_string()).collect(),
            selection_display_strings,
            allow_multiple_selections,
        }
    }
}
impl OEguiWidgetTrait for OEguiSelector {
    type Args = Input<KeyCode>;

    fn show(&self, id_str: &str, ui: &mut Ui, egui_engine: &Res<OEguiEngineWrapper>, args: &Self::Args) {
        let mut mutex_guard = egui_engine.get_mutex_guard();
        let stored_response = mutex_guard.selector_responses.get_mut(id_str);
        match stored_response {
            None => { mutex_guard.selector_responses.insert(id_str.to_string(), OEguiSelectorResponse { current_selections_as_ron_strings: vec![] }); }
            Some(stored_response) => {
                let current_selections_as_ron_strings = &mut stored_response.current_selections_as_ron_strings;

                match &self.egui_selector_mode {
                    OEguiSelectorMode::RadioButtons
                    | OEguiSelectorMode::Checkboxes
                    | OEguiSelectorMode::SelectionText => {
                        self.selection_choices_as_ron_strings.iter().enumerate().for_each(|(i, s)| {
                            let currently_selected = current_selections_as_ron_strings.contains(s);
                            let mut currently_selected_copy = currently_selected.clone();

                            let display_string = match &self.selection_display_strings {
                                None => { s.clone() }
                                Some(d) => { d[i].clone() }
                            };

                            let selection_code: i8 = match &self.egui_selector_mode {
                                OEguiSelectorMode::RadioButtons => {
                                    if ui.radio(currently_selected_copy, display_string.as_str()).clicked() {
                                        if !currently_selected { 1 } else { -1 }
                                    } else { 0 }
                                }
                                OEguiSelectorMode::Checkboxes => {
                                    if ui.checkbox(&mut currently_selected_copy, display_string.as_str()).clicked() {
                                        if !currently_selected { 1 } else { -1 }
                                    } else { 0 }
                                }
                                OEguiSelectorMode::SelectionText => {
                                    if ui.selectable_label(currently_selected_copy, display_string.as_str()).clicked() {
                                        if !currently_selected { 1 } else { -1 }
                                    } else { 0 }
                                }
                                _ => { unreachable!(); }
                            };

                            let keys = args;
                            let shift_select = self.allow_multiple_selections & &(keys.pressed(KeyCode::ShiftRight) || keys.pressed(KeyCode::ShiftLeft));

                            if selection_code == -1 && shift_select {
                                current_selections_as_ron_strings.retain(|x| x != s)
                            } else if selection_code == -1 {
                                current_selections_as_ron_strings.clear();
                                current_selections_as_ron_strings.push(s.clone());
                            } else if selection_code == 1 && current_selections_as_ron_strings.len() == 0 {
                                current_selections_as_ron_strings.push(s.clone());
                            } else if selection_code == 1 && current_selections_as_ron_strings.len() >= 1 && shift_select {
                                current_selections_as_ron_strings.push(s.clone());
                            } else if selection_code == 1 && current_selections_as_ron_strings.len() >= 1 {
                                current_selections_as_ron_strings.clear();
                                current_selections_as_ron_strings.push(s.clone());
                            }
                        })
                    }
                    OEguiSelectorMode::ComboBox => {
                        assert!(!self.allow_multiple_selections, "Combobox cannot handle multiple selections.");
                        // assert!(self.selection_choices_as_ron_strings.len() > 0);
                        if current_selections_as_ron_strings.len() == 0 { current_selections_as_ron_strings.push(self.selection_choices_as_ron_strings[0].clone()) }
                        let selected = current_selections_as_ron_strings[0].clone();
                        let selected_display = if let Some(selection_display_strings) = &self.selection_display_strings {
                            let selected_idx = self.selection_choices_as_ron_strings.iter().position(|x| x == &selected).unwrap();
                            selection_display_strings[selected_idx].clone()
                        } else {
                            selected.clone()
                        };

                        egui::ComboBox::new(format!("{}_combobox", id_str), "")
                            .selected_text(format!("{}", selected_display))
                            .show_ui(ui, |ui| {
                                self.selection_choices_as_ron_strings.iter().enumerate().for_each(|(i, s)| {
                                    let display_string = if let Some(selection_display_strings) = &self.selection_display_strings {
                                        selection_display_strings[i].clone()
                                    } else {
                                        s.clone()
                                    };

                                    let mut ss = display_string.clone();
                                    if ui.selectable_value(&mut ss, selected_display.clone(), display_string.as_str()).clicked() {
                                        current_selections_as_ron_strings.clear();
                                        current_selections_as_ron_strings.push(s.clone());
                                    }
                                });
                            });
                    }
                }

                // egui_engine.selector_responses.lock().unwrap().insert(id_str.to_string(), OEguiSelectorResponse { current_selections_as_ron_strings });
            }
        }
    }
}

pub struct OEguiSelectorResponse {
    current_selections_as_ron_strings: Vec<String>
}
impl OEguiSelectorResponse {
    pub fn current_selections<S: FromRonString>(&self) -> Vec<S> {
        self.current_selections_as_ron_strings.iter().map(|x| S::from_ron_string(x) ).collect()
    }
    #[allow(dead_code)]
    pub (crate) fn current_selections_as_ron_strings(&self) -> &Vec<String> {
        &self.current_selections_as_ron_strings
    }
}

pub enum OEguiSelectorMode {
    RadioButtons, Checkboxes, SelectionText, ComboBox
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait OEguiContainerTrait {
    type Args;

    fn does_ui_contain_cursor(&self, ui: &mut Ui, right_buffer: f32, left_buffer: f32, top_buffer: f32, bottom_buffer: f32, window_query: &Query<&Window, With<PrimaryWindow>>) -> bool {
        let Ok(window) = window_query.get_single() else { return false; };

        let rect = ui.min_rect();

        let cursor = window.cursor_position();
        return match cursor {
            None => { false }
            Some(cursor) => {
                let x = cursor.x;
                let y = cursor.y;

                let right = rect.right();
                let left = rect.left();
                let top = rect.top();
                let bottom = rect.bottom();

                if x < right + right_buffer && x > left - left_buffer && y < bottom + bottom_buffer && y > top - top_buffer {
                    true
                } else {
                    false
                }
            }
        }
    }
    fn show<R, F: FnOnce(&mut Ui) -> R>(&self, id_str: &str, ctx: &Context, egui_engine: &Res<OEguiEngineWrapper>, window_query: &Query<&Window, With<PrimaryWindow>>, args: &Self::Args, add_contents: F );
}

pub struct OEguiWindow {
    title: String,
    collapsible: bool,
    movable: bool,
    hscroll: bool,
    vscroll: bool,
    resizable: bool,
    start_open: bool,
}
impl OEguiWindow {
    pub fn new(title: &str, collapsible: bool, movable: bool, hscroll: bool, vscroll: bool, resizable: bool, start_open: bool) -> Self {
        Self {
            title: title.to_string(),
            collapsible,
            movable,
            hscroll,
            vscroll,
            resizable,
            start_open
        }
    }
}
impl OEguiContainerTrait for OEguiWindow {
    type Args = ();

    fn show<R, F: FnOnce(&mut Ui) -> R>(&self, id_str: &str, ctx: &Context, egui_engine: &Res<OEguiEngineWrapper>, window_query: &Query<&Window, With<PrimaryWindow>>, _args: &Self::Args, add_contents: F ) {
        let egui_engine_mutex = egui_engine.0.lock().unwrap();
        let saved_state = egui_engine_mutex.window_states.get(id_str);
        match saved_state {
            None => {
                drop(egui_engine_mutex);
                let mut egui_engine_mutex = egui_engine.0.lock().unwrap();
                egui_engine_mutex.window_states.insert(id_str.to_string(), OEguiWindowState::new(self.start_open, Pos2::default(), false));
                return;
            }
            Some(saved_state) => {
                let mut open = saved_state.open;
                let change_position = saved_state.change_position;
                let position = saved_state.position;
                drop(egui_engine_mutex);

                let mut window = egui::Window::new(self.title.as_str());
                window = window.id(Id::new(id_str));
                window = window.collapsible(self.collapsible);
                window = window.hscroll(self.hscroll);
                window = window.vscroll(self.vscroll);
                window = window.open(&mut open);
                window = window.movable(self.movable);
                window = window.resizable(self.resizable);

                if change_position {
                    window = window.current_pos(position);
                }

                window.show(ctx, |ui| {
                        add_contents(ui);
                        let ui_contains_pointer = self.does_ui_contain_cursor(ui, 3.0, 3.0, 32.0, 10.0, window_query);
                        if ui_contains_pointer {
                            let mut egui_engine_mutex = egui_engine.get_mutex_guard();
                            egui_engine_mutex.ui_contains_pointer = true;
                        }
                    });

                let mut egui_engine_mutex = egui_engine.0.lock().unwrap();
                let state = egui_engine_mutex.window_states.get_mut(id_str).expect("error");
                state.open = open;
            }
        }
    }
}

pub struct OEguiWindowState {
    open: bool,
    position: egui::Pos2,
    change_position: bool
}
impl OEguiWindowState {
    pub fn new(open: bool, position: egui::Pos2, change_position: bool) -> Self {
        Self {
            open,
            position,
            change_position,
        }
    }
}

pub enum OEguiWindowPosition {
    Auto,
    Absolute(Pos2),
    Anchor { align: Align2, offset: egui::Vec2 },
    DefaultPosition(Pos2),
    AbsoluteFromState,
    DefaultPositionFromState
}

pub struct OEguiSidePanel {
    side: Side,
    default_width: f32
}
impl OEguiSidePanel {
    pub fn new(side: Side, default_width: f32) -> Self {
        Self {
            side,
            default_width,
        }
    }
}
impl OEguiContainerTrait for OEguiSidePanel {
    type Args = ();

    fn show<R, F: FnOnce(&mut Ui) -> R>(&self, id_str: &str, ctx: &Context, egui_engine: &Res<OEguiEngineWrapper>, window_query: &Query<&Window, With<PrimaryWindow>>, _args: &Self::Args, add_contents: F) {
        let mutex_guard = egui_engine.get_mutex_guard();
        let saved_state = mutex_guard.side_panel_states.get(id_str);
        match saved_state {
            None => {
                drop(mutex_guard);
                let mut egui_engine_mutex = egui_engine.get_mutex_guard();
                egui_engine_mutex.side_panel_states.insert(id_str.to_string(), OEguiSidePanelState { open: true });
                return;
            }
            Some(saved_state) => {
                let open = saved_state.open;
                drop(mutex_guard);
                egui::SidePanel::new(self.side, id_str.to_string())
                    .default_width(self.default_width)
                    .show_animated(ctx, open, |ui| {
                        add_contents(ui);
                        let ui_contains_pointer = self.does_ui_contain_cursor(ui, 3.0, 3.0, 32.0, 10.0, window_query);
                        if ui_contains_pointer {
                            let mut egui_engine_mutex = egui_engine.get_mutex_guard();
                            egui_engine_mutex.ui_contains_pointer = true;
                        }
                    });
            }
        }
    }
}

pub struct OEguiSidePanelState {
    open: bool
}
impl OEguiSidePanelState {
    pub fn open(&self) -> bool {
        self.open
    }
}

pub struct OEguiTopBottomPanel {
    side: TopBottomSide,
    default_height: f32
}
impl OEguiTopBottomPanel {
    pub fn new(side: TopBottomSide, default_height: f32) -> Self {
        Self {
            side,
            default_height,
        }
    }
}
impl OEguiContainerTrait for OEguiTopBottomPanel {
    type Args = ();

    fn show<R, F: FnOnce(&mut Ui) -> R>(&self, id_str: &str, ctx: &Context, egui_engine: &Res<OEguiEngineWrapper>, window_query: &Query<&Window, With<PrimaryWindow>>, _args: &Self::Args, add_contents: F) {
        let mutex_guard = egui_engine.get_mutex_guard();
        let saved_state = mutex_guard.top_bottom_panel_states.get(id_str);
        match saved_state {
            None => {
                drop(mutex_guard);
                let mut egui_engine_mutex = egui_engine.get_mutex_guard();
                egui_engine_mutex.top_bottom_panel_states.insert(id_str.to_string(), OEguiTopBottomPanelState { open: true });
                return;
            }
            Some(saved_state) => {
                let open = saved_state.open;
                drop(mutex_guard);
                egui::TopBottomPanel::new(self.side, id_str.to_string())
                    .default_height(self.default_height)
                    .show_animated(ctx, open, |ui| {
                        add_contents(ui);
                        let ui_contains_pointer = self.does_ui_contain_cursor(ui, 3.0, 3.0, 32.0, 10.0, window_query);
                        if ui_contains_pointer {
                            let mut egui_engine_mutex = egui_engine.get_mutex_guard();
                            egui_engine_mutex.ui_contains_pointer = true;
                        }
                    });
            }
        }
    }
}

pub struct OEguiTopBottomPanelState {
    open: bool
}

