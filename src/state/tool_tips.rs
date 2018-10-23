use imgui::Ui;

pub fn tool_tip_with_text(ui: &Ui, show: bool, text: impl AsRef<str>) {
    if !show {
        return;
    }

    if ui.is_item_hovered() {
        ui.tooltip_text(text);
    }
}

pub fn horizontal_split_tt(ui: &Ui, show: bool) {
    tool_tip_with_text(
        ui,
        show,
        "Splits the current window horizontally\nadding a new globe",
    );
}

pub fn vertical_split_tt(ui: &Ui, show: bool) {
    tool_tip_with_text(
        ui,
        show,
        "Splits the current window vertically\nadding a new globe",
    );
}

pub fn collapse_tt(ui: &Ui, show: bool) {
    tool_tip_with_text(
        ui,
        show,
        "Combines this window with the window it was split with\nany globes inside of the windows will be removed",
    );
}


pub fn lock_tt(ui: &Ui, show: bool) {
    tool_tip_with_text(
        ui,
        show,
        "Locks this globe's rotation to be the\nsame as any other locked globes",
    );
}

pub fn unlock_tt(ui: &Ui, show: bool) {
    tool_tip_with_text(
        ui,
        show,
        "Allows this globe to rotate freely",
    );
}

pub fn height_tt(ui: &Ui, show: bool) {
    tool_tip_with_text(
        ui,
        show,
        "Changes how much the Earth's elevation affects the globe",
    );
}

pub fn drag_speed_tt(ui: &Ui, show: bool) {
    tool_tip_with_text(
        ui,
        show,
        "Changes how fast you can rotate the globe",
    );
}

pub fn time_multi_tt(ui: &Ui, show: bool) {
    tool_tip_with_text(
        ui,
        show,
        "Changes how fast time based variables will play",
    );
}

pub fn value_selector_tt(ui: &Ui, show: bool) {
    tool_tip_with_text(
        ui,
        show,
        "Select which variable you want to display on this globe",
    );
}

pub fn max_range_tt(ui: &Ui, show: bool) {
    tool_tip_with_text(
        ui,
        show,
        "Changes the upper range value",
    );
}

pub fn min_range_tt(ui: &Ui, show: bool) {
    tool_tip_with_text(
        ui,
        show,
        "Changes the lower range value",
    );
}

pub fn time_slider_tt(ui: &Ui, show: bool) {
    tool_tip_with_text(
        ui,
        show,
        "Changes the month selected for this variable",
    );
}

pub fn play_tt(ui: &Ui, show: bool) {
    tool_tip_with_text(
        ui,
        show,
        "Loops this variable through each month",
    );
}

pub fn pause_tt(ui: &Ui, show: bool) {
    tool_tip_with_text(
        ui,
        show,
        "Stop looping this variable through each month",
    );
}