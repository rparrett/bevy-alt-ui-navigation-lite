use bevy::{color::palettes::css::*, prelude::*};

use bevy_alt_ui_navigation_lite::prelude::*;

/// This example illustrates how to make a button "lock". To lock the UI, press
/// 'A' on controller or 'left click' on mouse when the button with the lock is
/// focused.
///
/// To leave lock mode, press 'escape' on keyboard or 'start' on controller.
/// This will emit a `NavRequest::Unlock` in the default input systems. Allowing
/// the focus to change again.
///
/// It is also possible to lock focus using the `NavRequest::Lock` request.
/// Here, we emit one when the "l" key is pressed.
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DefaultNavigationPlugins))
        .init_resource::<Images>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                extra_lock_key.before(NavRequestSystem),
                (print_nav_events, button_system).after(NavRequestSystem),
            ),
        )
        .run();
}

fn print_nav_events(mut events: EventReader<NavEvent>) {
    for event in events.read() {
        println!("{:?}", event);
    }
}

fn extra_lock_key(mut requests: EventWriter<NavRequest>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::KeyL) {
        requests.send(NavRequest::Lock);
    }
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (&Focusable, &mut BackgroundColor),
        (Changed<Focusable>, With<Button>),
    >,
) {
    for (focus, mut color) in interaction_query.iter_mut() {
        if let FocusState::Focused = focus.state() {
            color.0 = ORANGE_RED.into();
        } else {
            color.0 = DARK_GRAY.into();
        }
    }
}

#[derive(Resource)]
struct Images {
    lock: Handle<Image>,
}
impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();
        Images {
            lock: assets.load("lock.png"),
        }
    }
}

fn setup(mut commands: Commands, imgs: Res<Images>) {
    use Val::Percent as Pct;
    let center_pct = |v: usize| Pct((v as f32) * 25.0 + 25.0);
    // ui camera
    commands.spawn(Camera2d);
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            width: Pct(100.),
            height: Pct(100.),
            ..Default::default()
        })
        .with_children(|commands| {
            for x in 0..3 {
                for y in 0..3 {
                    let bundle = button_bundle(center_pct(x), center_pct(y));
                    let mut button_cmds = commands.spawn(bundle);
                    if x == 1 && y == 1 {
                        // We set the center button as "lock", pressing Action
                        // while it is focused will block the navigation system
                        //                 vvvvvvvvvvvvvvvvv
                        button_cmds.insert(Focusable::lock()).with_children(|cmds| {
                            cmds.spawn(ImageNode::new(imgs.lock.clone()));
                        });
                    } else {
                        button_cmds.insert(Focusable::default());
                    }
                }
            }
        });
}
fn button_bundle(left: Val, bottom: Val) -> impl Bundle {
    (
        Button,
        Node {
            width: Val::Px(95.),
            height: Val::Px(65.),
            left,
            bottom,
            position_type: PositionType::Absolute,
            ..Default::default()
        },
        BackgroundColor(DARK_GRAY.into()),
    )
}
