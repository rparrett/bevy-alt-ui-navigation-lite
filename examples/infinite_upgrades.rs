use std::fmt;

use bevy::color::palettes::css::*;
use bevy::ecs::system::EntityCommands;
use bevy::math::FloatOrd;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use bevy_alt_ui_navigation_lite::{
    prelude::*,
    systems::{default_gamepad_input, InputMapping},
};

/// This example demonstrates how to generate on the fly focusables to navigate.
fn main() {
    App::new()
        // Add your own cursor navigation system
        // by using `NavigationPlugin::<MyOwnNavigationStrategy>::new()`
        // See the [`bevy_alt_ui_navigation_lite::MenuNavigationStrategy`] trait.
        //
        // You can use a custom gamepad directional handling system if you want to.
        // This could be useful if you want such navigation in 3d space
        // to take into consideration the 3d camera perspective.
        //
        // Here we use the default one provided by `bevy_ui` because
        // it is already capable of handling navigation in 2d space
        // (even using `Sprite` over UI `Node`)
        .add_plugins((DefaultPlugins, NavigationPlugin::new()))
        // Since gamepad input already works for Sprite-based menus,
        // we add back the default gamepad input handling from `bevy_ui`.
        // default_gamepad_input depends on NavigationInputMapping so we
        // need to also add this resource back.
        .init_resource::<InputMapping>()
        .add_systems(
            Update,
            (
                (default_gamepad_input, mouse_pointer_system).before(NavRequestSystem),
                (
                    (upgrade_weapon, button_system).after(NavRequestSystem),
                    handle_menu_change,
                    animate_system,
                )
                    .chain(),
            ),
        )
        .add_systems(PostUpdate, mark_buttons)
        .add_systems(Startup, setup)
        // Our systems.
        .init_resource::<MenuMap>()
        .run();
}

/// Base color to swap back to when a focusable unfocuses.
#[derive(Component)]
struct BaseColor(Color);

/// The menu entity.
#[derive(Component)]
struct Menu {
    weapon: Weapon,
    position: IVec2,
}

/// Where to spawn the new menu relative to the current one.
#[derive(Component, Debug)]
enum SpawnDirection {
    Left,
    Right,
    Bottom,
}
impl SpawnDirection {
    const fn as_ivec2(&self) -> IVec2 {
        match self {
            SpawnDirection::Left => IVec2::NEG_X,
            SpawnDirection::Right => IVec2::X,
            SpawnDirection::Bottom => IVec2::NEG_Y,
        }
    }
}

/// Component to add to button sprites to select which upgrade to apply to the weapon.
#[derive(Component)]
enum WeaponUpgrade {
    Increment,
    Prefix(&'static str),
    Suffix(&'static str),
}
impl fmt::Display for WeaponUpgrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WeaponUpgrade::Increment => write!(f, "+1"),
            WeaponUpgrade::Prefix(prefix) => write!(f, "{prefix}"),
            WeaponUpgrade::Suffix(suffix) => write!(f, "of {suffix}"),
        }
    }
}

/// Used in [`Weapon`], chooses whether the number is in roman numeral form
/// or +arabic number.
#[derive(Debug, Clone, Copy)]
enum Upgrade {
    Roman(i32),
    Plus(i32),
}
impl Upgrade {
    fn increment(&mut self) {
        let (Self::Plus(x) | Self::Roman(x)) = self;
        *x += 1;
    }
}
impl fmt::Display for Upgrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Upgrade::Plus(i) => write!(f, "+{i}"),
            Upgrade::Roman(roman) => write!(f, "{}", make_roman(*roman)),
        }
    }
}

/// A sparse grid of menus, used to find whether
/// it is possible to spawn
#[derive(Resource, Debug, Default)]
struct MenuMap {
    grid: HashMap<IVec2, Entity>,
}
impl MenuMap {
    fn is_free(&self, at: IVec2) -> bool {
        !self.grid.contains_key(&at)
    }
}

