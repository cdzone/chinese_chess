//! 中国象棋客户端
//!
//! 使用 Bevy 引擎实现的中国象棋游戏客户端

pub mod board;
pub mod game;
pub mod network;
pub mod theme;
pub mod ui;

use bevy::prelude::*;

/// 游戏状态
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    /// 主菜单
    #[default]
    Menu,
    /// 连接服务器
    Connecting,
    /// 房间列表/等待
    Lobby,
    /// 游戏中
    Playing,
    /// 游戏结束
    GameOver,
}

/// 客户端插件
pub struct ChessClientPlugin;

impl Plugin for ChessClientPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                theme::ThemePlugin,
                board::BoardPlugin,
                game::GamePlugin,
                ui::UiPlugin,
                network::NetworkPlugin,
            ));
    }
}
