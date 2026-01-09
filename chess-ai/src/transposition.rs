//! 置换表
//!
//! 用于缓存已搜索过的局面，避免重复计算

use std::sync::atomic::{AtomicU64, Ordering};

/// 置换表条目类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    /// 精确值
    Exact,
    /// 下界（Alpha 截断）
    LowerBound,
    /// 上界（Beta 截断）
    UpperBound,
}

/// 置换表条目
/// 
/// 使用紧凑的布局以节省内存
/// 总共 16 字节
#[derive(Debug, Clone, Copy)]
pub struct TTEntry {
    /// Zobrist 哈希的高 32 位（用于验证）
    pub key: u32,
    /// 评估分数
    pub score: i16,
    /// 搜索深度
    pub depth: u8,
    /// 条目类型
    pub entry_type: EntryType,
    /// 最佳走法（编码为 u16: from_y << 12 | from_x << 8 | to_y << 4 | to_x）
    pub best_move: u16,
    /// 年龄（用于替换策略）
    pub age: u8,
}

impl TTEntry {
    /// 创建新条目
    pub fn new(
        key: u32,
        score: i32,
        depth: u8,
        entry_type: EntryType,
        best_move: Option<(u8, u8, u8, u8)>, // (from_x, from_y, to_x, to_y)
        age: u8,
    ) -> Self {
        let score = score.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        let best_move = best_move
            .map(|(fx, fy, tx, ty)| {
                ((fy as u16) << 12) | ((fx as u16) << 8) | ((ty as u16) << 4) | (tx as u16)
            })
            .unwrap_or(0);
        
        Self {
            key,
            score,
            depth,
            entry_type,
            best_move,
            age,
        }
    }
    
    /// 解码最佳走法
    pub fn decode_move(&self) -> Option<(u8, u8, u8, u8)> {
        if self.best_move == 0 {
            return None;
        }
        let from_y = ((self.best_move >> 12) & 0xF) as u8;
        let from_x = ((self.best_move >> 8) & 0xF) as u8;
        let to_y = ((self.best_move >> 4) & 0xF) as u8;
        let to_x = (self.best_move & 0xF) as u8;
        Some((from_x, from_y, to_x, to_y))
    }
}

/// 置换表
/// 
/// 使用固定大小的哈希表，支持线程安全的并发访问
pub struct TranspositionTable {
    /// 条目数组
    entries: Vec<Option<TTEntry>>,
    /// 表大小（条目数）
    size: usize,
    /// 当前年龄
    age: AtomicU64,
    /// 命中次数
    hits: AtomicU64,
    /// 查询次数
    probes: AtomicU64,
}

impl TranspositionTable {
    /// 创建指定大小的置换表
    /// 
    /// # Arguments
    /// * `size_mb` - 表大小（MB）
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let size = (size_mb * 1024 * 1024) / entry_size;
        
