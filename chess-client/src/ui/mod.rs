//! UI 模块
//!
//! 主菜单、游戏界面等 UI 组件

mod menu;
mod game_ui;
mod settings_ui;
mod lobby_ui;
mod saved_games_ui;
mod analysis_ui;

pub use menu::*;
pub use game_ui::*;
pub use settings_ui::*;
pub use lobby_ui::*;
pub use saved_games_ui::*;
pub use analysis_ui::*;

use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

use crate::settings::GameSettings;
use crate::GameState;

/// 基准分辨率（UI 设计基准）
const BASE_RESOLUTION: (f32, f32) = (1280.0, 720.0);

/// UI 插件
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // 添加帧率诊断插件
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            // 添加保存棋局列表资源
            .init_resource::<SavedGamesList>()
            // 添加 AI 分析状态资源
            .init_resource::<AiAnalysisState>()
            // UI 缩放系统（全局运行）
            .add_systems(Update, (update_ui_scale, update_fps_display))
            // 启动时创建 FPS 显示
            .add_systems(Startup, setup_fps_display)
            // 主菜单
            .add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(OnExit(GameState::Menu), cleanup_menu)
            .add_systems(Update, handle_menu_buttons.run_if(in_state(GameState::Menu)))
            // 保存的棋局列表
            .add_systems(OnEnter(GameState::SavedGames), setup_saved_games)
            .add_systems(OnExit(GameState::SavedGames), cleanup_saved_games)
            .add_systems(
                Update,
                (handle_saved_games_buttons, update_saved_games_list)
                    .run_if(in_state(GameState::SavedGames)),
            )
            // 大厅（房间列表）
            .add_systems(OnEnter(GameState::Lobby), setup_lobby)
            .add_systems(OnExit(GameState::Lobby), cleanup_lobby)
            .add_systems(
                Update,
                (handle_lobby_buttons, update_room_list).run_if(in_state(GameState::Lobby)),
            )
            // 设置页面
            .add_systems(OnEnter(GameState::Settings), setup_settings)
            .add_systems(OnExit(GameState::Settings), cleanup_settings)
            .add_systems(
                Update,
                (handle_settings_buttons, update_settings_display, update_tab_content, update_tab_buttons)
                    .run_if(in_state(GameState::Settings)),
            )
            // 游戏 UI
            .add_systems(OnEnter(GameState::Playing), setup_game_ui)
            .add_systems(OnExit(GameState::Playing), cleanup_game_ui)
            .add_systems(
                Update,
                (update_timer_display, update_move_history, update_pause_button_text, handle_game_buttons, update_ai_thinking_indicator, handle_move_history_scroll)
                    .run_if(in_state(GameState::Playing)),
            )
            // 游戏结束
            .add_systems(OnEnter(GameState::GameOver), setup_game_over_ui)
            .add_systems(OnExit(GameState::GameOver), cleanup_game_over_ui)
            .add_systems(Update, (handle_game_over_buttons, handle_analysis_buttons, handle_analysis_scroll).run_if(in_state(GameState::GameOver)));

        // LLM 分析轮询系统（仅在启用 llm feature 时注册）
        #[cfg(feature = "llm")]
        app.add_systems(Update, poll_ai_analysis_task.run_if(in_state(GameState::GameOver)));
    }
}

/// FPS 显示标记
#[derive(Component)]
pub struct FpsDisplay;

/// 创建 FPS 显示 UI
fn setup_fps_display(mut commands: Commands) {
    commands.spawn((
        Text::new("FPS: --"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        FpsDisplay,
    ));
}

/// 更新 FPS 显示
fn update_fps_display(
    settings: Res<GameSettings>,
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<(&mut Text, &mut Visibility), With<FpsDisplay>>,
) {
    for (mut text, mut visibility) in &mut query {
        // 根据设置控制显示/隐藏
        *visibility = if settings.show_fps {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        // 只在可见时更新文本
        if settings.show_fps {
            if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(value) = fps.smoothed() {
                    **text = format!("FPS: {:.0}", value);
                }
            }
        }
    }
}

/// 根据窗口大小动态调整 UI 缩放
fn update_ui_scale(windows: Query<&Window>, mut ui_scale: ResMut<UiScale>) {
    let Ok(window) = windows.single() else {
        return;
    };

    // 计算缩放比例：取宽高比例的较小值，确保 UI 不会超出屏幕
    let scale_x = window.width() / BASE_RESOLUTION.0;
    let scale_y = window.height() / BASE_RESOLUTION.1;
    let scale = scale_x.min(scale_y).max(0.5); // 最小缩放 0.5，避免太小

    // 只在缩放变化时更新，避免不必要的重绘
    if (ui_scale.0 - scale).abs() > 0.01 {
        ui_scale.0 = scale;
    }
}

/// UI 标记组件
#[derive(Component)]
pub struct UiMarker;

/// 菜单标记
#[derive(Component)]
pub struct MenuMarker;

/// 游戏 UI 标记
#[derive(Component)]
pub struct GameUiMarker;

/// 按钮类型
#[derive(Component, Clone, Debug)]
pub enum ButtonAction {
    // 主菜单
    CreatePvPRoom,
    JoinRoom,
    QuickMatch,
    PlayVsAi(protocol::Difficulty),
    LoadGame,
    Settings,
    ExitGame,
    // 游戏中
    Undo,
    Resign,
    Pause,
    Resume,
    SaveGame,
    // 游戏结束
    BackToMenu,
    PlayAgain,
    ExportGame,
    AiAnalysis,  // AI 复盘分析
    // AI 分析界面
    CloseAnalysis,
    SaveAnalysisReport,
    // 大厅
    RefreshRooms,
    BackToMenuFromLobby,
    JoinRoomById(protocol::RoomId),
}

/// 通用按钮样式
pub fn button_style() -> Node {
    Node {
        width: Val::Px(200.0),
        height: Val::Px(50.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        margin: UiRect::all(Val::Px(5.0)),
        ..default()
    }
}

/// 按钮颜色
pub const NORMAL_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const HOVERED_BUTTON: Color = Color::srgb(0.35, 0.35, 0.35);
pub const PRESSED_BUTTON: Color = Color::srgb(0.45, 0.45, 0.45);
