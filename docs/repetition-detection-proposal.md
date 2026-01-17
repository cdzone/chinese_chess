# AI 重复局面检测方案

> **状态：已实施** ✅  
> **实施日期：2026-01-12**

## 1. 问题分析

当前 AI 存在以下问题：
- `position_history` 字段已定义但未使用
- 搜索时不检测重复局面，可能导致 AI 陷入循环走法
- 没有按照中国象棋规则处理"三次重复判和"

### 现象描述

AI 在中局阶段可能会：
1. 反复走同一个棋子来回移动
2. 陷入两步或多步循环
3. 无法主动跳出循环寻找其他走法

### 根本原因

`chess-ai/src/search.rs` 中的 `alpha_beta` 函数：
- 置换表命中时直接返回缓存分数，不考虑路径重复
- 没有对重复局面给予惩罚分数
- 困毙返回 0 分的逻辑不适用于重复局面

## 2. 设计方案

### 2.1 核心思路

在 AI 搜索过程中维护一个**搜索路径哈希栈**，检测当前局面是否在路径中重复出现：
- 重复 1 次：给予轻微惩罚（避免无意义循环）
- 重复 2 次（即三次出现）：返回和棋分数 0

### 2.2 修改文件

| 文件 | 修改内容 |
|------|----------|
| `chess-ai/src/search.rs` | 添加路径哈希追踪，实现重复检测 |

### 2.3 具体实现

#### 修改 `AiEngine` 结构体

```rust
pub struct AiEngine {
    config: AiConfig,
    nodes_searched: u64,
    zobrist: ZobristTable,
    tt: TranspositionTable,
    // 新增：搜索路径哈希栈（用于检测重复局面）
    path_hashes: Vec<u64>,
}
```

#### 修改 `new` 构造函数

```rust
pub fn new(config: AiConfig) -> Self {
    let tt_size = config.tt_size_mb;
    Self {
        config,
        nodes_searched: 0,
        zobrist: ZobristTable::new(),
        tt: TranspositionTable::new(tt_size),
        path_hashes: Vec::with_capacity(64),  // 新增
    }
}
```

#### 修改 `search` 函数

```rust
pub fn search(&mut self, state: &BoardState) -> Option<Move> {
    self.nodes_searched = 0;
    self.tt.new_search();
    self.path_hashes.clear();  // 新增：清空路径栈
    
    let hash = self.zobrist.hash(&state.board, state.current_turn);
    self.path_hashes.push(hash);  // 新增：记录根节点
    
    // ... 其余逻辑保持不变 ...
}
```

#### 修改 `alpha_beta` 函数

```rust
fn alpha_beta(
    &mut self,
    state: &BoardState,
    hash: u64,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    deadline: &Instant,
) -> i32 {
    self.nodes_searched += 1;

    // ========== 新增：重复局面检测 ==========
    let repetition_count = self.path_hashes.iter().filter(|&&h| h == hash).count();
    if repetition_count >= 2 {
        // 三次重复（当前局面 + 路径中已有2次），返回和棋分数
        return 0;
    }
    // ========================================

    // 检查时间：超时时返回当前静态评估值
    if Instant::now() >= *deadline {
        return self.evaluate(state);
    }

    // 查询置换表
    if let Some(entry) = self.tt.probe(hash) {
        // ... 原有逻辑 ...
    }

    // ... 深度检查、走法生成等原有逻辑 ...

    for mv in moves {
        let mut new_state = state.clone();
        let captured = new_state.board.get(mv.to);
        new_state.board.move_piece(mv.from, mv.to);
        new_state.switch_turn();

        let new_hash = self.update_hash(hash, state, &mv, captured.map(|p| p.piece_type));
        
        // ========== 新增：入栈 ==========
        self.path_hashes.push(new_hash);
        // ================================
        
        let score = -self.alpha_beta(&new_state, new_hash, depth - 1, -beta, -alpha, deadline);
        
        // ========== 新增：出栈 ==========
        self.path_hashes.pop();
        // ================================

        if score >= beta {
            // Beta 剪枝
            self.tt.store(/* ... */);
            return beta;
        }
        if score > alpha {
            alpha = score;
            best_move = Some(mv);
            entry_type = EntryType::Exact;
        }
    }

    // ... 存储置换表等原有逻辑 ...
}
```

### 2.4 重复惩罚策略

| 重复次数 | 处理方式 | 说明 |
|----------|----------|------|
| 0 | 正常搜索 | 新局面 |
| 1 | 继续搜索 | 可接受的重复（如将军后回防） |
| ≥2 | 返回 0 | 三次重复判和 |