/// Animate anything. Used to move the camera smoothly.
///
/// See the `animate_system` for how this is used.
#[derive(Component, Debug, Clone, Copy, Default)]
enum Animate {
    /// Moves the thing on the XY plane toward `target` at `speed` unit per second.
    MoveToward { target: Vec2, speed: f32 },
    /// Shake the camera along `direction` until `until` with a forward/backward period of `period`.
    Shake {
        until: f64,
        direction: Vec2,
        period: f64,
    },
    #[default]
    None,
}

// === === ===
//
// Define custom navigation
//
// === === ===

trait ScreenSize {
    fn size(&self) -> Vec2;
}
impl ScreenSize for Sprite {
    fn size(&self) -> Vec2 {
        self.custom_size.unwrap_or_default()
    }
}
fn is_in_sizeable(at: Vec2, transform: &GlobalTransform, sizeable: &impl ScreenSize) -> bool {
    let ui_pos = transform.translation().truncate();
    let node_half_size = sizeable.size() / 2.0;
    let min = ui_pos - node_half_size;
    let max = ui_pos + node_half_size;
    (min.x..max.x).contains(&at.x) && (min.y..max.y).contains(&at.y)
}

// Since we do not use UI nodes for navigation, but instead 2d sprites,
// we need to define our own mouse pointer system.
//
// One additional complexity is that since we move the camera,
// we have to account for it in the mouse picking system.
//
// TODO: make some functions in bevy_ui/navigation/systems.rs public so that
// this is more user-friendly.
pub fn mouse_pointer_system(
    camera: Query<(&GlobalTransform, &Camera), With<Camera2d>>,
    camera_moving: Query<(), (Changed<GlobalTransform>, With<Camera2d>)>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    focusables: Query<(&GlobalTransform, &Sprite, Entity), With<Focusable>>,
    focused: Query<Entity, With<Focused>>,
    mut nav_cmds: EventWriter<NavRequest>,
) {
    // If the camera is currently moving, skip mouse pointing
    if camera_moving.iter().next().is_some() {
        return;
    }
    let Ok(window) = primary_query.get_single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Some((camera_transform, camera)) = camera.iter().next() else {
        return;
    };
    let Ok(world_cursor_pos) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };
    let world_cursor_pos = world_cursor_pos.get_point(0.0).truncate();
    let released = mouse.just_released(MouseButton::Left);
    let pressing = mouse.pressed(MouseButton::Left);
    let Ok(focused) = focused.get_single() else {
        return;
    };
    let under_mouse = focusables
        .iter()
        .filter(|(transform, sprite, _)| is_in_sizeable(world_cursor_pos, transform, *sprite))
        .max_by_key(|elem| FloatOrd(elem.0.translation().z))
        .map(|elem| elem.2);
    let Some(to_target) = under_mouse else {
        return;
    };
    let hover_focused = under_mouse == Some(focused);
    if (pressing || released) && !hover_focused {
        nav_cmds.send(NavRequest::FocusOn(to_target));
    }
    if released {
        nav_cmds.send(NavRequest::Action);
    }
}

// === === ===
//
// Some fancy code.
//
// === === ===

/// ```rust
/// assert_eq!(make_roman(2022), "MMXXII".to_owned());
/// assert_eq!(make_roman(101), "CI".to_owned());
/// assert_eq!(make_roman(5), "V".to_owned());
/// assert_eq!(make_roman(1), "I".to_owned());
/// assert_eq!(make_roman(19), "XIX".to_owned());
/// ```
fn make_roman(mut to_romanize: i32) -> String {
    const SYMBOLS: &[u8] = "MDCLXVI".as_bytes();
    const VALUES: &[i32] = &[1000, 500, 100, 50, 10, 5, 1];

    let at_index: fn(usize) -> (char, i32) = |i| (SYMBOLS[i].into(), VALUES[i]);
    let mut current_index = 0;
    let mut ret = String::with_capacity(3);
    loop {
        let (symbol, value) = at_index(current_index);
        while to_romanize >= value {
            ret.push(symbol);
            to_romanize -= value;
        }
        if to_romanize <= 0 {
            return ret;
        }
        let is_pow10 = current_index % 2 == 0;
        let next_index = current_index + if is_pow10 { 2 } else { 1 };
        let (next, next_value) = at_index(next_index);
        if to_romanize + next_value >= value {
            ret.push(next);
            to_romanize += next_value;
        } else {
            current_index += 1;
        }
    }
}

