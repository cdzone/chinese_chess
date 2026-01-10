//! 主菜单 UI

use bevy::prelude::*;
use protocol::Difficulty;

use super::{ButtonAction, MenuMarker, UiMarker, button_style, NORMAL_BUTTON, HOVERED_BUTTON, PRESSED_BUTTON};
use crate::game::ClientGame;
use crate::network::NetworkEvent;
use crate::settings::GameSettings;
use crate::GameState;

/// 主菜单分组标题颜色
const SECTION_TITLE_COLOR: Color = Color::srgb(0.7, 0.6, 0.5);

/// 设置主菜单
pub fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 根容器
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            UiMarker,
            MenuMarker,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("中国象棋"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.8, 0.6)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // 主内容区域（水平布局：单机对战 | 在线对战）
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Start,
                    column_gap: Val::Px(60.0),
                    ..default()
                })
                .with_children(|parent| {
                    // 左侧：单机对战
                    spawn_menu_section(
                        parent,
                        &asset_server,
                        "单机对战",
                        vec![
                            ("简单", ButtonAction::PlayVsAi(Difficulty::Easy)),
                            ("中等", ButtonAction::PlayVsAi(Difficulty::Medium)),
                            ("困难", ButtonAction::PlayVsAi(Difficulty::Hard)),
                        ],
                    );

                    // 分隔线
                    parent.spawn((
                        Node {
                            width: Val::Px(2.0),
                            height: Val::Px(200.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    ));

                    // 右侧：在线对战
                    spawn_menu_section(
                        parent,
                        &asset_server,
                        "在线对战",
                        vec![
                            ("快速匹配", ButtonAction::QuickMatch),
                            ("创建房间", ButtonAction::CreatePvPRoom),
                            ("加入房间", ButtonAction::JoinRoom),
                        ],
                    );
                });

            // 底部按钮区
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(40.0)),
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|parent| {
                    // 加载棋局（功能开发中，显示灰色）
                    spawn_disabled_button(
                        parent,
                        &asset_server,
                        "加载棋局",
                        ButtonAction::LoadGame,
                    );

                    // 设置
                    spawn_menu_button(
                        parent,
                        &asset_server,
                        "设置",
                        ButtonAction::Settings,
                    );

                    // 退出游戏
                    spawn_menu_button(
                        parent,
                        &asset_server,
                        "退出游戏",
                        ButtonAction::ExitGame,
                    );
                });
        });
}

/// 生成菜单分组
fn spawn_menu_section(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    title: &str,
    buttons: Vec<(&str, ButtonAction)>,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            // 分组标题
            parent.spawn((
                Text::new(title),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(SECTION_TITLE_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(15.0)),
                    ..default()
                },
            ));

            // 按钮
            for (text, action) in buttons {
                spawn_menu_button(parent, asset_server, text, action);
            }
        });
}

/// 生成菜单按钮
fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    text: &str,
    action: ButtonAction,
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
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 禁用按钮颜色
const DISABLED_BUTTON: Color = Color::srgb(0.18, 0.18, 0.18);

/// 生成禁用状态的菜单按钮（功能开发中）
fn spawn_disabled_button(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    text: &str,
    action: ButtonAction,
) {
    parent
        .spawn((
            Button,
            button_style(),
            BackgroundColor(DISABLED_BUTTON),
            action,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("{} (开发中)", text)),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),  // 灰色文字
            ));
        });
}

/// 清理主菜单
pub fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 处理菜单按钮点击
pub fn handle_menu_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut network_events: MessageWriter<NetworkEvent>,
    mut game_state: ResMut<NextState<GameState>>,
    mut game: ResMut<ClientGame>,
    mut exit: MessageWriter<AppExit>,
    mut network_state: ResMut<crate::network::NetworkState>,
    settings: Res<GameSettings>,
) {
    for (interaction, mut color, action) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                handle_button_action(action, &mut network_events, &mut game_state, &mut game, &mut exit, &mut network_state, &settings);
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

/// 处理按钮动作
fn handle_button_action(
    action: &ButtonAction,
    network_events: &mut MessageWriter<NetworkEvent>,
    game_state: &mut ResMut<NextState<GameState>>,
    game: &mut ResMut<ClientGame>,
    exit: &mut MessageWriter<AppExit>,
    network_state: &mut ResMut<crate::network::NetworkState>,
    settings: &GameSettings,
) {
    match action {
        ButtonAction::PlayVsAi(difficulty) => {
            // 本地 PvE 模式：完全离线，无需网络
            game.start_local_pve(*difficulty);
            // 使用设置中的时间限制
            let time_ms = settings.time_limit.to_millis();
            game.red_time_ms = time_ms;
            game.black_time_ms = time_ms;
            game_state.set(GameState::Playing);
            
            tracing::info!("Starting local PvE game with difficulty: {:?}", difficulty);
        }
        ButtonAction::QuickMatch => {
            // 快速匹配：登录后自动加入或创建房间
            network_state.pending_action = crate::network::PendingAction::QuickMatch;
            
            network_events.write(NetworkEvent::Connect {
                addr: settings.server_address.clone(),
                nickname: settings.nickname.clone(),
            });
            
            tracing::info!("Quick match requested");
        }
        ButtonAction::CreatePvPRoom => {
            // 设置待处理操作，登录成功后自动创建房间
            network_state.pending_action = crate::network::PendingAction::CreateRoom {
                room_type: protocol::RoomType::PvP,
                preferred_side: None,
            };
            
            // 使用设置中的服务器地址和昵称
            network_events.write(NetworkEvent::Connect {
                addr: settings.server_address.clone(),
                nickname: settings.nickname.clone(),
            });
        }
        ButtonAction::JoinRoom => {
            // 设置待处理操作，登录成功后自动获取房间列表
            network_state.pending_action = crate::network::PendingAction::ListRooms;
            
            // 使用设置中的服务器地址和昵称
            network_events.write(NetworkEvent::Connect {
                addr: settings.server_address.clone(),
                nickname: settings.nickname.clone(),
            });
            game_state.set(GameState::Lobby);
        }
        ButtonAction::LoadGame => {
            // TODO: 显示加载棋局界面
            tracing::info!("Load game clicked");
        }
        ButtonAction::Settings => {
            game_state.set(GameState::Settings);
        }
        ButtonAction::ExitGame => {
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}
