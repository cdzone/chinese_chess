//! AI 复盘分析 UI
//!
//! 显示 LLM 生成的对局分析报告

use bevy::prelude::*;

use super::{ButtonAction, UiMarker, NORMAL_BUTTON, HOVERED_BUTTON, PRESSED_BUTTON};
use crate::icons;

/// AI 分析 UI 标记
#[derive(Component)]
pub struct AnalysisUiMarker;

/// AI 分析加载状态标记
#[derive(Component)]
pub struct AnalysisLoadingMarker;

/// AI 分析结果标记
#[derive(Component)]
pub struct AnalysisResultMarker;

/// AI 分析滚动容器标记
#[derive(Component)]
pub struct AnalysisScrollContainer;

/// AI 分析状态资源
#[derive(Resource, Default)]
pub struct AiAnalysisState {
    /// 是否正在分析
    pub is_analyzing: bool,
    /// 分析结果
    pub result: Option<chess_ai::llm::GameAnalysis>,
    /// 错误信息
    pub error: Option<String>,
}

/// 设置 AI 分析加载界面
pub fn setup_analysis_loading_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            UiMarker,
            AnalysisUiMarker,
            AnalysisLoadingMarker,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(50.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    BorderRadius::all(Val::Px(12.0)),
                ))
                .with_children(|parent| {
                    // 标题
                    parent.spawn((
                        Text::new("AI 复盘分析"),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(30.0)),
                            ..default()
                        },
                    ));

                    // 加载动画
                    parent.spawn((
                        Text::new(format!("{} AI 正在分析对局...", icons::INFO)),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.3)),
                        Node {
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },
                    ));

                    // 提示信息
                    parent.spawn((
                        Text::new("预计需要 10-30 秒"),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        Node {
                            margin: UiRect::bottom(Val::Px(30.0)),
                            ..default()
                        },
                    ));

                    // 取消按钮
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.4, 0.2, 0.2)),
                            BorderRadius::all(Val::Px(6.0)),
                            ButtonAction::CloseAnalysis,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("取消"),
                                TextFont {
                                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

/// 设置 AI 分析结果界面
pub fn setup_analysis_result_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    analysis_state: Res<AiAnalysisState>,
) {
    let Some(ref analysis) = analysis_state.result else {
        return;
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            UiMarker,
            AnalysisUiMarker,
            AnalysisResultMarker,
        ))
        .with_children(|parent| {
            // 主容器（可滚动）
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(700.0),
                        max_height: Val::Percent(90.0),
                        padding: UiRect::all(Val::Px(30.0)),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
                    BorderRadius::all(Val::Px(12.0)),
                    ScrollPosition::default(),
                ))
                .with_children(|parent| {
                    // 标题栏
                    spawn_title_bar(parent, &asset_server);

                    // 整体评分
                    spawn_overall_rating(parent, &asset_server, &analysis.overall_rating);

                    // 开局评价
                    spawn_opening_review(parent, &asset_server, &analysis.opening_review);

                    // 关键时刻
                    spawn_key_moments(parent, &asset_server, &analysis.key_moments);

                    // 残局评价
                    spawn_endgame_review(parent, &asset_server, &analysis.endgame_review);

                    // 不足与提升
                    spawn_weaknesses(parent, &asset_server, &analysis.weaknesses);

                    // 改进建议
                    spawn_suggestions(parent, &asset_server, &analysis.suggestions);

                    // 底部按钮
                    spawn_bottom_buttons(parent, &asset_server);
                });
        });
}