#[derive(Debug, Clone)]
struct Weapon {
    upgrade: Upgrade,
    prefixes: Vec<&'static str>,
    suffixes: Vec<&'static str>,
    name: &'static str,
}

impl Weapon {
    fn new(name: &'static str, upgrade: Upgrade) -> Self {
        Self {
            upgrade,
            name,
            prefixes: default(),
            suffixes: default(),
        }
    }
}
impl fmt::Display for Weapon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            upgrade,
            prefixes,
            suffixes,
            name,
        } = self;
        for prefix in prefixes {
            write!(f, "{prefix} ")?;
        }
        write!(f, "{name}")?;
        for suffix in suffixes {
            write!(f, " of {suffix}")?;
        }
        write!(f, " {upgrade}")?;
        Ok(())
    }
}

const BUTTON_HPADDING: f32 = 5.0;
const BUTTON_WIDTH: f32 = 200.0;
const CAMERA_SPEED: f32 = 1200.0;
const FONT_SIZE: f32 = 30.0;
const MENU_GAP: f32 = 30.0;
const MENU_HEIGHT: f32 = 150.0;
const MENU_PADDING: f32 = 10.0;
const MENU_WIDTH: f32 = 700.0;

const NAMES: &[&str] = &[
    "flower",
    "staff",
    "sword",
    "club",
    "ball",
    "flower pot",
    "whistle",
    "flute",
    "triangle",
    "battle axe",
    "skull",
    "bludgeon",
    "bread",
];
const QUALIFICATIVES: &[&str] = &[
    "royalty",
    "diamond",
    "giga",
    "extra",
    "mega",
    "fire",
    "water",
    "air",
    "earth",
    "dwarf",
    "troll",
    "gnome",
    "mimicry",
    "wisdom",
    "destruction",
    "intelligence",
    "swiftness",
    "agility",
    "speed",
    "strength",
    "power",
    "dog",
    "cat",
    "bird",
    "giraffe",
];

// === === ===
//
// Manage an infinitely growing menu tree of weapon upgrades
//
// === === ===

fn setup(mut commands: Commands, mut menus: ResMut<MenuMap>) {
    let name = NAMES[fastrand::usize(0..NAMES.len())];
    let upgrade = if fastrand::bool() {
        Upgrade::Roman(1)
    } else {
        Upgrade::Plus(1)
    };
    let weapon = Weapon::new(name, upgrade);
    commands.spawn((Camera2d::default(), Animate::default()));
    let at = IVec2::ZERO;
    let menu = spawn_weapon_upgrade_menu(&mut commands, at, &weapon, None);
    menus.grid.insert(at, menu);
}

/// Update "button" (sprites) color based on their focus state.
fn button_system(
    mut interaction_query: Query<(&Focusable, &BaseColor, &mut Sprite), Changed<Focusable>>,
) {
    for (focus, base_color, mut sprite) in interaction_query.iter_mut() {
        let color = match focus.state() {
            FocusState::Focused => PINK.into(),
            FocusState::Active => GOLD.into(),
            FocusState::Prioritized => ORANGE_RED.into(),
            FocusState::Inert => base_color.0,
            FocusState::Blocked => DARK_GRAY.into(),
        };
        sprite.color = color;
    }
}

/// Handles the [`Animate`] component.
fn animate_system(mut animated: Query<(&Animate, &mut Transform)>, time: Res<Time>) {
    let delta = time.delta_secs();
    let current_time = time.elapsed_secs_f64();
    for (animate, mut transform) in &mut animated {
        let current_z = transform.translation.z;
        let current = transform.translation.xy();
        match animate {
            Animate::None => {}
            &Animate::MoveToward { target, speed } => {
                let diff = target - current;
                let diff_len = diff.length_squared();
                if diff_len > 0.5 {
                    // move toward target without overshooting it.
                    let distance_traversed = diff_len.sqrt().min(delta * speed);
                    let traversed = distance_traversed * diff.normalize_or_zero();
                    let new_position = current + traversed;
                    transform.translation = new_position.extend(current_z);
                }
            }
            &Animate::Shake {
                until,
                direction,
                period,
            } if until > current_time => {
                let sign = current_time % period < period / 2.0;
                let sign = if sign { 1.0 } else { -1.0 };
                let new_position = current + direction * sign;
                transform.translation = new_position.extend(current_z);
            }
            Animate::Shake { .. } => {}
        }
    }
}

