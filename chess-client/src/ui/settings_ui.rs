//! 设置页面 UI

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::winit::{UpdateMode, WinitSettings};

use crate::settings::GameSettings;
use crate::GameState;

use super::{button_style, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};

/// 设置页面标记
#[derive(Component)]
pub struct SettingsMarker;

/// 设置标签页
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
pub enum SettingsTab {
    #[default]
    Game,
    Display,
    Audio,
    Network,
    Advanced,
}

impl SettingsTab {
    pub fn display_name(&self) -> &'static str {
        match self {
            SettingsTab::Game => "游戏",
            SettingsTab::Display => "显示",
            SettingsTab::Audio => "音频",
            SettingsTab::Network => "网络",
            SettingsTab::Advanced => "高级",
        }
    }

    pub fn all() -> &'static [SettingsTab] {
        &[
            SettingsTab::Game,
            SettingsTab::Display,
            SettingsTab::Audio,
            SettingsTab::Network,
            SettingsTab::Advanced,
        ]
    }
}

/// 当前选中的标签页
#[derive(Resource, Default)]
pub struct CurrentSettingsTab(pub SettingsTab);

/// 临时设置（编辑中，未保存）
#[derive(Resource, Clone)]
pub struct TempSettings(pub GameSettings);

/// 设置按钮动作
#[derive(Component, Clone, Debug)]
pub enum SettingsAction {
    // 标签页切换
    SwitchTab(SettingsTab),
    // 底部按钮
    Save,
    Cancel,
    RestoreDefaults,
    // 游戏设置
    TimeLimitPrev,
    TimeLimitNext,
    AiTimeoutDecrease,
    AiTimeoutIncrease,
    DifficultyPrev,
    DifficultyNext,
    AnimationSpeedDecrease,
    AnimationSpeedIncrease,
    ToggleMoveHints,
    BoardFlipPrev,
    BoardFlipNext,
    // 显示设置
    ResolutionPrev,
    ResolutionNext,
    FullscreenPrev,
    FullscreenNext,
    ToggleVsync,
    FrameRatePrev,
    FrameRateNext,
    // 网络设置 - 文本输入需要特殊处理
    // 高级设置
    LogLevelPrev,
    LogLevelNext,
    ToggleShowFps,
}

/// 设置值显示标记
#[derive(Component)]
pub struct SettingValueDisplay(pub &'static str);

/// 设置页面初始化
pub fn setup_settings(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<GameSettings>,
) {
    // 初始化临时设置
    commands.insert_resource(TempSettings(settings.clone()));
    commands.insert_resource(CurrentSettingsTab::default());

    // 根容器
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            SettingsMarker,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("设置"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.8, 0.6)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // 主内容区域（标签栏 + 内容）
            parent
                .spawn(Node {
                    width: Val::Px(800.0),
                    height: Val::Px(500.0),
                    flex_direction: FlexDirection::Row,
                    ..default()
                })
                .with_children(|parent| {
                    // 左侧标签栏
                    spawn_tab_bar(parent, &asset_server);

                    // 右侧内容区
                    spawn_content_area(parent, &asset_server, &settings);
                });

            // 底部按钮栏
            spawn_bottom_buttons(parent, &asset_server);
        });
}

/// 生成标签栏
fn spawn_tab_bar(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent
        .spawn((
            Node {
                width: Val::Px(120.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        ))
        .with_children(|parent| {
            for tab in SettingsTab::all() {
                spawn_tab_button(parent, asset_server, *tab);
            }
        });
}

/// 生成标签按钮
fn spawn_tab_button(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer, tab: SettingsTab) {
    let is_selected = tab == SettingsTab::Game; // 默认选中游戏标签
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(if is_selected {
                Color::srgb(0.3, 0.3, 0.5)
            } else {
                NORMAL_BUTTON
            }),
            SettingsAction::SwitchTab(tab),
            tab,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(tab.display_name()),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 生成内容区域
fn spawn_content_area(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    settings: &GameSettings,
) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
            ContentAreaMarker,
        ))
        .with_children(|parent| {
            // 默认显示游戏设置
            spawn_game_settings(parent, asset_server, settings);
        });
}

/// 内容区域标记
#[derive(Component)]
pub struct ContentAreaMarker;