/// 生成标题栏
fn spawn_title_bar(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            margin: UiRect::bottom(Val::Px(20.0)),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("AI 复盘分析"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 关闭按钮
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(36.0),
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    BorderRadius::all(Val::Px(18.0)),
                    ButtonAction::CloseAnalysis,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(icons::CLOSE),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// 生成整体评分区域
fn spawn_overall_rating(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    rating: &chess_ai::llm::OverallRating,
) {
    spawn_section(parent, asset_server, "整体评分", |parent| {
        // 评分行
        spawn_rating_row(parent, asset_server, "红方棋力", rating.red_play_quality);
        spawn_rating_row(parent, asset_server, "黑方棋力", rating.black_play_quality);
        spawn_rating_row(parent, asset_server, "对局精彩度", rating.game_quality);

        // 总结
        parent.spawn((
            Text::new(&rating.summary),
            TextFont {
                font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node {
                margin: UiRect::top(Val::Px(10.0)),
                ..default()
            },
        ));
    });
}

/// 生成评分行
fn spawn_rating_row(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    label: &str,
    score: f32,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        })
        .with_children(|parent| {
            // 标签
            parent.spawn((
                Text::new(format!("{}: ", label)),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    width: Val::Px(100.0),
                    ..default()
                },
            ));

            // 星级
            let stars = chess_ai::llm::OverallRating::stars(score);
            parent.spawn((
                Text::new(stars),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.84, 0.0)),
            ));

            // 分数
            parent.spawn((
                Text::new(format!(" {:.1}/10", score)),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

/// 生成开局评价区域
fn spawn_opening_review(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    review: &chess_ai::llm::OpeningReview,
) {
    spawn_section(parent, asset_server, "开局评价", |parent| {
        if let Some(ref name) = review.name {
            parent.spawn((
                Text::new(format!("开局: {}", name)),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.7, 0.5)),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));
        }

        parent.spawn((
            Text::new(format!("评价: {}", review.evaluation)),
            TextFont {
                font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
        ));

        parent.spawn((
            Text::new(&review.comment),
            TextFont {
                font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
    });
}

/// 生成关键时刻区域
fn spawn_key_moments(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    moments: &[chess_ai::llm::KeyMoment],
) {
    spawn_section(parent, asset_server, "关键时刻", |parent| {
        if moments.is_empty() {
            parent.spawn((
                Text::new("无特别关键的时刻"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
            return;
        }

        for moment in moments {
            spawn_key_moment(parent, asset_server, moment);
        }
    });
}

/// 生成单个关键时刻
fn spawn_key_moment(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    moment: &chess_ai::llm::KeyMoment,
) {
    let (icon, type_color) = match moment.moment_type {
        chess_ai::llm::MomentType::Brilliant => (icons::BRILLIANT, Color::srgb(1.0, 0.84, 0.0)),
        chess_ai::llm::MomentType::Mistake => (icons::MISTAKE, Color::srgb(0.9, 0.3, 0.3)),
        chess_ai::llm::MomentType::TurningPoint => (icons::TURNING_POINT, Color::srgb(0.3, 0.7, 0.9)),
    };

    let side_name = match moment.side {
        chess_ai::llm::MomentSide::Red => "红方",
        chess_ai::llm::MomentSide::Black => "黑方",
    };

    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.5)),
            BorderRadius::all(Val::Px(6.0)),
        ))
        .with_children(|parent| {
            // 标题行
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(format!(
                            "{} 第{}步 {} {}",
                            icon, moment.move_number, side_name, moment.move_notation
                        )),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(type_color),
                    ));
                });

            // 类型标签
            parent.spawn((
                Text::new(format!("[{}]", moment.moment_type.display_name())),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(type_color),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));

            // 分析
            parent.spawn((
                Text::new(&moment.analysis),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
}

/// 生成残局评价区域
fn spawn_endgame_review(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    review: &chess_ai::llm::EndgameReview,
) {
    spawn_section(parent, asset_server, "残局评价", |parent| {
        parent.spawn((
            Text::new(format!("评价: {}", review.evaluation)),
            TextFont {
                font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
        ));

        parent.spawn((
            Text::new(&review.comment),
            TextFont {
                font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
    });
}

/// 生成不足与提升区域
fn spawn_weaknesses(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    weaknesses: &chess_ai::llm::Weaknesses,
) {
    // 如果双方都没有不足，不显示此区域
    if weaknesses.red.is_empty() && weaknesses.black.is_empty() {
        return;
    }

    spawn_section(parent, asset_server, "不足与提升", |parent| {
        // 红方不足
        if !weaknesses.red.is_empty() {
            parent.spawn((
                Text::new("红方不足:"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.5, 0.5)),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));

            for weakness in &weaknesses.red {
                parent.spawn((
                    Text::new(format!("• {}", weakness)),
                    TextFont {
                        font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::new(Val::Px(15.0), Val::Px(0.0), Val::Px(0.0), Val::Px(3.0)),
                        ..default()
                    },
                ));
            }
        }

        // 黑方不足
        if !weaknesses.black.is_empty() {
            parent.spawn((
                Text::new("黑方不足:"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(15.0), Val::Px(5.0)),
                    ..default()
                },
            ));

            for weakness in &weaknesses.black {
                parent.spawn((
                    Text::new(format!("• {}", weakness)),
                    TextFont {
                        font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::new(Val::Px(15.0), Val::Px(0.0), Val::Px(0.0), Val::Px(3.0)),
                        ..default()
                    },
                ));
            }
        }
    });
}

/// 生成改进建议区域
fn spawn_suggestions(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    suggestions: &chess_ai::llm::Suggestions,
) {
    spawn_section(parent, asset_server, "改进建议", |parent| {
        // 红方建议
        if !suggestions.red.is_empty() {
            parent.spawn((
                Text::new("给红方:"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.5, 0.5)),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));

            for suggestion in &suggestions.red {
                parent.spawn((
                    Text::new(format!("• {}", suggestion)),
                    TextFont {
                        font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::new(Val::Px(15.0), Val::Px(0.0), Val::Px(0.0), Val::Px(3.0)),
                        ..default()
                    },
                ));
            }
        }

        // 黑方建议
        if !suggestions.black.is_empty() {
            parent.spawn((
                Text::new("给黑方:"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(15.0), Val::Px(5.0)),
                    ..default()
                },
            ));

            for suggestion in &suggestions.black {
                parent.spawn((
                    Text::new(format!("• {}", suggestion)),
                    TextFont {
                        font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::new(Val::Px(15.0), Val::Px(0.0), Val::Px(0.0), Val::Px(3.0)),
                        ..default()
                    },
                ));
            }
        }

        if suggestions.red.is_empty() && suggestions.black.is_empty() {
            parent.spawn((
                Text::new("暂无具体建议"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        }
    });
}

/// 生成底部按钮
fn spawn_bottom_buttons(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(15.0),
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        })
        .with_children(|parent| {
            // 关闭按钮
            spawn_action_button(
                parent,
                asset_server,
                "关闭",
                ButtonAction::CloseAnalysis,
                Color::srgb(0.3, 0.3, 0.3),
            );
        });
}

/// 生成操作按钮
fn spawn_action_button(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    text: &str,
    action: ButtonAction,
    bg_color: Color,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(140.0),
                height: Val::Px(45.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg_color),
            BorderRadius::all(Val::Px(8.0)),
            action,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 生成区域容器
fn spawn_section<F>(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    title: &str,
    content: F,
) where
    F: FnOnce(&mut ChildSpawnerCommands),
{
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(15.0)),
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|parent| {
            // 区域标题
            parent.spawn((
                Text::new(format!("─ {} ─", title)),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.8, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ));

            content(parent);
        });
}

/// 清理分析 UI
pub fn cleanup_analysis_ui(mut commands: Commands, query: Query<Entity, With<AnalysisUiMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 处理分析界面按钮
pub fn handle_analysis_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut analysis_state: ResMut<AiAnalysisState>,
    mut commands: Commands,
    query: Query<Entity, With<AnalysisUiMarker>>,
) {
    for (interaction, mut color, action) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                match action {
                    ButtonAction::CloseAnalysis => {
                        // 清理分析状态
                        analysis_state.is_analyzing = false;
                        analysis_state.result = None;
                        analysis_state.error = None;

                        // 清理 UI
                        for entity in query.iter() {
                            commands.entity(entity).despawn();
                        }
                    }
                    ButtonAction::SaveAnalysisReport => {
                        // TODO: 保存分析报告到文件
                        tracing::info!("Save analysis report clicked");
                    }
                    _ => {}
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
