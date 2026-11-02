use bevy::ecs::system::EntityCommands;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use bevy_yarnspinner::prelude::*;
use eds_bevy_common::states_sets::{GameplayState, LevelState, OverlayState};

use crate::assets::GuiAssets;
use crate::yarn_ui::{AlchemyDialogFont, DialogueRootNode};

// const DIALOG_WIDTH: f32 = 800.0 * 0.8;
const DIALOG_WIDTH_PERCENT: f32 = 80.0;
const OPTION_WIDTH_PERCENT: f32 = 80.0;
const DIALOG_WIDTH: Val = Val::Percent(DIALOG_WIDTH_PERCENT);
const TEXT_BORDER_HORIZONTAL: Val = Val::Percent(4.0);
const TEXT_BORDER_TOP: f32 = 30.0;
const TEXT_BORDER_BOTTOM: f32 = TEXT_BORDER_TOP + 10.0;

pub(crate) fn ui_setup_plugin(app: &mut App) {
    app.add_systems(Startup, |mut commands: Commands| {
        commands.insert_resource(AlchemyDialogFont(default()));
    })
    .add_systems(OnEnter(LevelState::LevelLoaded), setup)
    .add_systems(OnExit(GameplayState::Playing), remove_dialogue)
    .add_systems(OnEnter(OverlayState::ModalDialog), show_dialogue)
    .add_systems(OnExit(OverlayState::ModalDialog), hide_dialogue);
}

#[derive(Debug, Default, Component)]
pub(crate) struct DialogueNode;

#[derive(Debug, Default, Component)]
pub(crate) struct DialogueNameNode;

#[derive(Debug, Default, Component)]
pub(crate) struct DialogueContinueNode;

#[derive(Debug, Default, Component)]
pub(crate) struct OptionsNode;

#[derive(Debug, Component)]
pub(crate) struct OptionButton(pub OptionId);

fn show_dialogue(mut query: Query<&mut Visibility, With<DialogueRootNode>>) {
    for mut vis in query.iter_mut() {
        *vis = Visibility::Inherited;
    }
}
fn hide_dialogue(mut query: Query<&mut Visibility, With<DialogueRootNode>>) {
    for mut vis in query.iter_mut() {
        *vis = Visibility::Hidden;
    }
}
fn remove_dialogue(mut commands: Commands, mut query: Query<Entity, With<DialogueRootNode>>) {
    for ent in query.iter_mut() {
        commands.entity(ent).despawn();
    }
}

fn setup(mut commands: Commands, font: Res<AlchemyDialogFont>, gui_assets: Res<GuiAssets>) {
    // root node
    commands
        .spawn((
            DespawnOnExit(GameplayState::Playing),
            fmt_name("root"),
            Node {
                display: Display::Grid,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_content: AlignContent::End,
                justify_content: JustifyContent::SpaceAround,
                grid_auto_flow: GridAutoFlow::Row,
                grid_template_columns: vec![RepeatedGridTrack::minmax(
                    1,
                    MinTrackSizingFunction::Auto,
                    MaxTrackSizingFunction::Percent(DIALOG_WIDTH_PERCENT),
                )],
                ..default()
            },
            Visibility::Hidden,
            DialogueRootNode,
        ))
        .with_children(|parent| {
            parent.spawn((
                fmt_name("name"),
                Text::default(),
                text_style::name(&font.0),
                Node {
                    margin: UiRect {
                        left: TEXT_BORDER_HORIZONTAL / 2.0,
                        bottom: Val::Px(-8.0),
                        ..default()
                    },
                    ..default()
                },
                ZIndex(1),
                DialogueNameNode,
                Label,
            ));

            parent
                .spawn((
                    fmt_name("dialogue"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::FlexStart,
                        padding: UiRect {
                            top: Val::Px(TEXT_BORDER_TOP),
                            bottom: Val::Px(TEXT_BORDER_BOTTOM),
                            left: TEXT_BORDER_HORIZONTAL,
                            right: TEXT_BORDER_HORIZONTAL,
                        },
                        border_radius: BorderRadius::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::BLACK.with_alpha(0.8)),
                ))
                .with_children(|parent| {
                    // Dialog itself
                    parent.spawn((
                        fmt_name("text"),
                        Text::default(),
                        text_style::standard(&font.0),
                        style::standard(),
                        DialogueNode,
                        Label,
                    ));
                })
                .with_children(|parent| {
                    // Options
                    parent.spawn((
                        fmt_name("options"),
                        Node {
                            display: Display::None,
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::FlexEnd,
                            align_items: AlignItems::FlexStart,
                            margin: UiRect::top(Val::Px(20.0)),
                            width: Val::Percent(OPTION_WIDTH_PERCENT),
                            ..default()
                        },
                        Visibility::Hidden,
                        OptionsNode,
                    ));
                });

            parent.spawn((
                fmt_name("continue indicator"),
                // ImageNode {
                //     // 27 x 27 pixels
                //     image: image_handle::CONTINUE_INDICATOR,
                //     ..default()
                // },
                Text::new("\u{f14f7}"), // nf-md-book_open_variant
                TextFont::from_font_size(Val::Em(1.0))
                    .with_font(gui_assets.symbols_nerd_font.clone()),
                Node {
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    margin: UiRect {
                        top: Val::Px(-18.),
                        bottom: Val::Px(25.),
                        ..default()
                    },
                    ..default()
                },
                ZIndex(1),
                Visibility::Hidden,
                DialogueContinueNode,
            ));
        });
}