/// 生成游戏设置内容
fn spawn_game_settings(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    settings: &GameSettings,
) {
    // 本地对局时间限制
    spawn_setting_row(
        parent,
        asset_server,
        "本地对局时间限制",
        settings.time_limit.display_name(),
        "time_limit",
        SettingsAction::TimeLimitPrev,
        SettingsAction::TimeLimitNext,
    );

    // AI 思考时间上限
    spawn_setting_row(
        parent,
        asset_server,
        "AI 思考时间上限",
        &format!("{} 秒", settings.ai_timeout_secs),
        "ai_timeout",
        SettingsAction::AiTimeoutDecrease,
        SettingsAction::AiTimeoutIncrease,
    );

    // 默认 AI 难度
    spawn_setting_row(
        parent,
        asset_server,
        "默认 AI 难度",
        match settings.default_difficulty {
            protocol::Difficulty::Easy => "简单",
            protocol::Difficulty::Medium => "中等",
            protocol::Difficulty::Hard => "困难",
        },
        "difficulty",
        SettingsAction::DifficultyPrev,
        SettingsAction::DifficultyNext,
    );

    // 棋子动画速度
    spawn_setting_row(
        parent,
        asset_server,
        "棋子动画速度",
        &format!("{:.1}x", settings.animation_speed),
        "animation_speed",
        SettingsAction::AnimationSpeedDecrease,
        SettingsAction::AnimationSpeedIncrease,
    );

    // 走子提示
    spawn_toggle_row(
        parent,
        asset_server,
        "走子提示",
        settings.show_move_hints,
        "show_move_hints",
        SettingsAction::ToggleMoveHints,
    );

    // 翻转棋盘
    spawn_setting_row(
        parent,
        asset_server,
        "翻转棋盘",
        settings.board_flip.display_name(),
        "board_flip",
        SettingsAction::BoardFlipPrev,
        SettingsAction::BoardFlipNext,
    );
}

/// 生成显示设置内容
fn spawn_display_settings(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    settings: &GameSettings,
) {
    // 窗口分辨率
    spawn_setting_row(
        parent,
        asset_server,
        "窗口分辨率",
        settings.resolution.display_name(),
        "resolution",
        SettingsAction::ResolutionPrev,
        SettingsAction::ResolutionNext,
    );

    // 全屏模式
    spawn_setting_row(
        parent,
        asset_server,
        "全屏模式",
        settings.fullscreen_mode.display_name(),
        "fullscreen",
        SettingsAction::FullscreenPrev,
        SettingsAction::FullscreenNext,
    );

    // 垂直同步
    spawn_toggle_row(
        parent,
        asset_server,
        "垂直同步",
        settings.vsync,
        "vsync",
        SettingsAction::ToggleVsync,
    );

    // 帧率限制
    spawn_setting_row(
        parent,
        asset_server,
        "帧率限制",
        settings.frame_rate_limit.display_name(),
        "frame_rate",
        SettingsAction::FrameRatePrev,
        SettingsAction::FrameRateNext,
    );
}

/// 生成音频设置内容（预留）
fn spawn_audio_settings(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent.spawn((
        Text::new("音频设置功能敬请期待..."),
        TextFont {
            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.6, 0.6, 0.6)),
        Node {
            margin: UiRect::all(Val::Px(20.0)),
            ..default()
        },
    ));
}

/// 生成网络设置内容
fn spawn_network_settings(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    settings: &GameSettings,
) {
    // 服务器地址（只读显示，暂不支持编辑）
    spawn_text_display_row(
        parent,
        asset_server,
        "默认服务器地址",
        &settings.server_address,
    );

    // 昵称（只读显示，暂不支持编辑）
    spawn_text_display_row(parent, asset_server, "默认昵称", &settings.nickname);

    // 分隔线
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(15.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
    ));

    // LLM 设置标题
    parent.spawn((
        Text::new("─ LLM 设置（AI 复盘分析）─"),
        TextFont {
            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.6, 0.8, 0.9)),
        Node {
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        },
    ));

    // Ollama 服务地址
    spawn_text_display_row(
        parent,
        asset_server,
        "Ollama 地址",
        &settings.llm_base_url,
    );

    // LLM 模型
    spawn_text_display_row(parent, asset_server, "LLM 模型", &settings.llm_model);

    // 提示
    parent.spawn((
        Text::new("文本输入功能开发中..."),
        TextFont {
            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.5, 0.5, 0.5)),
        Node {
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        },
    ));

    // LLM 使用说明
    parent.spawn((
        Text::new("提示：需要本地运行 Ollama 服务才能使用 AI 复盘功能"),
        TextFont {
            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.6, 0.6, 0.6)),
        Node {
            margin: UiRect::top(Val::Px(5.0)),
            ..default()
        },
    ));
}

