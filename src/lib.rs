/*!
[`ButtonBundle`]: bevy::prelude::ButtonBundle
[Changed]: bevy::prelude::Changed
[doc-root]: ./index.html
[`Entity`]: bevy::prelude::Entity
[entity-id]: bevy::ecs::system::EntityCommands::id
[`FocusableButtonBundle`]: components::FocusableButtonBundle
[`Focusable::cancel`]: resolve::Focusable::cancel
[`Focusable::block`]: resolve::Focusable::block
[`Focusable::dormant`]: resolve::Focusable::dormant
[`Focusable`]: resolve::Focusable
[`Focusable::lock`]: resolve::Focusable::lock
[`generic_default_mouse_input`]: systems::generic_default_mouse_input
[`InputMapping`]: systems::InputMapping
[`InputMapping::keyboard_navigation`]: systems::InputMapping::keyboard_navigation
[module-event_helpers]: events::NavEventReaderExt
[module-marking]: mark
[module-systems]: systems
[Name]: bevy::core::Name
[`NavEvent::FocusChanged`]: events::NavEvent::FocusChanged
[`NavEvent`]: events::NavEvent
[`NavEvent::InitiallyFocused`]: events::NavEvent::InitiallyFocused
[`MenuSetting`]: menu::MenuSetting
[`NavMenu`]: menu::MenuSetting
[`MenuBuilder`]: menu::MenuBuilder
[MenuBuilder::reachable_from]: menu::MenuBuilder::EntityParent
[MenuBuilder::reachable_from_named]: menu::MenuBuilder::from_named
[`NavRequest`]: events::NavRequest
[`NavRequest::Action`]: events::NavRequest::Action
[`NavRequest::FocusOn`]: events::NavRequest::FocusOn
[`NavRequest::Free`]: events::NavRequest::Unlock
[`NavRequest::Unlock`]: events::NavRequest::Unlock
[`NavRequest::ScopeMove`]: events::NavRequest::ScopeMove
[`NavRequestSystem`]: NavRequestSystem
*/
#![doc = include_str!("../Readme.md")]
#![forbid(missing_docs)]
#![allow(clippy::unnecessary_lazy_evaluations)]

mod commands;
pub mod events;
mod marker;
pub mod menu;
mod named;
mod resolve;
pub mod systems;

use std::marker::PhantomData;

use bevy::ecs::system::{SystemParam, SystemParamItem};
use bevy::prelude::*;

pub use non_empty_vec::NonEmpty;

use resolve::UiProjectionQuery;

/// Default imports for `bevy_alt_ui_navigation_lite`.
pub mod prelude {
    pub use crate::events::{NavEvent, NavEventReaderExt, NavRequest};
    pub use crate::menu::{MenuBuilder, MenuSetting};
    pub use crate::resolve::{
        FocusAction, FocusState, Focusable, Focused, MenuNavigationStrategy, NavLock,
    };
    pub use crate::NavRequestSystem;
    pub use crate::{DefaultNavigationPlugins, NavigationPlugin};
}
/// Utilities to mark focusables within a menu with a specific component.
pub mod mark {
    pub use crate::menu::NavMarker;
    pub use crate::NavMarkerPropagationPlugin;
}
/// Types useful to define your own custom navigation inputs.
pub mod custom {
    pub use crate::resolve::UiProjectionQuery;
    pub use crate::resolve::{Rect, ScreenBoundaries};
    pub use crate::GenericNavigationPlugin;
}

/// Plugin for menu marker propagation.
///
/// For a marker of type `T` to be propagated when using
/// [`mark::NavMarker`], you need to add a
/// `NavMarkerPropagationPlugin<T>` to your bevy app. It is possible to add any
/// amount of `NavMarkerPropagationPlugin<T>` for as many `T` you need to
/// propagate through the menu system.
pub struct NavMarkerPropagationPlugin<T>(PhantomData<T>);
impl<T> NavMarkerPropagationPlugin<T> {
    #[allow(clippy::new_without_default)]
    /// Create a new [`NavMarkerPropagationPlugin`].
    pub fn new() -> Self {
        NavMarkerPropagationPlugin(PhantomData)
    }
}

