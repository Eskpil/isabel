use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SidecarRequestPopup {
    pub name: String,
    pub anchor_x: usize,
    pub anchor_y: usize,
    pub anchor_width: usize,
    pub anchor_height: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "args")]
pub enum SidecarRequest {
    #[serde(rename = "request_popup")]
    Popup(SidecarRequestPopup),
}