/// 生成高级设置内容
fn spawn_advanced_settings(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    settings: &GameSettings,
) {
    // 日志级别
    spawn_setting_row(
        parent,
        asset_server,
        "日志级别",
        settings.log_level.display_name(),
        "log_level",
        SettingsAction::LogLevelPrev,
        SettingsAction::LogLevelNext,
    );

    // 显示 FPS
    spawn_toggle_row(
        parent,
        asset_server,
        "显示 FPS",
        settings.show_fps,
        "show_fps",
        SettingsAction::ToggleShowFps,
    );

    // 语言（预留）
    parent.spawn((
        Text::new("语言切换功能敬请期待..."),
        TextFont {
            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.5, 0.5, 0.5)),
        Node {
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        },
    ));
}

/// 生成设置行（带左右箭头）
fn spawn_setting_row(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    label: &str,
    value: &str,
    value_id: &'static str,
    prev_action: SettingsAction,
    next_action: SettingsAction,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        })
        .with_children(|parent| {
            // 标签
            parent.spawn((
                Text::new(label),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 值选择器
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    // 左箭头
                    spawn_arrow_button(parent, asset_server, "<", prev_action);

                    // 值显示
                    parent.spawn((
                        Text::new(value),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.8, 0.6)),
                        Node {
                            width: Val::Px(120.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        SettingValueDisplay(value_id),
                    ));

                    // 右箭头
                    spawn_arrow_button(parent, asset_server, ">", next_action);
                });
        });
}

/// 生成开关行
fn spawn_toggle_row(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    label: &str,
    value: bool,
    value_id: &'static str,
    toggle_action: SettingsAction,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        })
        .with_children(|parent| {
            // 标签
            parent.spawn((
                Text::new(label),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 开关按钮
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(80.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(if value {
                        Color::srgb(0.2, 0.5, 0.2)
                    } else {
                        Color::srgb(0.4, 0.2, 0.2)
                    }),
                    toggle_action,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(if value { "开" } else { "关" }),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        SettingValueDisplay(value_id),
                    ));
                });
        });
}

