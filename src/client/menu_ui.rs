use bevy::{ecs::system::EntityCommands, prelude::*};

use bevy::app::AppExit;

use crate::{states::AppState, util::scoped::Scoped};

pub struct MenuUiPlugin;

#[derive(Component, Copy, Clone, Debug, PartialEq)]
enum UiInput {
    Play,
    Exit,
}

impl Plugin for MenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ButtonsPlugin);
        app.add_systems(OnEnter(AppState::Menu), setup);
        app.add_systems(Update, input.run_if(in_state(AppState::Menu)));
    }
}

fn input(
    mut exit: EventWriter<AppExit>,
    mut app: ResMut<NextState<AppState>>,
    mut q: Query<(&Interaction, &UiInput), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, menu_button) in &mut q {
        if Interaction::Pressed == *interaction {
            match menu_button {
                UiInput::Play => app.set(AppState::Connecting),
                UiInput::Exit => exit.send(AppExit),
            }
        }
    }
}

fn setup(mut commands: Commands) {
    let ui_root = commands
        .spawn((
            Scoped(AppState::Menu),
            Name::new("ui-root"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    commands
        .spawn((
            Name::new("ui-button-list"),
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
        ))
        .set_parent(ui_root)
        .with_button("Play", (Name::new("ui-play"), UiInput::Play))
        .with_button("Exit", (Name::new("ui-exit"), UiInput::Exit));
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub struct ButtonsPlugin;

impl Plugin for ButtonsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_buttons);
    }
}

#[derive(Component)]
pub struct AnimatedButton;

fn update_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<AnimatedButton>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
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

pub trait ButtonExt {
    fn with_button(&mut self, text: impl Into<String>, components: impl Bundle) -> &mut Self;
}

impl<'w, 's, 'a> ButtonExt for EntityCommands<'w, 's, 'a> {
    fn with_button(&mut self, text: impl Into<String>, components: impl Bundle) -> &mut Self {
        let text = text.into();
        self.with_children(|commands| {
            commands
                .spawn((
                    AnimatedButton,
                    ButtonBundle {
                        style: Style {
                            width: Val::Auto,
                            height: Val::Auto,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    components,
                ))
                .with_children(|commands| {
                    commands.spawn(TextBundle::from_section(
                        text,
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });
        self
    }
}
