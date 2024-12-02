use bevy::{color::palettes::css::*, prelude::*};

use bevy_alt_ui_navigation_lite::{
    mark::{NavMarker, NavMarkerPropagationPlugin},
    prelude::*,
};

macro_rules! column_type {
    (enum $type_name:ident , $i_base:expr) => {
        #[derive(Component, Clone, Debug)]
        enum $type_name {
            Top,
            Middle,
            Bottom,
        }
        impl $type_name {
            fn i(&self) -> usize {
                match *self {
                    $type_name::Top => $i_base + 0,
                    $type_name::Middle => $i_base + 1,
                    $type_name::Bottom => $i_base + 2,
                }
            }
        }
    };
}
column_type!(enum LeftColMenu, 0);
column_type!(enum CenterColMenu, 3);
column_type!(enum RightColMenu, 6);

/// This example demonstrates the `marker` module features.
///
/// It demonstrates:
/// 1. How to register multiple marking types
/// 2. How to add menu markers that automatically add components to focusables
///    within the menu
/// 3. How to use the marker components to tell menus involved in `NavEvent`
///    events.
///
/// It constructs 9 menus of 3 buttons, you can navigate between them with the
/// leftmost/rightmost buttons in the menus.
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // We must add the NavMarker plugin for each menu marker types we want
            NavMarkerPropagationPlugin::<LeftColMenu>::new(),
            NavMarkerPropagationPlugin::<CenterColMenu>::new(),
            NavMarkerPropagationPlugin::<RightColMenu>::new(),
            DefaultNavigationPlugins,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (button_system, print_menus).after(NavRequestSystem))
        .run();
}

fn print_menus(
    left_menus: Query<&LeftColMenu, Added<Focused>>,
    center_menus: Query<&CenterColMenu, Added<Focused>>,
    right_menus: Query<&RightColMenu, Added<Focused>>,
) {
    // To do something when entering a menu, you use a `Query` on a
    // component specified in the `NavMarkerPropagationPlugin`
    //
    // Notice in `setup` how we DID NOT add any `*ColumnMenus` components to
    // any entity? It is the `NavMarkerPropagationPlugin` that added the
    // components to the focusables within the `MarkingMenu`.
    if let Ok(menu) = left_menus.get_single() {
        println!("Entered Red column menu: {menu:?}");
    }
    if let Ok(menu) = center_menus.get_single() {
        println!("Entered Green column menu: {menu:?}");
    }
    if let Ok(menu) = right_menus.get_single() {
        println!("Entered Blue column menu: {menu:?}");
    }
}

fn button_system(
    mut interaction_query: Query<(&Focusable, &mut BackgroundColor), Changed<Focusable>>,
) {
    for (focus, mut color) in interaction_query.iter_mut() {
        *color = match focus.state() {
            FocusState::Focused => ORANGE.into(),
            FocusState::Active => GOLD.into(),
            FocusState::Prioritized => GRAY.into(),
            FocusState::Inert | FocusState::Blocked => BLACK.into(),
        };
    }
}

fn setup(mut commands: Commands) {
    use FlexDirection::{Column, Row};
    use Val::{Percent as Pct, Px};
    // ui camera
    commands.spawn(Camera2d);

    // First argument to `bndl!` is the color of the node, second is the Style
    macro_rules! bndl {
        ($color:expr, {$($style:tt)*} ) => ((
            Node {
                $($style)*
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                ..Default::default()
            },
            BackgroundColor(Color::from($color).into())
        ))
    }
    let wrap = MenuSetting::new().wrapping();
    let reachable_from = MenuBuilder::EntityParent;
    let good_margin = UiRect::all(Val::Px(20.0));
    // white background
    let root = bndl!(WHITE, {
        width: Pct(100.),
        height: Pct(100.),
        flex_direction: Row,
    });
    // root menu to access each `cell`
    let keyboard = bndl!(DARK_GRAY, {
        width: Px(50.0 * 3.2),
        height: Px(50.0 * 3.2),
        flex_direction: Column,
        flex_wrap: FlexWrap::Wrap,
    });
    // black container
    let billboard = bndl!(BLACK, { flex_direction: Row, margin: good_margin, });
    // colored columns
    let column = |color| bndl!(color, { flex_direction: Column, margin: good_margin, });
    // each row of a column
    let cell = bndl!(Color::srgba(1.0, 1.0, 1.0, 0.2), {
        flex_direction: Row,
        margin: good_margin,
        padding: good_margin,
    });
    // navigable buttons within columns
    let button = bndl!(Color::BLACK, {
        width: Px(40.),
        height: Px(40.),
        margin: UiRect::all(Px(5.0)),
    });
    // spawn nine different buttons for the keyboard menu
    macro_rules! nine {
        ($k:expr) => {
            [$k, $k, $k, $k, $k, $k, $k, $k, $k]
        };
    }
    let bts: [Entity; 9] = nine![commands.spawn((button.clone(), Focusable::default())).id()];
    // create a cell in a column, with three navigable buttons
    macro_rules! spawn_cell {
        ($cmds: expr) => {{
            $cmds.spawn(cell.clone()).with_children(|cmds| {
                let focus = || Focusable::default();
                cmds.spawn((button.clone(), focus()));
                cmds.spawn((button.clone(), focus()));
                cmds.spawn((button.clone(), focus()));
            })
        }};
    }

    // spawn the whole UI tree
    commands.spawn(root).with_children(|cmds| {
        cmds.spawn((
            keyboard,
            MenuSetting::new().wrapping().scope(),
            MenuBuilder::Root, // Add root menu
        ))
        .add_children(&bts);

        cmds.spawn(billboard).with_children(|cmds| {
            // Note: each colored column has a different type, but
            // within each column there are three menus (Top, Middle, Bottom)
            //
            // in `print_menus`, we detect the menu in which we are
            // using the `Query<&LeftColumnMenus>` query.
            //
            // `wrap` = `MenuSetting::Wrapping2d`, see type alias on top of this
            // function.
            cmds.spawn(column(RED)).with_children(|cmds| {
                let menu = |row: LeftColMenu| (wrap, reachable_from(bts[row.i()]), NavMarker(row));
                spawn_cell!(cmds).insert(menu(LeftColMenu::Top));
                spawn_cell!(cmds).insert(menu(LeftColMenu::Middle));
                spawn_cell!(cmds).insert(menu(LeftColMenu::Bottom));
            });
            cmds.spawn(column(GREEN)).with_children(|cmds| {
                let menu =
                    |row: CenterColMenu| (wrap, reachable_from(bts[row.i()]), NavMarker(row));
                spawn_cell!(cmds).insert(menu(CenterColMenu::Top));
                spawn_cell!(cmds).insert(menu(CenterColMenu::Middle));
                spawn_cell!(cmds).insert(menu(CenterColMenu::Bottom));
            });
            cmds.spawn(column(BLUE)).with_children(|cmds| {
                let menu = |row: RightColMenu| (wrap, reachable_from(bts[row.i()]), NavMarker(row));
                spawn_cell!(cmds).insert(menu(RightColMenu::Top));
                spawn_cell!(cmds).insert(menu(RightColMenu::Middle));
                spawn_cell!(cmds).insert(menu(RightColMenu::Bottom));
            });
        });
    });
}
