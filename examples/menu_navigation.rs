use bevy::{color::palettes::css::*, prelude::*};

use bevy_alt_ui_navigation_lite::{
    prelude::{
        DefaultNavigationPlugins, FocusState, Focusable, MenuBuilder, MenuSetting, NavEvent,
        NavRequest, NavRequestSystem,
    },
    systems::InputMapping,
};

/// This example demonstrates a more complex menu system where you navigate
/// through menus and go to submenus using the `Action` and `Cancel`
/// (`SPACE` and `BACKSPACE` on keyboard) requests.
///
/// This introduces the concept of "active" and "dormant" focusable elements.
///
/// The focus goes back to active elements from the parent menu if you request
/// `Cancel` in a given submenu.
///
/// The focus goes back to the child menu's dormant element if you request
/// `Action` while the parent menu's corresponding `Focusable` is focused.
///
/// To navigate to the right column, move focus to the button with the right arrow
/// and press `SPACE`, to navigate to the left, press `BACKSPACE`. Notice how
/// going back to an already explored menu sets the focused element to the last
/// focused one.
///
/// This example also demonstrates the `NavRequest::FocusOn` request. When
/// `SPACE` is pressed when a green circle button is focused, it sends the
/// `FocusOn` request with a first row button as target.
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DefaultNavigationPlugins))
        .init_resource::<Materials>()
        .insert_resource(Gameui::new())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (button_system, handle_nav_events).after(NavRequestSystem),
        )
        .run();
}

#[derive(Resource)]
struct Gameui {
    from: Vec<Entity>,
    to: Entity,
}
impl Gameui {
    pub fn new() -> Self {
        Self {
            from: Vec::new(),
            to: Entity::from_raw(1),
        }
    }
}

#[derive(Resource)]
struct Materials {
    background: Color,
    rarrow: Handle<Image>,
    circle: Handle<Image>,
}

impl FromWorld for Materials {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();
        Materials {
            background: BLACK.into(),
            rarrow: assets.load("rarrow.png"),
            circle: assets.load("green_circle.png"),
        }
    }
}

fn button_system(
    mut interaction_query: Query<(&Focusable, &mut BackgroundColor), Changed<Focusable>>,
) {
    for (focus, mut material) in interaction_query.iter_mut() {
        let color = match focus.state() {
            FocusState::Focused => ORANGE_RED,
            FocusState::Active => GOLD,
            FocusState::Prioritized => GRAY,
            FocusState::Inert => DARK_GRAY,
            FocusState::Blocked => ANTIQUE_WHITE,
        };
        *material = color.into();
    }
}

fn handle_nav_events(
    mut events: EventReader<NavEvent>,
    mut requests: EventWriter<NavRequest>,
    game: Res<Gameui>,
) {
    use NavRequest::Action;
    for event in events.read() {
        if let NavEvent::FocusChanged { from, to } = &event {
            info!("----------\nfrom: {:?}\n  to: {:?}", from, to);
        }
        match event {
            NavEvent::NoChanges {
                from,
                request: Action,
            } if game.from.contains(from.first()) => {
                requests.write(NavRequest::FocusOn(game.to));
            }
            _ => {}
        }
    }
}

fn menu(materials: &Materials) -> impl Bundle {
    (
        Node {
            width: Val::Percent(20.0),
            height: Val::Percent(95.0),
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Stretch,
            ..Default::default()
        },
        BackgroundColor(materials.background),
    )
}
fn setup(
    mut commands: Commands,
    materials: Res<Materials>,
    mut game: ResMut<Gameui>,
    mut input_mapping: ResMut<InputMapping>,
) {
    input_mapping.keyboard_navigation = true;
    // ui camera
    commands.spawn(Camera2d);

    let pct = Val::Percent;
    let node = Node {
        position_type: PositionType::Absolute,
        flex_direction: FlexDirection::Row,
        width: pct(100.),
        height: pct(100.),
        ..Default::default()
    };
    let image_node = Node {
        width: pct(100.),
        height: pct(100.),
        ..Default::default()
    };

    let rarrow = || (ImageNode::new(materials.rarrow.clone()), image_node.clone());
    let circle = || (ImageNode::new(materials.circle.clone()), image_node.clone());

    commands.spawn(node).with_children(|commands| {
        let mut next_menu_button: Option<Entity> = None;
        for j in 0..5 {
            commands
                .spawn((
                    menu(&materials),
                    // Note: when next_menu_button is None,
                    // `with_parent(next_menu_button)` represents the root menu
                    MenuSetting::new().wrapping(),
                    MenuBuilder::from(next_menu_button),
                ))
                .with_children(|commands| {
                    for i in 0..4 {
                        let mut button = commands.spawn(button());
                        button.insert(Focusable::default());
                        if j == 0 && i == 3 {
                            game.to = button.id();
                        }
                        if j == i {
                            button.with_children(|commands| {
                                commands.spawn(rarrow());
                            });
                            next_menu_button = Some(button.id());
                        }
                        if j == 3 && i == 1 {
                            button.insert(Focusable::cancel()).with_children(|cmds| {
                                cmds.spawn(circle());
                            });
                        }
                        if j == 2 && i == 1 {
                            button.insert(Focusable::new().blocked());
                        }
                        if j == 4 {
                            let to_add = button
                                .with_children(|commands| {
                                    commands.spawn(circle());
                                })
                                .id();
                            game.from.push(to_add);
                        }
                    }
                });
        }
    });
}
fn button() -> impl Bundle {
    let pct = Val::Percent;

    (
        Button,
        Node {
            width: pct(95.),
            height: pct(12.),
            margin: UiRect::all(Val::Percent(3.0)),
            ..Default::default()
        },
    )
}
