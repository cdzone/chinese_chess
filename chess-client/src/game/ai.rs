//! 本地 AI 系统
//!
//! 使用 Bevy 的 AsyncComputeTaskPool 在后台线程运行 AI 计算，不阻塞渲染

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use chess_ai::AiEngine;
use protocol::{BoardState, Difficulty, Move, Notation};
use std::time::{Duration, Instant};

use super::{ClientGame, GameMode};
use crate::settings::GameSettings;
use crate::GameState;

/// AI 计算任务组件
#[derive(Component)]
pub struct AiComputeTask {
    /// 异步任务
    task: Task<Option<Move>>,
    /// 开始时间
    started_at: Instant,
}

/// AI 思考状态（用于 UI 显示）
#[derive(Resource, Default)]
pub struct AiThinkingState {
    /// 是否正在思考
    pub is_thinking: bool,
    /// 思考开始时间
    pub started_at: Option<Instant>,
}

impl AiThinkingState {
    /// 获取思考时长（秒）
    pub fn elapsed_secs(&self) -> f32 {
        self.started_at
            .map(|t| t.elapsed().as_secs_f32())
            .unwrap_or(0.0)
    }
}

/// AI 走棋事件
#[derive(Event, Clone, Debug)]
pub struct AiMoveEvent {
    pub from: protocol::Position,
    pub to: protocol::Position,
}

/// AI 插件
pub struct LocalAiPlugin;

impl Plugin for LocalAiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AiThinkingState::default())
            .add_event::<AiMoveEvent>()
            .add_systems(
                Update,
                (
                    trigger_ai_compute,
                    poll_ai_result,
                    handle_ai_move,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            // 退出 Playing 状态时清理 AI 任务
            .add_systems(OnExit(GameState::Playing), cleanup_ai_tasks);
    }
}

/// 清理 AI 任务（退出游戏时）
fn cleanup_ai_tasks(
    mut commands: Commands,
    tasks: Query<Entity, With<AiComputeTask>>,
    mut thinking_state: ResMut<AiThinkingState>,
) {
    for entity in &tasks {
        commands.entity(entity).despawn();
    }
    thinking_state.is_thinking = false;
    thinking_state.started_at = None;
}

/// 触发 AI 计算
fn trigger_ai_compute(
    game: Res<ClientGame>,
    mut commands: Commands,
    existing_tasks: Query<&AiComputeTask>,
    mut thinking_state: ResMut<AiThinkingState>,
) {
    // 防止重复触发
    if !existing_tasks.is_empty() {
        return;
    }

    // 检查是否需要 AI 走棋
    if !game.should_ai_move() {
        if thinking_state.is_thinking {
            thinking_state.is_thinking = false;
            thinking_state.started_at = None;
        }
        return;
    }

    // 获取游戏状态和难度
    let game_state = match &game.game_state {
        Some(s) => s.clone(),
        None => return,
    };

    let difficulty = match &game.game_mode {
        Some(GameMode::LocalPvE { difficulty }) => *difficulty,
        _ => return,
    };

    // 更新思考状态
    thinking_state.is_thinking = true;
    thinking_state.started_at = Some(Instant::now());

    tracing::info!("AI 开始思考... 难度: {:?}", difficulty);

    // 在后台线程池中计算（不阻塞渲染）
    let task_pool = AsyncComputeTaskPool::get();
    let task = task_pool.spawn(async move {
        compute_ai_move(game_state, difficulty)
    });

    commands.spawn(AiComputeTask {
        task,
        started_at: Instant::now(),
    });
}

/// 计算 AI 走法（在后台线程运行）
fn compute_ai_move(game_state: BoardState, difficulty: Difficulty) -> Option<Move> {
    let mut engine = AiEngine::from_difficulty(difficulty);
    engine.search(&game_state)
}

