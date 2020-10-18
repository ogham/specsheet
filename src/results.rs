use derive_more::AddAssign;
use serde::Serialize;


#[derive(Debug, Serialize)]
pub struct ResultsSection {
    pub check_outputs: Vec<CheckOutput>,
    pub totals: Stats,
}

#[derive(Debug, Serialize)]
pub struct CheckOutput {
    pub passed: bool,
    pub message: String,
    pub results: Vec<ResultMessage>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "state", content = "message", rename_all = "lowercase")]
pub enum ResultMessage {
    Passed(String),
    Failed(String),
    Error(String),
}

#[derive(Debug, Default, Copy, Clone, Serialize, AddAssign)]
pub struct Stats {
    pub check_count: u32,
    pub pass_count: u32,
    pub fail_count: u32,
    pub err_count: u32,
}


impl ResultsSection {
    pub fn failed(&self) -> bool {
        self.totals.fail_count > 0 || self.totals.err_count > 0
    }
}

