//! UI 模块
//!
//! 主菜单、游戏界面等 UI 组件

mod menu;
mod game_ui;

pub use menu::*;
pub use game_ui::*;

use bevy::prelude::*;

use crate::GameState;

/// UI 插件
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // 主菜单
            .add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(OnExit(GameState::Menu), cleanup_menu)
            .add_systems(Update, handle_menu_buttons.run_if(in_state(GameState::Menu)))
            // 游戏 UI
            .add_systems(OnEnter(GameState::Playing), setup_game_ui)
            .add_systems(OnExit(GameState::Playing), cleanup_game_ui)
            .add_systems(
                Update,
                (update_timer_display, update_move_history, update_pause_button_text, handle_game_buttons)
                    .run_if(in_state(GameState::Playing)),
            )
            // 游戏结束
            .add_systems(OnEnter(GameState::GameOver), setup_game_over_ui)
            .add_systems(OnExit(GameState::GameOver), cleanup_game_over_ui)
            .add_systems(Update, handle_game_over_buttons.run_if(in_state(GameState::GameOver)));
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
    PlayVsAi(protocol::Difficulty),
    LoadGame,
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