impl<T: 'static + Sync + Send + Component + Clone> Plugin for NavMarkerPropagationPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                marker::mark_new_menus::<T>,
                marker::mark_new_focusables::<T>,
            ),
        );
    }
}

/// The label of the system in which the [`NavRequest`] events are handled, the
/// focus state of the [`Focusable`]s is updated and the [`NavEvent`] events
/// are sent.
///
/// Systems updating visuals of UI elements should run _after_ the `NavRequestSystem`,
/// while systems that emit [`NavRequest`] should run _before_ it.
/// For example, an input system should run before the `NavRequestSystem`.
///
/// Failing to do so won't cause logical errors, but will make the UI feel more slugish
/// than necessary. This is especially critical of you are running on low framerate.
///
/// # Example
///
/// ```rust, no_run
/// use bevy_alt_ui_navigation_lite::prelude::*;
/// use bevy_alt_ui_navigation_lite::events::Direction;
/// use bevy_alt_ui_navigation_lite::custom::GenericNavigationPlugin;
/// use bevy::prelude::*;
/// # use std::marker::PhantomData;
/// # use bevy::ecs::system::SystemParam;
/// # #[derive(SystemParam)] struct MoveCursor3d<'w, 's> {
/// #   _foo: PhantomData<(&'w (), &'s ())>
/// # }
/// # impl<'w, 's> MenuNavigationStrategy for MoveCursor3d<'w, 's> {
/// #   fn resolve_2d<'a>(
/// #       &self,
/// #       focused: Entity,
/// #       direction: Direction,
/// #       cycles: bool,
/// #       siblings: &'a [Entity],
/// #   ) -> Option<&'a Entity> { None }
/// # }
/// # fn button_system() {}
/// fn main() {
///     App::new()
///         .add_plugins(GenericNavigationPlugin::<MoveCursor3d>::new())
///         // ...
///         // Add the button color update system after the focus update system
///         .add_systems(Update, button_system.after(NavRequestSystem))
///         // ...
///         .run();
/// }
/// ```
///
/// [`NavRequest`]: prelude::NavRequest
/// [`NavEvent`]: prelude::NavEvent
/// [`Focusable`]: prelude::Focusable
#[derive(Clone, Debug, Hash, PartialEq, Eq, SystemSet)]
pub struct NavRequestSystem;

/// The navigation plugin.
///
/// Add it to your app with `.add_plugins(NavigationPlugin::new())` and send
/// [`NavRequest`]s to move focus within declared [`Focusable`] entities.
///
/// You should prefer `bevy_ui` provided defaults
/// if you don't want to bother with that.
///
/// # Note on generic parameters
///
/// The `STGY` type parameter might seem complicated, but all you have to do
/// is for your type to implement [`SystemParam`] and [`MenuNavigationStrategy`].
///
/// [`MenuNavigationStrategy`]: resolve::MenuNavigationStrategy
/// [`Focusable`]: prelude::Focusable
/// [`NavRequest`]: prelude::NavRequest
#[derive(Default)]
pub struct GenericNavigationPlugin<STGY>(PhantomData<fn() -> STGY>);
/// A default [`GenericNavigationPlugin`] for `bevy_ui`.
pub type NavigationPlugin<'w, 's> = GenericNavigationPlugin<UiProjectionQuery<'w, 's>>;