### 2.5 边界情况处理

1. **置换表与重复检测的交互**：
   - 置换表存储的是静态评估，不包含路径信息
   - 重复检测必须在置换表查询之前执行

2. **静态搜索（quiescence）**：
   - 静态搜索只处理吃子，一般不会产生重复
   - 暂不添加重复检测，保持性能

3. **根节点处理**：
   - 根节点的走法选择也应考虑重复惩罚
   - 在主搜索循环中同样使用 `path_hashes`

## 3. 测试计划

### 3.1 单元测试

```rust
#[test]
fn test_repetition_detection() {
    // 构造一个容易产生重复的局面
    let fen = "4k4/9/9/9/9/9/9/4R4/9/4K4 w 0 1";
    let state = Fen::parse(fen).unwrap();
    let mut engine = AiEngine::from_difficulty(Difficulty::Medium);
    
    // 模拟多次搜索，验证不会返回相同走法导致循环
    let mut moves = Vec::new();
    let mut current_state = state.clone();
    
    for _ in 0..6 {
        if let Some(mv) = engine.search(&current_state) {
            moves.push(mv);
            current_state.board.move_piece(mv.from, mv.to);
            current_state.switch_turn();
        }
    }
    
    // 验证没有简单的两步循环
    for i in 2..moves.len() {
        let is_simple_cycle = moves[i].from == moves[i-2].to 
                           && moves[i].to == moves[i-2].from;
        assert!(!is_simple_cycle, "不应该出现简单两步循环");
    }
}

#[test]
fn test_repetition_returns_draw_score() {
    // 测试三次重复返回和棋分数
    let state = BoardState::initial();
    let mut engine = AiEngine::from_difficulty(Difficulty::Easy);
    
    // 验证 path_hashes 正确维护
    let _ = engine.search(&state);
    // path_hashes 应该在搜索结束后只剩根节点
    assert_eq!(engine.path_hashes.len(), 1);
}
```

### 3.2 集成测试

手动测试场景：
1. 开局阶段：验证 AI 不会反复移动同一棋子
2. 中局阶段：验证 AI 在有多个选择时不会陷入循环
3. 残局阶段：验证 AI 能正确处理和棋局面

## 4. 风险评估

| 风险 | 影响程度 | 缓解措施 |
|------|----------|----------|
| 性能下降 | 中 | `path_hashes` 使用 Vec，查找是 O(n)，但搜索深度有限（≤6），影响可接受 |
| 误判重复 | 低 | Zobrist 哈希碰撞概率极低（64位哈希，约 1/2^64） |
| 过早判和 | 低 | 只在搜索路径内检测，不影响跨回合的正常重复 |

## 5. 后续优化（可选）

### 5.1 性能优化

如果搜索深度增加，可改用 `HashSet<u64>` 加速查找：

```rust
use std::collections::HashSet;

path_hashes: HashSet<u64>,
path_hashes_stack: Vec<u64>,  // 用于出栈
```

### 5.2 规则完善

按中国象棋正式规则，可进一步细化：
- **长将判负**：连续将军超过一定次数，将军方判负
- **长捉判负**：连续捉子超过一定次数，捉子方判负
- **一将一捉**：将军方判负

### 5.3 历史局面继承

将 `BoardState.position_history` 传入搜索，检测跨回合重复：

```rust
pub fn search(&mut self, state: &BoardState) -> Option<Move> {
    self.path_hashes.clear();
    // 继承历史局面
    self.path_hashes.extend(&state.position_history);
    // ...
}
```

## 6. 实施计划

| 阶段 | 内容 | 预计时间 |
|------|------|----------|
| 1 | 修改 `AiEngine` 结构体 | 5 分钟 |
| 2 | 修改 `search` 函数 | 5 分钟 |
| 3 | 修改 `alpha_beta` 函数 | 10 分钟 |
| 4 | 添加单元测试 | 10 分钟 |
| 5 | 集成测试验证 | 10 分钟 |

**总计：约 40 分钟**

---

## 审查清单

- [ ] 方案是否解决了 AI 循环走法的问题？
- [ ] 重复检测的时机是否正确（在置换表查询之前）？
- [ ] 性能影响是否可接受？
- [ ] 测试覆盖是否充分？
- [ ] 是否需要实现后续优化中的功能？

---

*文档版本：v1.0*  
*创建日期：2026-01-12*  
*作者：AI Assistant*
