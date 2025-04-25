use std::path::PathBuf;

use strum_macros::Display;

use crate::ui::AppTab;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum Action {
    Quit,
    EnterInsertMode,
    LeaveInsertMode,
    SwitchTab(AppTab),
    UpdateView,
    UpdateModel(ModelAction),
    LoadStudentList(PathBuf),
    ExportTo(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum ModelAction {
    IncrementThreshold(u8), // u8 repr grade for which the threshold should be changed
    DecrementThreshold(u8), // u8 repr grade for which the threshold should be changed
    IncrementMaxPoints,
    DecrementMaxPoints,
    SetMaxPoints(u16),
    ToggleHalfPoints,
    SetScale(u8), // u8 repr grade. See GradeScaleType::try_from()
    IncrementStudentPoints(String),
    DecrementStudentPoints(String),
}
