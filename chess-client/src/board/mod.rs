//! 棋盘渲染模块
//!
//! 负责棋盘和棋子的渲染

mod render;
pub mod pieces;

pub use render::*;
pub use pieces::*;

use bevy::prelude::*;

use crate::settings::GameSettings;
use crate::GameState;

/// 棋盘插件
pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BoardLayout::default())
            .add_systems(OnEnter(GameState::Playing), setup_board)
            .add_systems(OnExit(GameState::Playing), cleanup_board)
            .add_systems(
                Update,
                (
                    update_board_on_layout_change,
                    update_pieces,
                    update_highlights,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// 棋盘布局配置
#[derive(Resource, Clone, Debug)]
pub struct BoardLayout {
    /// 棋盘左上角位置 (屏幕坐标)
    pub origin: Vec2,
    /// 格子大小
    pub cell_size: f32,
    /// 棋子半径
    pub piece_radius: f32,
    /// 棋盘宽度 (9列)
    pub width: f32,
    /// 棋盘高度 (10行)
    pub height: f32,
}

impl Default for BoardLayout {
    fn default() -> Self {
        let cell_size = 60.0;
        Self {
            origin: Vec2::new(-240.0, -270.0), // 居中显示
            cell_size,
            piece_radius: cell_size * 0.42,
            width: cell_size * 8.0,  // 8个间隔
            height: cell_size * 9.0, // 9个间隔
        }
    }
}

impl BoardLayout {
    /// 根据窗口大小计算布局
    pub fn from_window_size(width: f32, height: f32) -> Self {
        // 留出边距给 UI
        let available_width = width * 0.6;
        let available_height = height * 0.9;

        // 棋盘比例 8:9 (宽:高)
        let board_ratio = 8.0 / 9.0;
        let available_ratio = available_width / available_height;

        let cell_size = if available_ratio > board_ratio {
            // 高度受限
            available_height / 9.0
        } else {
            // 宽度受限
            available_width / 8.0
        };

        let board_width = cell_size * 8.0;
        let board_height = cell_size * 9.0;

        // 棋盘居中偏左（右侧留给 UI）
        let origin_x = -width * 0.1 - board_width / 2.0;
        let origin_y = -board_height / 2.0;

        Self {
            origin: Vec2::new(origin_x, origin_y),
            cell_size,
            piece_radius: cell_size * 0.42,
            width: board_width,
            height: board_height,
        }
    }

    /// 将棋盘坐标转换为屏幕坐标
    pub fn board_to_screen(&self, x: u8, y: u8) -> Vec2 {
        Vec2::new(
            self.origin.x + x as f32 * self.cell_size,
            self.origin.y + y as f32 * self.cell_size,
        )
    }

    /// 将屏幕坐标转换为棋盘坐标
    pub fn screen_to_board(&self, pos: Vec2) -> Option<(u8, u8)> {
        let relative = pos - self.origin;
        let x = (relative.x / self.cell_size + 0.5).floor() as i32;
        let y = (relative.y / self.cell_size + 0.5).floor() as i32;

        if (0..=8).contains(&x) && (0..=9).contains(&y) {
            Some((x as u8, y as u8))
        } else {
            None
        }
    }
}

/// 棋盘标记组件
#[derive(Component)]
pub struct BoardMarker;

/// 设置棋盘
fn setup_board(
    mut commands: Commands,
    layout: Res<BoardLayout>,
    theme: Res<crate::theme::ColorTheme>,
) {
    render::spawn_board(&mut commands, &layout, &theme);
}

/// 清理棋盘
fn cleanup_board(mut commands: Commands, query: Query<Entity, With<BoardMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 当布局变化时重新渲染棋盘
fn update_board_on_layout_change(
    mut commands: Commands,
    layout: Res<BoardLayout>,
    theme: Res<crate::theme::ColorTheme>,
    game: Res<crate::game::ClientGame>,
    settings: Res<GameSettings>,
    board_query: Query<Entity, With<BoardMarker>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !layout.is_changed() {
        return;
    }

    // 清除所有棋盘相关实体
    for entity in board_query.iter() {
        commands.entity(entity).despawn();
    }

    // 重新生成棋盘
    render::spawn_board(&mut commands, &layout, &theme);

    // 重新生成棋子
    if let Some(state) = &game.game_state {
        pieces::spawn_pieces(
            &mut commands,
            &state.board,
            &layout,
            &theme,
            &asset_server,
            settings.piece_shape,
            &mut meshes,
            &mut materials,
        );
    }

    // 重新生成高亮
    if let Some(pos) = game.selected_piece {
        render::spawn_highlight(
            &mut commands,
            &layout,
            pos,
            theme.selected_highlight,
            HighlightType::Selected,
        );
    }

    for &pos in &game.valid_moves {
        render::spawn_highlight(
            &mut commands,
            &layout,
            pos,
            theme.valid_move_indicator,
            HighlightType::ValidMove,
        );
    }

    if let Some((from, to)) = game.last_move {
        render::spawn_highlight(
            &mut commands,
            &layout,
            from,
            theme.last_move_highlight,
            HighlightType::LastMove,
        );
        render::spawn_highlight(
            &mut commands,
            &layout,
            to,
            theme.last_move_highlight,
            HighlightType::LastMove,
        );
    }
}

/// 更新棋子显示
fn update_pieces(
    mut commands: Commands,
    game: Res<crate::game::ClientGame>,
    layout: Res<BoardLayout>,
    theme: Res<crate::theme::ColorTheme>,
    settings: Res<GameSettings>,
    pieces_query: Query<Entity, With<PieceMarker>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // 布局变化时由 update_board_on_layout_change 处理
    if layout.is_changed() {
        return;
    }

    if !game.is_changed() && !settings.is_changed() {
        return;
    }

    // 清除旧棋子
    for entity in pieces_query.iter() {
        commands.entity(entity).despawn();
    }

    // 生成新棋子
    if let Some(state) = &game.game_state {
        pieces::spawn_pieces(
            &mut commands,
            &state.board,
            &layout,
            &theme,
            &asset_server,
            settings.piece_shape,
            &mut meshes,
            &mut materials,
        );
    }
}

/// 更新高亮显示
fn update_highlights(
    mut commands: Commands,
    game: Res<crate::game::ClientGame>,
    layout: Res<BoardLayout>,
    theme: Res<crate::theme::ColorTheme>,
    highlights_query: Query<Entity, With<HighlightMarker>>,
) {
    // 布局变化时由 update_board_on_layout_change 处理
    if layout.is_changed() {
        return;
    }
    
    if !game.is_changed() {
        return;
    }

    // 清除旧高亮
    for entity in highlights_query.iter() {
        commands.entity(entity).despawn();
    }

    // 生成选中高亮
    if let Some(pos) = game.selected_piece {
        render::spawn_highlight(
            &mut commands,
            &layout,
            pos,
            theme.selected_highlight,
            HighlightType::Selected,
        );
    }

    // 生成合法落点高亮
    for &pos in &game.valid_moves {
        render::spawn_highlight(
            &mut commands,
            &layout,
            pos,
            theme.valid_move_indicator,
            HighlightType::ValidMove,
        );
    }

    // 生成最后走子高亮
    if let Some((from, to)) = game.last_move {
        render::spawn_highlight(
            &mut commands,
            &layout,
            from,
            theme.last_move_highlight,
            HighlightType::LastMove,
        );
        render::spawn_highlight(
            &mut commands,
            &layout,
            to,
            theme.last_move_highlight,
            HighlightType::LastMove,
        );
    }
}

/// 高亮标记组件
#[derive(Component)]
pub struct HighlightMarker;

/// 高亮类型
#[derive(Clone, Copy, Debug)]
pub enum HighlightType {
    Selected,
    ValidMove,
    LastMove,
}