impl<STGY: resolve::MenuNavigationStrategy> GenericNavigationPlugin<STGY> {
    /// Create a new [`GenericNavigationPlugin`] with the provided `STGY`,
    /// see also [`resolve::MenuNavigationStrategy`].
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
impl<STGY: SystemParam + 'static> Plugin for GenericNavigationPlugin<STGY>
where
    for<'w, 's> SystemParamItem<'w, 's, STGY>: resolve::MenuNavigationStrategy,
{
    fn build(&self, app: &mut App) {
        #[cfg(feature = "bevy_reflect")]
        app.register_type::<menu::MenuBuilder>()
            .register_type::<menu::MenuSetting>()
            .register_type::<resolve::Focusable>()
            .register_type::<resolve::FocusAction>()
            .register_type::<resolve::FocusState>()
            .register_type::<resolve::LockReason>()
            .register_type::<resolve::NavLock>()
            .register_type::<resolve::Rect>()
            .register_type::<resolve::ScreenBoundaries>()
            .register_type::<resolve::TreeMenu>()
            .register_type::<systems::InputMapping>();

        app.add_event::<events::NavRequest>()
            .add_event::<events::NavEvent>()
            .insert_resource(resolve::NavLock::new())
            .add_systems(
                Update,
                (
                    (resolve::set_first_focused, resolve::consistent_menu),
                    resolve::listen_nav_requests::<STGY>.in_set(NavRequestSystem),
                )
                    .chain(),
            )
            .add_systems(
                PreUpdate,
                (named::resolve_named_menus, resolve::insert_tree_menus).chain(),
            );
    }
}

/// The navigation plugin and the default input scheme.
///
/// Add it to your app with `.add_plugins(DefaultNavigationPlugins)`.
///
/// This provides default implementations for input handling, if you want
/// your own custom input handling, you should use [`NavigationPlugin`] and
/// provide your own input handling systems.
pub struct DefaultNavigationPlugins;
impl PluginGroup for DefaultNavigationPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        bevy::app::PluginGroupBuilder::start::<Self>()
            .add(NavigationPlugin::new())
            .add(systems::DefaultNavigationSystems)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use bevy::{ecs::event::Event, prelude::*};

    use super::*;
    // Why things might fail?
    // -> State becomes inconsistent, assumptions are broken
    // How would assumptions be broken?
    // -> The ECS hierarchy changed under our feet
    // -> state was modified by users and we didn't expect it
    // -> internal state is not updated correctly to reflect the actual state
    // Consistency design:
    // - Strong dependency on bevy hierarchy not being mucked with
    //   (doesn't handle changed parents well)
    // - Need to get rid of TreeMenu::active_child probably
    // - Possible to "check and fix" the state in a system that accepts
    //   Changed<Parent> + RemovedComponent<Focusable | TreeMenu | Parent>
    // - But the check cannot anticipate when the hierarchy is changed,
    //   so we are doomed to expose to users inconsistent states
    //   -> implication: we don't need to maintain it in real time, since
    //      after certain hierarchy manipulations, it will be inconsistent either way.
    //      So we could do with only checking and updating when receiving
    //      NavRequest (sounds like good use case for system chaining)

    /// Define a menu structure to spawn.
    ///
    /// This just describes the menu structure,  use [`SpawnHierarchy::spawn`],
    /// to spawn the entities in the world,.
    enum SpawnHierarchy {
        Rootless(SpawnRootless),
        Menu(SpawnMenu),
    }
    impl SpawnHierarchy {
        fn spawn(self, world: &mut World) {
            match self {
                Self::Rootless(menu) => menu.spawn(world),
                Self::Menu(menu) => menu.spawn(&mut world.spawn_empty()),
            };
        }
    }
    struct SpawnFocusable {
        name: &'static str,
        prioritized: bool,
        child_menu: Option<SpawnMenu>,
    }

