//! 大厅 UI - 房间列表界面

use bevy::prelude::*;
use protocol::{RoomInfo, RoomState, RoomType};

use super::{ButtonAction, UiMarker, button_style, NORMAL_BUTTON, HOVERED_BUTTON, PRESSED_BUTTON};
use crate::network::{ConnectionStatus, NetworkEvent, NetworkState};
use crate::GameState;

/// 大厅 UI 标记
#[derive(Component)]
pub struct LobbyMarker;

/// 房间列表容器标记
#[derive(Component)]
pub struct RoomListContainer;

/// 房间条目标记
#[derive(Component)]
pub struct RoomEntry(pub protocol::RoomId);

/// 设置大厅 UI
pub fn setup_lobby(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            UiMarker,
            LobbyMarker,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("房间列表"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.8, 0.6)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // 房间列表容器（带滚动）
            parent
                .spawn((
                    Node {
                        width: Val::Px(600.0),
                        height: Val::Px(400.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        padding: UiRect::all(Val::Px(10.0)),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    RoomListContainer,
                ))
                .with_children(|parent| {
                    // 初始提示
                    parent.spawn((
                        Text::new("正在加载房间列表..."),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        Node {
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                    ));
                });

            // 底部按钮栏
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(30.0)),
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|parent| {
                    // 刷新按钮
                    spawn_lobby_button(parent, &asset_server, "刷新", ButtonAction::RefreshRooms);
                    
                    // 返回主菜单
                    spawn_lobby_button(parent, &asset_server, "返回", ButtonAction::BackToMenuFromLobby);
                });
        });
}

/// 生成大厅按钮
fn spawn_lobby_button(
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
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 清理大厅 UI
pub fn cleanup_lobby(mut commands: Commands, query: Query<Entity, With<LobbyMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 更新房间列表显示
pub fn update_room_list(
    mut commands: Commands,
    network: Res<NetworkState>,
    asset_server: Res<AssetServer>,
    container_query: Query<(Entity, Option<&Children>), With<RoomListContainer>>,
) {
    // 只在房间列表变化时更新
    if !network.is_changed() {
        return;
    }

    let Ok((container, children)) = container_query.single() else {
        return;
    };

    // 清除容器的所有子节点
    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    // 检查连接错误
    if network.status == ConnectionStatus::Error {
        let error_msg = network.connection_error.as_deref().unwrap_or("连接失败");
        commands.entity(container).with_children(|parent| {
            // 错误图标和消息
            parent.spawn((
                Text::new(format!("⚠ {}", error_msg)),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.4, 0.3)),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));
            
            // 提示信息
            parent.spawn((
                Text::new("请检查服务器是否启动，或点击「返回」回到主菜单"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                Node {
                    margin: UiRect::horizontal(Val::Px(20.0)),
                    ..default()
                },
            ));
        });
        return;
    }
    
    // 检查是否正在连接
    if network.status == ConnectionStatus::Connecting {
        commands.entity(container).with_children(|parent| {
            parent.spawn((
                Text::new("正在连接服务器..."),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));
        });
        return;
    }

    if network.room_list.is_empty() {
        // 显示空列表提示
        commands.entity(container).with_children(|parent| {
            parent.spawn((
                Text::new("暂无可用房间，请创建新房间或稍后刷新"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));
        });
    } else {
        // 对房间列表排序：等待中 > 游戏中 > 暂停中 > 已结束
        let mut sorted_rooms = network.room_list.clone();
        sorted_rooms.sort_by_key(|r| {
            match r.state {
                RoomState::Waiting => 0,
                RoomState::Playing => 1,
                RoomState::Paused => 2,
                RoomState::Finished => 3,
            }
        });
        
        // 显示房间列表
        commands.entity(container).with_children(|parent| {
            for room in &sorted_rooms {
                spawn_room_entry(parent, &asset_server, room);
            }
        });
    }
}

/// 生成房间条目
fn spawn_room_entry(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer, room: &RoomInfo) {
    let status_text = match room.state {
        RoomState::Waiting => "等待中",
        RoomState::Playing => "游戏中",
        RoomState::Paused => "暂停中",
        RoomState::Finished => "已结束",
    };

    let room_type_text = match &room.room_type {
        RoomType::PvP => "PvP",
        RoomType::PvE(diff) => match diff {
            protocol::Difficulty::Easy => "PvE 简单",
            protocol::Difficulty::Medium => "PvE 中等",
            protocol::Difficulty::Hard => "PvE 困难",
        },
    };

    // 计算玩家数量
    let player_count = room.red_player.is_some() as u8 + room.black_player.is_some() as u8;
    let can_join = room.state == RoomState::Waiting && matches!(room.room_type, RoomType::PvP);

    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(15.0)),
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            RoomEntry(room.id),
        ))
        .with_children(|parent| {
            // 左侧：房间信息
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    // 房间名称
                    parent.spawn((
                        Text::new(format!("房间 #{}", room.id)),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // 房间详情
                    parent.spawn((
                        Text::new(format!(
                            "{} | {} | 玩家: {}/2",
                            room_type_text, status_text, player_count
                        )),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                });

            // 右侧：加入按钮
            if can_join {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(80.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.5, 0.3)),
                        ButtonAction::JoinRoomById(room.id),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new("加入"),
                            TextFont {
                                font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            } else {
                // 不可加入时显示灰色文字
                parent.spawn((
                    Text::new(if room.state == RoomState::Playing {
                        "进行中"
                    } else {
                        "不可加入"
                    }),
                    TextFont {
                        font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            }
        });
}

/// 处理大厅按钮
pub fn handle_lobby_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut network_events: MessageWriter<NetworkEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color, action) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                
                match action {
                    ButtonAction::RefreshRooms => {
                        network_events.write(NetworkEvent::ListRooms);
                        tracing::info!("Refreshing room list");
                    }
                    ButtonAction::BackToMenuFromLobby => {
                        network_events.write(NetworkEvent::Disconnect);
                        game_state.set(GameState::Menu);
                    }
                    ButtonAction::JoinRoomById(room_id) => {
                        network_events.write(NetworkEvent::JoinRoom { room_id: *room_id });
                        tracing::info!("Joining room: {}", room_id);
                    }
                    _ => {}
                }
            }
            Interaction::Hovered => {
                // 加入按钮使用不同的悬停颜色
                if matches!(action, ButtonAction::JoinRoomById(_)) {
                    *color = Color::srgb(0.25, 0.6, 0.35).into();
                } else {
                    *color = HOVERED_BUTTON.into();
                }
            }
            Interaction::None => {
                if matches!(action, ButtonAction::JoinRoomById(_)) {
                    *color = Color::srgb(0.2, 0.5, 0.3).into();
                } else {
                    *color = NORMAL_BUTTON.into();
                }
            }
        }
    }
}