        Self {
            entries: vec![None; size],
            size,
            age: AtomicU64::new(0),
            hits: AtomicU64::new(0),
            probes: AtomicU64::new(0),
        }
    }
    
    /// 创建默认大小（64MB）的置换表
    pub fn default_size() -> Self {
        Self::new(64)
    }
    
    /// 计算索引
    #[inline]
    fn index(&self, hash: u64) -> usize {
        (hash as usize) % self.size
    }
    
    /// 提取验证键
    #[inline]
    fn verification_key(hash: u64) -> u32 {
        (hash >> 32) as u32
    }
    
    /// 查询条目
    pub fn probe(&self, hash: u64) -> Option<&TTEntry> {
        self.probes.fetch_add(1, Ordering::Relaxed);
        
        let index = self.index(hash);
        let key = Self::verification_key(hash);
        
        if let Some(ref entry) = self.entries[index] {
            if entry.key == key {
                self.hits.fetch_add(1, Ordering::Relaxed);
                return Some(entry);
            }
        }
        
        None
    }
    
    /// 存储条目
    pub fn store(
        &mut self,
        hash: u64,
        score: i32,
        depth: u8,
        entry_type: EntryType,
        best_move: Option<(u8, u8, u8, u8)>,
    ) {
        let index = self.index(hash);
        let key = Self::verification_key(hash);
        let age = self.age.load(Ordering::Relaxed) as u8;
        
        // 替换策略：
        // 1. 空槽直接写入
        // 2. 新条目深度更大时替换
        // 3. 旧条目年龄不同时替换
        let should_replace = match &self.entries[index] {
            None => true,
            Some(existing) => {
                existing.age != age || depth >= existing.depth
            }
        };
        
        if should_replace {
            self.entries[index] = Some(TTEntry::new(key, score, depth, entry_type, best_move, age));
        }
    }
    
    /// 增加年龄（每次新搜索时调用）
    pub fn new_search(&self) {
        self.age.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 清空表
    pub fn clear(&mut self) {
        self.entries.fill(None);
        self.hits.store(0, Ordering::Relaxed);
        self.probes.store(0, Ordering::Relaxed);
    }
    
    /// 获取命中率
    pub fn hit_rate(&self) -> f64 {
        let probes = self.probes.load(Ordering::Relaxed);
        let hits = self.hits.load(Ordering::Relaxed);
        if probes == 0 {
            0.0
        } else {
            hits as f64 / probes as f64
        }
    }
    
    /// 获取使用率
    pub fn usage(&self) -> f64 {
        let used = self.entries.iter().filter(|e| e.is_some()).count();
        used as f64 / self.size as f64
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> TTStats {
        TTStats {
            size_mb: (self.size * std::mem::size_of::<Option<TTEntry>>()) / (1024 * 1024),
            entries: self.size,
            used: self.entries.iter().filter(|e| e.is_some()).count(),
            hits: self.hits.load(Ordering::Relaxed),
            probes: self.probes.load(Ordering::Relaxed),
        }
    }
}

/// 置换表统计信息
#[derive(Debug, Clone)]
pub struct TTStats {
    pub size_mb: usize,
    pub entries: usize,
    pub used: usize,
    pub hits: u64,
    pub probes: u64,
}

impl TTStats {
    pub fn hit_rate(&self) -> f64 {
        if self.probes == 0 {
            0.0
        } else {
            self.hits as f64 / self.probes as f64
        }
    }
    
    pub fn usage(&self) -> f64 {
        self.used as f64 / self.entries as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tt_store_and_probe() {
        let mut tt = TranspositionTable::new(1); // 1MB
        
        let hash = 0x1234567890ABCDEF_u64;
        tt.store(hash, 100, 5, EntryType::Exact, Some((1, 2, 3, 4)));
        
        let entry = tt.probe(hash);
        assert!(entry.is_some());
        
        let entry = entry.unwrap();
        assert_eq!(entry.score, 100);
        assert_eq!(entry.depth, 5);
        assert_eq!(entry.entry_type, EntryType::Exact);
        assert_eq!(entry.decode_move(), Some((1, 2, 3, 4)));
    }
    
    #[test]
    fn test_tt_miss() {
        let tt = TranspositionTable::new(1);
        
        let entry = tt.probe(0x1234567890ABCDEF);
        assert!(entry.is_none());
    }
    
    #[test]
    fn test_tt_replacement() {
        let mut tt = TranspositionTable::new(1);
        
        let hash = 0x1234567890ABCDEF_u64;
        
        // 存储深度 3 的条目
        tt.store(hash, 50, 3, EntryType::Exact, None);
        
        // 用深度 5 的条目替换
        tt.store(hash, 100, 5, EntryType::Exact, None);
        
        let entry = tt.probe(hash).unwrap();
        assert_eq!(entry.depth, 5);
        assert_eq!(entry.score, 100);
    }
    
    #[test]
    fn test_move_encoding() {
        let entry = TTEntry::new(0, 0, 0, EntryType::Exact, Some((8, 9, 4, 5)), 0);
        let decoded = entry.decode_move();
        assert_eq!(decoded, Some((8, 9, 4, 5)));
    }
}
