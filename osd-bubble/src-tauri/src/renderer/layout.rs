pub struct LayoutNode {
    pub text: String,
    pub is_modifier: bool,
    pub is_multiplier: bool,
    pub width: f32,
    pub text_width: f32,
}

pub struct BubbleLayout {
    pub nodes: Vec<LayoutNode>,
    pub total_width: f32,
    pub total_height: f32,
}

impl BubbleLayout {
    pub fn build(keys: &[String]) -> Self {
        let mut nodes = Vec::new();
        let mut total_width = 0.0;
        let padding_x = 16.0;
        let spacing = 8.0;
        let height = 56.0;

        for (i, key) in keys.iter().enumerate() {
            let is_multiplier = key.starts_with('×');
            let is_mouse = key == "LeftClick" || key == "RightClick" || key == "MiddleClick" || key == "ScrollUp" || key == "ScrollDown";
            let is_modifier = !is_multiplier && !is_mouse && key.chars().count() > 1; 
            
            // 粗略估算文字宽度 (如果是乘数使用紧凑字宽，鼠标按键给予固定图标宽度)
            let text_width = if is_multiplier {
                key.chars().count() as f32 * 11.0
            } else if is_mouse {
                24.0
            } else {
                key.chars().count() as f32 * 14.0
            };
            
            // 如果是乘数胶囊微标，分配专属的微标宽度，其余按键保持键帽基准
            let width = if is_multiplier {
                (text_width + 16.0).max(36.0)
            } else {
                (text_width + 24.0).max(height)
            };
            
            nodes.push(LayoutNode {
                text: key.clone(),
                is_modifier,
                is_multiplier,
                width,
                text_width,
            });

            total_width += width;
            if i > 0 {
                total_width += spacing;
            }
        }

        // 加上最外层的 padding
        total_width += padding_x * 2.0;

        Self {
            nodes,
            total_width,
            total_height: height,
        }
    }
}