/// Move camera to the menu that is currently focused if the focus changed menu.
fn handle_menu_change(
    mut nav_events: EventReader<NavEvent>,
    mut cam: Query<&mut Animate, With<Camera2d>>,
    menu_position: Query<&GlobalTransform, With<Menu>>,
    menu_query: Query<&ParentMenu>,
) {
    for event in nav_events.read() {
        if let NavEvent::FocusChanged { to, from } = event {
            let menu_query = (menu_query.get(*from.first()), menu_query.get(*to.first()));
            if let (Ok(from), Ok(to)) = menu_query {
                if from.0 != to.0 {
                    let menu_pos = match menu_position.get(to.0) {
                        Ok(pos) => pos,
                        Err(_) => continue,
                    };
                    let mut animate = match cam.get_single_mut() {
                        Ok(cam) => cam,
                        Err(_) => continue,
                    };
                    let target = menu_pos.translation().xy();
                    *animate = Animate::MoveToward {
                        target,
                        speed: CAMERA_SPEED,
                    };
                }
            }
        }
    }
}

/// Handle generating new menus when an upgrade is selected.
fn upgrade_weapon(
    mut commands: Commands,
    mut events: EventReader<NavEvent>,
    mut requests: EventWriter<NavRequest>,
    (mut menus, time): (ResMut<MenuMap>, Res<Time>),
    mut cam: Query<&mut Animate, With<Camera2d>>,
    query: Query<(&ParentMenu, &WeaponUpgrade, &SpawnDirection, Entity)>,
    menu_data: Query<&Menu>,
) {
    for (&ParentMenu(current_menu), upgrade, direction, entity) in
        events.nav_iter().activated_in_query(&query)
    {
        let menu = menu_data.get(current_menu).unwrap();
        let mut weapon = menu.weapon.clone();
        match upgrade {
            WeaponUpgrade::Increment => weapon.upgrade.increment(),
            WeaponUpgrade::Prefix(prefix) => weapon.prefixes.push(*prefix),
            WeaponUpgrade::Suffix(suffix) => weapon.suffixes.push(*suffix),
        }
        let at = menu.position + direction.as_ivec2();
        if menus.is_free(at) {
            // Exercise to the reader: write an alternate system that does not use
            // `Menu.weapon`, but instead reads the `WeaponUpgrade` component of all
            // focusable in the `from` field of `NavEvent::NoChanges` to generate
            // the current weapon upgrade.
            let menu = spawn_weapon_upgrade_menu(&mut commands, at, &weapon, Some(entity));
            menus.grid.insert(at, menu);
            requests.send(NavRequest::Action);
        } else {
            let direction = direction.as_ivec2().as_vec2();
            let mut animate = match cam.get_single_mut() {
                Ok(cam) => cam,
                Err(_) => continue,
            };
            let half_second = time.elapsed_secs_f64() + 0.5;
            *animate = Animate::Shake {
                until: half_second,
                direction,
                period: 0.15,
            }
        }
    }
}