    impl SpawnFocusable {
        fn spawn(self, mut entity: EntityWorldMut) {
            let SpawnFocusable {
                name,
                prioritized,
                child_menu,
            } = self;
            entity.insert(Name::new(name));
            let focusable = if prioritized {
                Focusable::new().prioritized()
            } else {
                Focusable::new()
            };
            entity.insert(focusable);
            if let Some(child_menu) = child_menu {
                // SAFETY: we do not call any methods on `entity` after `world_mut()`
                unsafe {
                    child_menu.spawn(&mut entity.world_mut().spawn_empty());
                };
                std::mem::drop(entity);
            }
        }
    }
    struct SpawnMenu {
        name: &'static str,
        children: Vec<SpawnFocusable>,
    }
    impl SpawnMenu {
        fn spawn(self, entity: &mut EntityWorldMut) {
            let SpawnMenu { name, children } = self;
            let parent_focusable = name.strip_suffix(" Menu");
            let menu_builder = match parent_focusable {
                Some(name) => MenuBuilder::from_named(name),
                None => MenuBuilder::Root,
            };
            entity.insert((Name::new(name), menu_builder, MenuSetting::new()));
            entity.with_children(|commands| {
                for child in children.into_iter() {
                    child.spawn(commands.spawn_empty());
                }
            });
        }
    }
    struct SpawnRootless {
        focusables: Vec<SpawnFocusable>,
    }
    impl SpawnRootless {
        fn spawn(self, world: &mut World) {
            for focusable in self.focusables.into_iter() {
                focusable.spawn(world.spawn_empty())
            }
        }
    }
    /// Define a `SpawnHierarchy`.
    ///
    /// Syntax:
    /// - `spawn_hierarchy![ <focus_kind>, ... ]`:
    ///   A hierarchy of focusable components with a root menu.
    /// - `spawn_hierarchy!(@rootless [ <focus_kind> , ...] )`:
    ///   A hierarchy of focusable components **without** a root menu.
    /// - `<focus_kind>` is one of the following:
    ///   - `focusable("Custom")`: a focusable with the `Name::new("Custom")` component
    ///   - `focusable_to("Custom" [ <focus_kind> , ...] )`:
    ///     a focusable with the `Name::new("Custom")` component, parent of a menu (`MenuBuilder`)
    ///     marked with the `Name::new("Custom Menu")` component. The menu content is the
    ///     content of the provided list
    ///   - `prioritized("Custom")`: a focusable with the `Name::new("Custom")` component,
    ///     spawned with `Focusable::new().prioritized()`.
    macro_rules! spawn_hierarchy {
        ( @rootless [ $( $elem_kind:ident $elem_args:tt ),* $(,)? ] ) => (
            SpawnHierarchy::Rootless(SpawnRootless {
                focusables: vec![ $(
                    spawn_hierarchy!(@elem $elem_kind $elem_args),
                )* ],
            })
        );
        ( @menu $name:expr, $( $elem_name:ident $elem_args:tt ),* $(,)? ) => (
            SpawnMenu {
                name: $name,
                children: vec![ $(
                    spawn_hierarchy!(@elem $elem_name $elem_args),
                )* ],
            }
        );
        ( @elem prioritized ( $name:literal ) ) => (
            SpawnFocusable {
                name: $name,
                prioritized: true,
                child_menu: None,
            }
        );
        ( @elem focusable ( $name:literal ) ) => (
            SpawnFocusable {
                name: $name,
                prioritized: false,
                child_menu: None,
            }
        );
        ( @elem focusable_to ( $name:literal [ $( $submenu:tt )* ] ) ) => (
            SpawnFocusable {
                name: $name,
                prioritized: false,
                child_menu: Some( spawn_hierarchy!(@menu concat!( $name , " Menu"),  $( $submenu )* ) ),
            }
        );
        ($( $elem_name:ident $elem_args:tt ),* $(,)? ) => (
            SpawnHierarchy::Menu(spawn_hierarchy!(@menu "Root", $( $elem_name $elem_args ),*))
        );
    }

    /// Assert identity of a list of entities by their `Name` component
    /// (makes understanding test failures easier)
    ///
    /// This is a macro, so that when there is an assert failure or panic,
    /// the line of code it points to is the calling site,
    /// rather than the function body.
    ///
    /// There is nothing beside that that would prevent converting this into a function.
    macro_rules! assert_expected_focus_change {
        ($app:expr, $events:expr, $expected_from:expr, $expected_to:expr $(,)?) => {
            if let [NavEvent::FocusChanged { to, from }] = $events {
                let actual_from = $app.name_list(&*from);
                assert_eq!(&*actual_from, $expected_from);

                let actual_to = $app.name_list(&*to);
                assert_eq!(&*actual_to, $expected_to);
            } else {
                panic!(
                    "Expected a signle FocusChanged NavEvent, got: {:#?}",
                    $events
                );
            }
        };
    }

