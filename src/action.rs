use std::path::PathBuf;

use strum_macros::Display;

use crate::ui::AppTab;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum Action {
    Quit,
    ProcessCommand(String),
    EnterCommandMode,
    LeaveCommandMode,
    SwitchTab(AppTab),
    UpdateView,
    ChangeScale(ScaleAction),
    LoadStudentList(PathBuf),
    ExportTo(PathBuf),
    IncrementStudentPoints(String),
    DecrementStudentPoints(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum ScaleAction {
    IncrementThreshold(u8), // u8 repr grade for which the threshold should be changed
    DecrementThreshold(u8), // u8 repr grade for which the threshold should be changed
    IncrementMaxPoints,
    DecrementMaxPoints,
    SetMaxPoints(u16),
    ToggleHalfPoints,
    SetScale(u8), // u8 repr grade. See GradeScaleType::try_from()
}