/// 轮询 AI 结果（非阻塞）
fn poll_ai_result(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut AiComputeTask)>,
    mut ai_events: EventWriter<AiMoveEvent>,
    mut thinking_state: ResMut<AiThinkingState>,
    mut game: ResMut<ClientGame>,
    mut game_state: ResMut<NextState<GameState>>,
    settings: Res<GameSettings>,
) {
    let ai_timeout_secs = settings.ai_timeout_secs as u64;
    
    for (entity, mut task_component) in &mut tasks {
        // P2: 检查超时保护
        if task_component.started_at.elapsed() > Duration::from_secs(ai_timeout_secs) {
            tracing::error!("AI 计算超时（>{}秒），强制终止", ai_timeout_secs);

            // 设置 AI 失败结果（玩家获胜）
            let result = protocol::GameResult::RedWin(protocol::WinReason::Timeout);
            game.set_result(result);
            game_state.set(GameState::GameOver);

            commands.entity(entity).despawn();
            thinking_state.is_thinking = false;
            thinking_state.started_at = None;
            continue;
        }

        // P0 修复：使用 Task::is_finished() 检查任务是否完成，避免阻塞
        if !task_component.task.is_finished() {
            continue;
        }

        // 任务已完成，使用 block_on 获取结果（此时不会阻塞，因为任务已完成）
        let ai_move = bevy::tasks::block_on(&mut task_component.task);
        let elapsed = task_component.started_at.elapsed();

        // 更新思考状态
        thinking_state.is_thinking = false;
        thinking_state.started_at = None;

        // P2: 检查暂停状态，丢弃结果
        if game.is_paused {
            tracing::debug!("AI 计算完成，但游戏已暂停，丢弃结果");
            commands.entity(entity).despawn();
            continue;
        }

        match ai_move {
            Some(mv) => {
                tracing::info!("AI 走棋: {:?} -> {:?}, 耗时: {:?}", mv.from, mv.to, elapsed);

                // 发送 AI 走棋事件
                ai_events.send(AiMoveEvent {
                    from: mv.from,
                    to: mv.to,
                });
            }
            None => {
                // P1 修复：AI 无合法走法，触发游戏结束
                tracing::warn!("AI 无法找到合法走法，玩家胜利");
                let result = protocol::GameResult::RedWin(protocol::WinReason::Checkmate);
                game.set_result(result);
                game_state.set(GameState::GameOver);
            }
        }

        // 清理任务实体
        commands.entity(entity).despawn();
    }
}

/// 处理 AI 走棋事件
fn handle_ai_move(
    mut ai_events: EventReader<AiMoveEvent>,
    mut game: ResMut<ClientGame>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for event in ai_events.read() {
        // 本地模式：直接更新游戏状态
        if let Some(state) = &game.game_state {
            let mv = Move::new(event.from, event.to);

            // 验证走法合法性
            let legal_moves = protocol::MoveGenerator::generate_legal(state);
            if !legal_moves.iter().any(|m| m.from == mv.from && m.to == mv.to) {
                tracing::error!("AI 走法不合法: {:?}", mv);
                return;
            }

            // 执行走法
            let mut new_state = state.clone();
            new_state.board.move_piece(mv.from, mv.to);
            new_state.current_turn = new_state.current_turn.opponent();

            // 生成中文纵线表示法
            let notation = Notation::to_chinese(&state.board, &mv)
                .unwrap_or_else(|| format!("{:?}->{:?}", mv.from, mv.to));

            // 更新游戏状态
            game.update_state(new_state.clone(), mv.from, mv.to, notation);

            // P0 修复：检查游戏是否结束，并触发状态转换
            check_game_over(&mut game, &new_state, &mut game_state);
        }
    }
}

/// 检查游戏是否结束
fn check_game_over(
    game: &mut ResMut<ClientGame>,
    state: &BoardState,
    game_state: &mut ResMut<NextState<GameState>>,
) {
    use protocol::{GameResult, WinReason, MoveGenerator};

    // 检查是否将死（当前方无合法走法）
    let legal_moves = MoveGenerator::generate_legal(state);
    if legal_moves.is_empty() {
        // 如果无合法走法，判定对方获胜
        let result = match state.current_turn {
            protocol::Side::Red => GameResult::BlackWin(WinReason::Checkmate),
            protocol::Side::Black => GameResult::RedWin(WinReason::Checkmate),
        };
        tracing::info!("游戏结束: {:?}", result);
        game.set_result(result);
        game_state.set(GameState::GameOver);
    }
}
