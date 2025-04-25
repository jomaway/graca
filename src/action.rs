use std::path::PathBuf;

use crossterm::event::KeyEvent;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::app::SelectedTab;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    User(UserEvent),
    ProcessCommand(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum UserEvent {
    IncrementThreshold(usize),
    DecrementThreshold(usize),
    SetMaxPoints(u16),
    SetScale(usize),
    LoadStudentList(PathBuf),
    ExportTo(PathBuf),
    SwitchTab(SelectedTab),
    Table(TableEvents),
    StudentResults(ExamResultEvents),
    IncrementPoints(usize),
    DecrementPoints(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum TableEvents {
    FirstRow,
    LastRow,
    NextRow,
    PrevRow,
}

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum ExamResultEvents {
    AddStudent(String),
    RenameStudent(String),
}
