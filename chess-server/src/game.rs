//! 对局控制
//!
//! 包含计时器系统

use std::time::Instant;

use protocol::{Side, INITIAL_TIME_MS};

/// 游戏计时器
#[derive(Debug)]
pub struct GameTimer {
    /// 红方剩余时间（毫秒）
    red_time_ms: u64,
    /// 黑方剩余时间（毫秒）
    black_time_ms: u64,
    /// 当前走子方
    current_turn: Side,
    /// 当前回合开始时间
    turn_start: Option<Instant>,
    /// 是否暂停
    paused: bool,
}

impl GameTimer {
    /// 创建新计时器（每方默认 10 分钟）
    pub fn new() -> Self {
        Self {
            red_time_ms: INITIAL_TIME_MS,
            black_time_ms: INITIAL_TIME_MS,
            current_turn: Side::Red,
            turn_start: Some(Instant::now()),
            paused: false,
        }
    }

    /// 创建自定义时间的计时器
    pub fn with_time(time_ms: u64) -> Self {
        Self {
            red_time_ms: time_ms,
            black_time_ms: time_ms,
            current_turn: Side::Red,
            turn_start: Some(Instant::now()),
            paused: false,
        }
    }

    /// 获取红方剩余时间（毫秒）
    pub fn red_time_ms(&self) -> u64 {
        if self.current_turn == Side::Red && !self.paused {
            self.calculate_remaining(self.red_time_ms)
        } else {
            self.red_time_ms
        }
    }

    /// 获取黑方剩余时间（毫秒）
    pub fn black_time_ms(&self) -> u64 {
        if self.current_turn == Side::Black && !self.paused {
            self.calculate_remaining(self.black_time_ms)
        } else {
            self.black_time_ms
        }
    }

    /// 计算剩余时间
    fn calculate_remaining(&self, base_time: u64) -> u64 {
        if let Some(start) = self.turn_start {
            let elapsed = start.elapsed().as_millis() as u64;
            base_time.saturating_sub(elapsed)
        } else {
            base_time
        }
    }

    /// 切换走子方
    pub fn switch_turn(&mut self) {
        // 更新当前方的剩余时间
        match self.current_turn {
            Side::Red => {
                self.red_time_ms = self.red_time_ms();
            }
            Side::Black => {
                self.black_time_ms = self.black_time_ms();
            }
        }

        // 切换到对方
        self.current_turn = self.current_turn.opponent();
        self.turn_start = Some(Instant::now());
    }

    /// 暂停计时器
    pub fn pause(&mut self) {
        if !self.paused {
            // 保存当前剩余时间
            match self.current_turn {
                Side::Red => {
                    self.red_time_ms = self.red_time_ms();
                }
                Side::Black => {
                    self.black_time_ms = self.black_time_ms();
                }
            }
            self.turn_start = None;
            self.paused = true;
        }
    }

    /// 恢复计时器
    pub fn resume(&mut self) {
        if self.paused {
            self.turn_start = Some(Instant::now());
            self.paused = false;
        }
    }

    /// 停止计时器
    pub fn stop(&mut self) {
        self.pause();
    }

    /// 检查是否超时
    pub fn is_timeout(&self, side: Side) -> bool {
        match side {
            Side::Red => self.red_time_ms() == 0,
            Side::Black => self.black_time_ms() == 0,
        }
    }

    /// 获取当前走子方
    pub fn current_turn(&self) -> Side {
        self.current_turn
    }

    /// 是否暂停
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// 设置剩余时间（用于重连恢复）
    pub fn set_times(&mut self, red_time_ms: u64, black_time_ms: u64) {
        self.red_time_ms = red_time_ms;
        self.black_time_ms = black_time_ms;
    }

    /// 重置当前回合开始时间（用于断线重连）
    pub fn reset_turn_start(&mut self) {
        if !self.paused {
            self.turn_start = Some(Instant::now());
        }
    }
}

impl Default for GameTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_timer_initial() {
        let timer = GameTimer::new();
        assert_eq!(timer.red_time_ms(), INITIAL_TIME_MS);
        assert_eq!(timer.black_time_ms(), INITIAL_TIME_MS);
        assert_eq!(timer.current_turn(), Side::Red);
    }

    #[test]
    fn test_timer_switch() {
        let mut timer = GameTimer::new();
        
        // 等待足够长时间以确保时间变化可测量（避免高负载系统上的 flaky）
        thread::sleep(Duration::from_millis(200));
        
        // 红方时间应该减少
        let red_time = timer.red_time_ms();
        assert!(red_time < INITIAL_TIME_MS);
        
        // 切换到黑方
        timer.switch_turn();
        assert_eq!(timer.current_turn(), Side::Black);
        
        // 红方时间固定
        let red_time_after = timer.red_time_ms();
        thread::sleep(Duration::from_millis(100));
        assert_eq!(timer.red_time_ms(), red_time_after);
    }

    #[test]
    fn test_timer_pause_resume() {
        let mut timer = GameTimer::new();
        
        thread::sleep(Duration::from_millis(200));
        timer.pause();
        
        let time_at_pause = timer.red_time_ms();
        thread::sleep(Duration::from_millis(200));
        
        // 暂停期间时间不变
        assert_eq!(timer.red_time_ms(), time_at_pause);
        
        timer.resume();
        thread::sleep(Duration::from_millis(200));
        
        // 恢复后时间继续减少
        assert!(timer.red_time_ms() < time_at_pause);
    }
}
