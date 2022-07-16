pub struct VCDStyle {
    pub background_color: String,
    pub trace_color: String,
    pub timeline_background_color: String,
    pub timeline_line_color: String,
    pub timeline_tick_color: String,
    pub signal_label_background_color: String,
    pub grid_lines: Option<String>,
}

impl VCDStyle {
    pub fn scansion() -> VCDStyle {
        Self {
            background_color: "#282828".into(),
            signal_label_background_color: "#e8e8e8".into(),
            timeline_background_color: "#f3f5de".into(),
            timeline_line_color: "#cbcbcb".into(),
            timeline_tick_color: "#000000".into(),
            trace_color: "#87ecd1".into(),
            grid_lines: None,
        }
    }
    pub fn gtkwave() -> VCDStyle {
        Self {
            background_color: "#000000".into(),
            signal_label_background_color: "#e8e8e8".into(),
            timeline_background_color: "#000000".into(),
            timeline_line_color: "#cbcbcb".into(),
            timeline_tick_color: "#FFFFFF".into(),
            trace_color: "#00ff00".into(),
            grid_lines: Some("#202070".into()),
        }
    }
}