fn fmt_name(name: &str) -> Name {
    Name::new(format!("YS: {name}"))
}

pub(crate) fn create_dialog_text(
    font: Res<AlchemyDialogFont>,
    text: impl Into<String>,
    invisible: impl Into<String>,
) -> [(TextSpan, TextFont, TextColor); 2] {
    [
        (
            TextSpan(text.into()),
            text_style::standard(&font.0).0,
            text_style::standard(&font.0).1,
        ),
        (
            TextSpan(invisible.into()),
            text_style::standard(&font.0).0,
            TextColor(Color::NONE),
        ),
    ]
}

pub(crate) fn spawn_options<'a, T>(
    entity_commands: &mut EntityCommands,
    font: Res<AlchemyDialogFont>,
    options: T,
) where
    T: IntoIterator<Item = &'a DialogueOption>,
    <T as IntoIterator>::IntoIter: 'a,
{
    entity_commands.with_children(|parent| {
        for (i, option) in options.into_iter().enumerate() {
            parent
                .spawn((
                    fmt_name("option button"),
                    Node {
                        justify_content: JustifyContent::FlexStart,
                        width: Val::Percent(100.0),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            fmt_name("option text"),
                            Button,
                            // UiDebugOptions {
                            //     enabled: true,
                            //     outline_border_box: true,
                            //     outline_padding_box: false,
                            //     outline_content_box: false,
                            //     outline_scrollbars: true,
                            //     line_width: 2.,
                            //     line_color_override: None,
                            //     show_hidden: false,
                            //     show_clipped: true,
                            //     ignore_border_radius: true,
                            // },
                            Hovered::default(),
                            Text::default(),
                            style::options(),
                            ImageNode::default().with_color(Color::NONE),
                            OptionButton(option.id),
                            Label,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextSpan(format!("{}: ", i + 1)),
                                text_style::option_id(&font.0),
                            ));
                            parent.spawn((
                                TextSpan(option.line.text.clone()),
                                text_style::option_text(&font.0),
                            ));
                        });
                });
        }
    });
}

mod style {
    use super::*;
    pub(crate) fn standard() -> Node {
        Node {
            // max_width: Val::Px(DIALOG_WIDTH - 2.0 * TEXT_BORDER_HORIZONTAL),
            max_width: DIALOG_WIDTH,
            ..default()
        }
    }
    pub(crate) fn options() -> Node {
        const INDENT_MODIFIER: f32 = 1.0;
        Node {
            margin: UiRect::horizontal(TEXT_BORDER_HORIZONTAL),
            width: Val::Percent(OPTION_WIDTH_PERCENT),
            // max_width: Val::Px(800.0 - 10.0 * 2.0 * INDENT_MODIFIER),
            // max_width: Val::Vw(80.0),
            ..default()
        }
    }
}

mod text_style {
    use super::*;
    use bevy::color::palettes::css;
    pub(crate) fn standard(font: &Handle<Font>) -> (TextFont, TextColor) {
        (
            TextFont {
                font: FontSource::Handle(font.clone()),
                font_size: FontSize::Px(20.0),
                ..default()
            },
            TextColor(Color::WHITE),
        )
    }
    pub(crate) fn name(font: &Handle<Font>) -> (TextFont, TextColor) {
        (
            TextFont {
                font: FontSource::Handle(font.clone()),
                font_size: FontSize::Px(18.0),
                ..standard(font).0
            },
            standard(font).1,
        )
    }

    pub(crate) fn option_id(font: &Handle<Font>) -> (TextFont, TextColor) {
        (
            TextFont {
                font: FontSource::Handle(font.clone()),
                ..option_text(font).0
            },
            TextColor(css::ALICE_BLUE.into()),
        )
    }

    pub(crate) fn option_text(font: &Handle<Font>) -> (TextFont, TextColor) {
        (
            TextFont {
                font_size: FontSize::Px(18.0),
                ..standard(font).0
            },
            TextColor(css::TOMATO.into()),
        )
    }
}