/// Boilerplate to create a `Sprite` with some text inside of it.
fn spawn_button(commands: &mut EntityCommands, color: Color, at: Vec2, text: String) {
    let item_position = |at: Vec2| Transform::from_translation(at.extend(0.05));
    commands
        .insert((
            Sprite {
                color,
                custom_size: Some(Vec2::new(BUTTON_WIDTH, FONT_SIZE + 2.0 * BUTTON_HPADDING)),
                ..default()
            },
            item_position(at),
            BaseColor(color),
        ))
        .with_children(|commands| {
            commands.spawn((
                Text2d::new(text),
                TextFont {
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
                item_position(Vec2::ZERO),
            ));
        });
}

// TODO: block the buttons that go toward unavailable space
/// Create an upgrade menu with the weapon name and the upgrade buttons.
fn spawn_weapon_upgrade_menu(
    commands: &mut Commands,
    position: IVec2,
    weapon: &Weapon,
    parent: Option<Entity>,
) -> Entity {
    let quals = QUALIFICATIVES.len();
    let suffix = QUALIFICATIVES[fastrand::usize(0..quals)];
    let prefix = QUALIFICATIVES[fastrand::usize(0..quals)];

    let menu_grid_offset = Vec2::new(MENU_WIDTH, MENU_HEIGHT) + MENU_GAP;
    let at = position.as_vec2() * menu_grid_offset;
    let item_position = |at: Vec2| Transform::from_translation(at.extend(0.1));
    // Rectangle
    commands
        .spawn((
            Sprite {
                // TODO: random color
                color: Color::srgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(MENU_WIDTH, MENU_HEIGHT)),
                ..default()
            },
            item_position(at),
            Menu {
                weapon: weapon.clone(),
                position,
            },
            MenuSetting::new(),
            MenuBuilder::from(parent),
            MarkButtons,
        ))
        .with_children(|commands| {
            // Weapon name
            commands.spawn((
                Text2d::new(weapon.to_string()),
                TextFont {
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
                item_position(Vec2::Y * (MENU_HEIGHT / 2.0 - MENU_PADDING - FONT_SIZE / 2.0)),
            ));

            // buttons
            let upgrades = [
                (WeaponUpgrade::Prefix(prefix), SpawnDirection::Left),
                (WeaponUpgrade::Suffix(suffix), SpawnDirection::Bottom),
                (WeaponUpgrade::Increment, SpawnDirection::Right),
            ];
            let upgrade_count = upgrades.len();
            let x_padding = 0.5 / upgrade_count as f32;
            let button_y = MENU_HEIGHT / 2.0 - MENU_PADDING - BUTTON_HPADDING - FONT_SIZE / 2.0;

            for (i, (upgrade, direction)) in upgrades.into_iter().enumerate() {
                let x_offset = i as f32 / upgrade_count as f32;
                let button_x = (x_offset - 0.5 + x_padding) * (MENU_WIDTH - MENU_PADDING);
                let button_pos = Vec2::new(button_x, -button_y);
                let text = upgrade.to_string();
                let mut entity = commands.spawn((Focusable::default(), upgrade, direction));
                spawn_button(&mut entity, CRIMSON.into(), button_pos, text);
            }
        })
        .id()
}

#[derive(Component, Clone, Copy, PartialEq)]
struct ParentMenu(Entity);

#[derive(Component)]
struct MarkButtons;

// TODO: note that bevy-ui-navigation had a dedicated module to automate this.
// It could be added to bevy_hierarchy
/// This system adds a component that links directly to the parent menu of a focusable.
fn mark_buttons(
    mut cmds: Commands,
    menu_markers: Query<Entity, With<MarkButtons>>,
    focusables: Query<(), With<Focusable>>,
    menus: Query<(), With<MenuSetting>>,
    children: Query<&Children>,
) {
    fn mark_focusable(
        entity_children: &Children,
        marker: ParentMenu,
        commands: &mut Commands,
        focusables: &Query<(), With<Focusable>>,
        menus: &Query<(), With<MenuSetting>>,
        children: &Query<&Children>,
    ) {
        for entity in entity_children {
            match () {
                () if focusables.get(*entity).is_ok() => {
                    commands.entity(*entity).insert(marker);
                }
                () if menus.get(*entity).is_ok() => {}
                () => {
                    if let Ok(entities) = children.get(*entity) {
                        mark_focusable(entities, marker, commands, focusables, menus, children);
                    }
                }
            }
        }
    }
    for menu in &menu_markers {
        if let Ok(entities) = children.get(menu) {
            let marker = ParentMenu(menu);
            mark_focusable(entities, marker, &mut cmds, &focusables, &menus, &children);
        }
        cmds.entity(menu).remove::<MarkButtons>();
    }
}