    // A navigation strategy that does nothing, useful for testing.
    #[derive(SystemParam)]
    struct MockNavigationStrategy<'w, 's> {
        _f: PhantomData<fn() -> (&'w (), &'s ())>,
    }
    // Just to make the next `impl` block shorter, unused otherwise.
    use events::Direction as D;
    impl MenuNavigationStrategy for MockNavigationStrategy<'_, '_> {
        fn resolve_2d<'a>(&self, _: Entity, _: D, _: bool, _: &'a [Entity]) -> Option<&'a Entity> {
            None
        }
    }
    fn receive_events<E: Event + Clone>(world: &World) -> Vec<E> {
        let events = world.resource::<Events<E>>();
        events.iter_current_update_events().cloned().collect()
    }

    /// Wrapper around `App` to make it easier to test the navigation systems.
    struct NavEcsMock {
        app: App,
    }
    impl NavEcsMock {
        fn currently_focused(&mut self) -> &str {
            let mut query = self
                .app
                .world_mut()
                .query_filtered::<&Name, With<Focused>>();
            query.iter(self.app.world()).next().unwrap()
        }
        fn kill_named(&mut self, to_kill: &str) -> Vec<NavEvent> {
            let mut query = self.app.world_mut().query::<(Entity, &Name)>();
            let requested = query
                .iter(self.app.world())
                .find_map(|(e, name)| (&**name == to_kill).then(|| e));
            if let Some(to_kill) = requested {
                self.app.world_mut().despawn(to_kill);
            }
            self.app.update();
            receive_events(self.app.world_mut())
        }
        fn name_list(&mut self, entity_list: &[Entity]) -> Vec<&str> {
            let mut query = self.app.world_mut().query::<&Name>();
            entity_list
                .iter()
                .filter_map(|e| query.get(self.app.world(), *e).ok())
                .map(|name| &**name)
                .collect()
        }
        fn new(hierarchy: SpawnHierarchy) -> Self {
            let mut app = App::new();
            app.add_plugins(GenericNavigationPlugin::<MockNavigationStrategy>::new());
            hierarchy.spawn(app.world_mut());
            // Run once to convert the `MenuSetting` and `MenuBuilder` into `TreeMenu`.
            app.update();

            Self { app }
        }
        fn run_focus_on(&mut self, entity_name: &str) -> Vec<NavEvent> {
            let mut query = self.app.world_mut().query::<(Entity, &Name)>();
            let requested = query
                .iter(self.app.world())
                .find_map(|(e, name)| (&**name == entity_name).then(|| e))
                .unwrap();
            self.app
                .world_mut()
                .send_event(NavRequest::FocusOn(requested));
            self.app.update();
            receive_events(self.app.world_mut())
        }
        fn run_request(&mut self, request: NavRequest) -> Vec<NavEvent> {
            self.app.world_mut().send_event(request);
            self.app.update();
            receive_events(self.app.world_mut())
        }
        fn state_of(&mut self, requested: &str) -> FocusState {
            let mut query = self.app.world_mut().query::<(&Focusable, &Name)>();
            let requested = query
                .iter(self.app.world())
                .find_map(|(focus, name)| (&**name == requested).then(|| focus));
            requested.unwrap().state()
        }
    }

    // ====
    // Expected basic functionalities
    // ====

    #[test]
    fn move_in_menuless() {
        let mut app = NavEcsMock::new(spawn_hierarchy!(@rootless [
            prioritized("Initial"),
            focusable("Left"),
            focusable("Right"),
        ]));
        assert_eq!(app.currently_focused(), "Initial");
        app.run_focus_on("Left");
        assert_eq!(app.currently_focused(), "Left");
    }

    #[test]
    fn deep_initial_focusable() {
        let mut app = NavEcsMock::new(spawn_hierarchy![
            focusable("Middle"),
            focusable_to("Left" [
                focusable("LCenter1"),
                focusable("LCenter2"),
                focusable_to("LTop" [
                    prioritized("LTopForward"),
                    focusable("LTopBackward"),
                ]),
                focusable("LCenter3"),
                focusable("LBottom"),
            ]),
            focusable("Right"),
        ]);
        use FocusState::{Active, Inert};
        assert_eq!(app.currently_focused(), "LTopForward");
        assert_eq!(app.state_of("Left"), Active);
        assert_eq!(app.state_of("Right"), Inert);
        assert_eq!(app.state_of("Middle"), Inert);
        assert_eq!(app.state_of("LTop"), Active);
        assert_eq!(app.state_of("LCenter1"), Inert);
        assert_eq!(app.state_of("LTopBackward"), Inert);
    }

    #[test]
    fn move_in_complex_menu_hierarchy() {
        let mut app = NavEcsMock::new(spawn_hierarchy![
            prioritized("Initial"),
            focusable_to("Left" [
                focusable_to("LTop" [
                    focusable("LTopForward"),
                    focusable("LTopBackward"),
                ]),
                focusable_to("LBottom" [
                    focusable("LBottomForward"),
                    focusable("LBottomForward1"),
                    focusable("LBottomForward2"),
                    prioritized("LBottomBackward"),
                    focusable("LBottomForward3"),
                    focusable("LBottomForward4"),
                    focusable("LBottomForward5"),
                ]),
            ]),
            focusable_to("Right" [
                focusable_to("RTop" [
                    focusable("RTopForward"),
                    focusable("RTopBackward"),
                ]),
                focusable("RBottom"),
            ]),
        ]);
        assert_eq!(app.currently_focused(), "Initial");

        // Move deep into a menu
        let events = app.run_focus_on("RBottom");
        assert_expected_focus_change!(app, &events[..], ["Initial"], ["RBottom", "Right"]);

        // Go up and back down several layers of menus
        let events = app.run_focus_on("LTopForward");
        assert_expected_focus_change!(
            app,
            &events[..],
            ["RBottom", "Right"],
            ["LTopForward", "LTop", "Left"],
        );
        // See if cancel event works
        let events = app.run_request(NavRequest::Cancel);
        assert_expected_focus_change!(app, &events[..], ["LTopForward", "LTop"], ["LTop"]);

        // Move to sibling within menu
        let events = app.run_focus_on("LBottom");
        assert_expected_focus_change!(app, &events[..], ["LTop"], ["LBottom"]);

        // Move down into menu by activating a focusable
        // (also make sure `prioritized` works)
        let events = app.run_request(NavRequest::Action);
        assert_expected_focus_change!(
            app,
            &events[..],
            ["LBottom"],
            ["LBottomBackward", "LBottom"]
        );
    }

    // ====
    // What happens when Focused element is killed
    // ====

    // Select a new focusable in the same menu (or anything if no menus exist)
    #[test]
    fn focus_rootless_kill_robust() {
        let mut app = NavEcsMock::new(spawn_hierarchy!(@rootless [
            prioritized("Initial"),
            focusable("Right"),
        ]));
        assert_eq!(app.currently_focused(), "Initial");
        app.kill_named("Initial");
        assert_eq!(app.currently_focused(), "Right");

        app.kill_named("Right");
        let events = app.run_request(NavRequest::Action);
        assert_eq!(events.len(), 0, "{:#?}", events);
    }

    // Go up the menu tree if it was the last focusable in the menu
    // And swap to something in the same menu if focusable killed in it.
    #[test]
    fn menu_elem_kill_robust() {
        let mut app = NavEcsMock::new(spawn_hierarchy![
            focusable_to("Left" [
                focusable("LTop"),
                focusable("LBottom"),
            ]),
            focusable_to("Antony" [
                prioritized("Caesar"),
                focusable("Brutus"),
            ]),
            focusable_to("Octavian" [
                focusable("RTop"),
                focusable("RBottom"),
            ]),
        ]);
        // NOTE: was broken because didn't properly set
        // active_child and Active when initial focus was given to
        // a deep element.
        assert_eq!(app.currently_focused(), "Caesar");
        assert_eq!(app.state_of("Antony"), FocusState::Active);
        app.kill_named("Caesar");
        assert_eq!(app.currently_focused(), "Brutus");
        app.kill_named("Brutus");
        assert_eq!(app.currently_focused(), "Antony");
    }

    // ====
    // removal of parent menu and focusables
    // ====

    // Relink the child menu to the removed parent's parents
    // Make sure this works with root as well
    // Relink when the focusable parent of a menu is killed
    // NOTE: user is warned against engaging in such operations, implementation can wait

    // ====
    // some reparenting potential problems
    // ====

    // Focused element is reparented to a new menu
    // Active element is reparented to a new menu
    // NOTE: those are not expected to work. Currently considered a user error.
}
