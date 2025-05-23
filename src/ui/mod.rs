use strum_macros::EnumIter;

pub mod report_tab;
pub mod scale_tab;
pub mod students_tab;
pub mod theme;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum AppTab {
    #[default]
    Scale,
    Result,
    Report,
}

impl AppTab {
    pub fn to_string(&self) -> String {
        match self {
            AppTab::Scale => "Scale [1]".into(),
            AppTab::Result => "Result [2]".into(),
            AppTab::Report => "Report [3]".into(),
        }
    }
}
