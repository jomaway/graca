use strum_macros::EnumIter;

pub mod exam_result_table;
pub mod grading_scale_table;
pub mod report_tab;
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

// pub struct View {
//     scale_table: GradingScaleTable,
//     result_table: ExamResultTable,
//     exam_chart: ExamChart,
//     input_field: Input,
//     status_msg: Option<String>,
//     selected_tab: AppTab,
// }