/// 生成文本显示行（只读）
fn spawn_text_display_row(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    label: &str,
    value: &str,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        })
        .with_children(|parent| {
            // 标签
            parent.spawn((
                Text::new(label),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 值
            parent.spawn((
                Text::new(value),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

/// 生成箭头按钮
fn spawn_arrow_button(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    text: &str,
    action: SettingsAction,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(30.0),
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON),
            action,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 生成底部按钮栏
fn spawn_bottom_buttons(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent
        .spawn(Node {
            width: Val::Px(800.0),
            height: Val::Px(60.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        })
        .with_children(|parent| {
            spawn_bottom_button(parent, asset_server, "保存", SettingsAction::Save);
            spawn_bottom_button(parent, asset_server, "取消", SettingsAction::Cancel);
            spawn_bottom_button(
                parent,
                asset_server,
                "恢复默认",
                SettingsAction::RestoreDefaults,
            );
        });
}

/// 生成底部按钮
fn spawn_bottom_button(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    text: &str,
    action: SettingsAction,
) {
    parent
        .spawn((
            Button,
            button_style(),
            BackgroundColor(NORMAL_BUTTON),
            action,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 清理设置页面
pub fn cleanup_settings(
    mut commands: Commands,
    query: Query<Entity, With<SettingsMarker>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TempSettings>();
    commands.remove_resource::<CurrentSettingsTab>();
}

/// 处理设置按钮
pub fn handle_settings_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &SettingsAction, Option<&SettingsTab>),
        (Changed<Interaction>, With<Button>),
    >,
    mut temp_settings: ResMut<TempSettings>,
    mut current_tab: ResMut<CurrentSettingsTab>,
    mut game_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<GameSettings>,
    mut windows: Query<&mut Window>,
    mut winit_settings: ResMut<WinitSettings>,
) {
    for (interaction, mut color, action, tab) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                handle_settings_action(
                    action,
                    tab,
                    &mut temp_settings,
                    &mut current_tab,
                    &mut game_state,
                    &mut settings,
                    &mut windows,
                    &mut winit_settings,
                );
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                // 标签页按钮保持选中状态颜色
                if let Some(t) = tab {
                    if *t == current_tab.0 {
                        *color = Color::srgb(0.3, 0.3, 0.5).into();
                    } else {
                        *color = NORMAL_BUTTON.into();
                    }
                } else {
                    *color = NORMAL_BUTTON.into();
                }
            }
        }
    }
}

/// 处理设置动作
fn handle_settings_action(
    action: &SettingsAction,
    _tab: Option<&SettingsTab>,
    temp_settings: &mut ResMut<TempSettings>,
    current_tab: &mut ResMut<CurrentSettingsTab>,
    game_state: &mut ResMut<NextState<GameState>>,
    settings: &mut ResMut<GameSettings>,
    windows: &mut Query<&mut Window>,
    winit_settings: &mut ResMut<WinitSettings>,
) {
    match action {
        SettingsAction::SwitchTab(tab) => {
            current_tab.0 = *tab;
        }
        SettingsAction::Save => {
            // 保存设置
            **settings = temp_settings.0.clone();
            if let Err(e) = settings.save() {
                tracing::error!("保存设置失败: {}", e);
            }
            // 应用显示设置
            apply_display_settings(&temp_settings.0, windows, winit_settings);
            game_state.set(GameState::Menu);
        }
        SettingsAction::Cancel => {
            game_state.set(GameState::Menu);
        }
        SettingsAction::RestoreDefaults => {
            temp_settings.0 = GameSettings::default();
        }
        // 游戏设置
        SettingsAction::TimeLimitPrev => {
            temp_settings.0.time_limit = temp_settings.0.time_limit.prev();
        }
        SettingsAction::TimeLimitNext => {
            temp_settings.0.time_limit = temp_settings.0.time_limit.next();
        }
        SettingsAction::AiTimeoutDecrease => {
            if temp_settings.0.ai_timeout_secs > 5 {
                temp_settings.0.ai_timeout_secs -= 1;
            }
        }
        SettingsAction::AiTimeoutIncrease => {
            if temp_settings.0.ai_timeout_secs < 30 {
                temp_settings.0.ai_timeout_secs += 1;
            }
        }
        SettingsAction::DifficultyPrev => {
            temp_settings.0.default_difficulty = match temp_settings.0.default_difficulty {
                protocol::Difficulty::Easy => protocol::Difficulty::Hard,
                protocol::Difficulty::Medium => protocol::Difficulty::Easy,
                protocol::Difficulty::Hard => protocol::Difficulty::Medium,
            };
        }
        SettingsAction::DifficultyNext => {
            temp_settings.0.default_difficulty = match temp_settings.0.default_difficulty {
                protocol::Difficulty::Easy => protocol::Difficulty::Medium,
                protocol::Difficulty::Medium => protocol::Difficulty::Hard,
                protocol::Difficulty::Hard => protocol::Difficulty::Easy,
            };
        }
        SettingsAction::AnimationSpeedDecrease => {
            if temp_settings.0.animation_speed > 0.5 {
                temp_settings.0.animation_speed -= 0.1;
            }
        }
        SettingsAction::AnimationSpeedIncrease => {
            if temp_settings.0.animation_speed < 2.0 {
                temp_settings.0.animation_speed += 0.1;
            }
        }
        SettingsAction::ToggleMoveHints => {
            temp_settings.0.show_move_hints = !temp_settings.0.show_move_hints;
        }
        SettingsAction::BoardFlipPrev => {
            temp_settings.0.board_flip = temp_settings.0.board_flip.prev();
        }
        SettingsAction::BoardFlipNext => {
            temp_settings.0.board_flip = temp_settings.0.board_flip.next();
        }
        // 显示设置
        SettingsAction::ResolutionPrev => {
            temp_settings.0.resolution = temp_settings.0.resolution.prev();
        }
        SettingsAction::ResolutionNext => {
            temp_settings.0.resolution = temp_settings.0.resolution.next();
        }
        SettingsAction::FullscreenPrev => {
            temp_settings.0.fullscreen_mode = temp_settings.0.fullscreen_mode.prev();
        }
        SettingsAction::FullscreenNext => {
            temp_settings.0.fullscreen_mode = temp_settings.0.fullscreen_mode.next();
        }
        SettingsAction::ToggleVsync => {
            temp_settings.0.vsync = !temp_settings.0.vsync;
        }
        SettingsAction::FrameRatePrev => {
            temp_settings.0.frame_rate_limit = temp_settings.0.frame_rate_limit.prev();
        }
        SettingsAction::FrameRateNext => {
            temp_settings.0.frame_rate_limit = temp_settings.0.frame_rate_limit.next();
        }
        // 高级设置
        SettingsAction::LogLevelPrev => {
            temp_settings.0.log_level = temp_settings.0.log_level.prev();
        }
        SettingsAction::LogLevelNext => {
            temp_settings.0.log_level = temp_settings.0.log_level.next();
        }
        SettingsAction::ToggleShowFps => {
            temp_settings.0.show_fps = !temp_settings.0.show_fps;
        }
    }
}

/// 应用显示设置
fn apply_display_settings(
    settings: &GameSettings,
    windows: &mut Query<&mut Window>,
    winit_settings: &mut ResMut<WinitSettings>,
) {
    use std::time::Duration;
    
    if let Ok(mut window) = windows.single_mut() {
        // 分辨率
        window.resolution = WindowResolution::new(
            settings.resolution.width(),
            settings.resolution.height(),
        );
        // 全屏模式
        window.mode = settings.fullscreen_mode.to_window_mode();
        // 垂直同步
        window.present_mode = settings.present_mode();
    }
    
    // 帧率限制：通过 WinitSettings 的 UpdateMode 实现
    // 当有帧率限制时，使用 Reactive 模式配合 wait 时间
    // 当无限制时，使用 Continuous 模式
    if let Some(fps) = settings.frame_rate_limit.to_fps() {
        let frame_duration = Duration::from_secs_f64(1.0 / fps);
        winit_settings.focused_mode = UpdateMode::reactive(frame_duration);
        winit_settings.unfocused_mode = UpdateMode::reactive_low_power(frame_duration);
    } else {
        // 无限制：使用连续模式
        winit_settings.focused_mode = UpdateMode::Continuous;
        winit_settings.unfocused_mode = UpdateMode::reactive_low_power(Duration::from_millis(100));
    }
}

/// 更新设置值显示
pub fn update_settings_display(
    temp_settings: Res<TempSettings>,
    mut query: Query<(&mut Text, &SettingValueDisplay)>,
) {
    if !temp_settings.is_changed() {
        return;
    }

    for (mut text, display) in &mut query {
        let new_value = match display.0 {
            "time_limit" => temp_settings.0.time_limit.display_name().to_string(),
            "ai_timeout" => format!("{} 秒", temp_settings.0.ai_timeout_secs),
            "difficulty" => match temp_settings.0.default_difficulty {
                protocol::Difficulty::Easy => "简单".to_string(),
                protocol::Difficulty::Medium => "中等".to_string(),
                protocol::Difficulty::Hard => "困难".to_string(),
            },
            "animation_speed" => format!("{:.1}x", temp_settings.0.animation_speed),
            "show_move_hints" => {
                if temp_settings.0.show_move_hints {
                    "开"
                } else {
                    "关"
                }
                .to_string()
            }
            "board_flip" => temp_settings.0.board_flip.display_name().to_string(),
            "resolution" => temp_settings.0.resolution.display_name().to_string(),
            "fullscreen" => temp_settings.0.fullscreen_mode.display_name().to_string(),
            "vsync" => {
                if temp_settings.0.vsync {
                    "开"
                } else {
                    "关"
                }
                .to_string()
            }
            "frame_rate" => temp_settings.0.frame_rate_limit.display_name().to_string(),
            "log_level" => temp_settings.0.log_level.display_name().to_string(),
            "show_fps" => {
                if temp_settings.0.show_fps {
                    "开"
                } else {
                    "关"
                }
                .to_string()
            }
            _ => continue,
        };
        **text = new_value;
    }
}

/// 更新标签页内容
pub fn update_tab_content(
    mut commands: Commands,
    current_tab: Res<CurrentSettingsTab>,
    temp_settings: Res<TempSettings>,
    asset_server: Res<AssetServer>,
    content_query: Query<(Entity, Option<&Children>), With<ContentAreaMarker>>,
) {
    if !current_tab.is_changed() {
        return;
    }

    // 清除现有内容
    for (entity, children) in content_query.iter() {
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        // 重新生成内容
        commands.entity(entity).with_children(|parent| {
            match current_tab.0 {
                SettingsTab::Game => spawn_game_settings(parent, &asset_server, &temp_settings.0),
                SettingsTab::Display => {
                    spawn_display_settings(parent, &asset_server, &temp_settings.0)
                }
                SettingsTab::Audio => spawn_audio_settings(parent, &asset_server),
                SettingsTab::Network => {
                    spawn_network_settings(parent, &asset_server, &temp_settings.0)
                }
                SettingsTab::Advanced => {
                    spawn_advanced_settings(parent, &asset_server, &temp_settings.0)
                }
            }
        });
    }
}

/// 更新标签按钮样式
pub fn update_tab_buttons(
    current_tab: Res<CurrentSettingsTab>,
    mut query: Query<(&mut BackgroundColor, &SettingsTab), With<Button>>,
) {
    if !current_tab.is_changed() {
        return;
    }

    for (mut color, tab) in &mut query {
        if *tab == current_tab.0 {
            *color = Color::srgb(0.3, 0.3, 0.5).into();
        } else {
            *color = NORMAL_BUTTON.into();
        }
    }
}
