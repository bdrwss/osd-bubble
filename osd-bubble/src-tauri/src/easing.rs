//! 缓动函数模块
//! 为气泡动画与后续 UI 动画提供统一的缓动曲线。
//! 所有函数满足：输入输出均在 [0, 1] 区间（ease_out_back 除外，允许轻微过冲）。

/// 线性插值：匀速运动
pub fn linear(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}

/// easeOutCubic：先快后慢，适合元素退场/淡出
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    1.0 - inv * inv * inv
}

/// easeInOutQuad：先慢后快再慢，适合对称的往返过渡
pub fn ease_in_out_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// easeOutBack：终点轻微过冲后回弹，适合强调性入场动画
/// 过冲系数 c1 = 1.70158（CSS ease-back 标准值）
pub fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    let inv = t - 1.0;
    1.0 + C3 * inv * inv * inv + C1 * inv * inv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundaries() {
        for f in [linear, ease_out_cubic, ease_in_out_quad] {
            assert!((f(0.0)).abs() < 1e-6, "f(0) should be 0");
            assert!((f(1.0) - 1.0).abs() < 1e-6, "f(1) should be 1");
        }
        // ease_out_back 边界同样收敛到 0 和 1
        assert!(ease_out_back(0.0).abs() < 1e-6);
        assert!((ease_out_back(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_input_clamped() {
        assert_eq!(linear(-0.5), 0.0);
        assert_eq!(linear(1.5), 1.0);
        assert_eq!(ease_out_cubic(-0.5), 0.0);
        assert!((ease_out_cubic(1.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_linear_midpoint() {
        assert!((linear(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_ease_out_cubic_monotonic_and_front_loaded() {
        let mut prev = 0.0;
        for i in 1..=10 {
            let t = i as f32 / 10.0;
            let v = ease_out_cubic(t);
            assert!(v > prev, "ease_out_cubic must be monotonically increasing");
            prev = v;
        }
        // 前 1/3 时长应完成超过 50% 的进度（先快后慢）
        assert!(ease_out_cubic(1.0 / 3.0) > 0.5);
    }

    #[test]
    fn test_ease_in_out_quad_monotonic_and_symmetric() {
        let mut prev = 0.0;
        for i in 1..=10 {
            let t = i as f32 / 10.0;
            let v = ease_in_out_quad(t);
            assert!(v > prev, "ease_in_out_quad must be monotonically increasing");
            prev = v;
        }
        assert!((ease_in_out_quad(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_ease_out_back_overshoots() {
        // 中段应出现超过 1.0 的过冲
        let mut overshoot = false;
        for i in 1..10 {
            let t = i as f32 / 10.0;
            if ease_out_back(t) > 1.0 {
                overshoot = true;
            }
        }
        assert!(overshoot, "ease_out_back should overshoot 1.0 mid-animation");
    }
}
