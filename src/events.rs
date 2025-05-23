//! Navigation events and requests.
//!
//! The navigation system works through bevy's `Events` system.
//! It is a system with one input and two outputs:
//! * Input `EventWriter<NavRequest>`, tells the navigation system what to do.
//!   Your app should have a system that writes to a `EventWriter<NavRequest>`
//!   based on inputs or internal game state.
//!   Bevy provides default systems in `bevy_ui`.
//!   But you can add your own requests on top of the ones the default systems send.
//!   For example to unlock the UI with [`NavRequest::Unlock`].
//! * Output [`Focusable`] components.
//!   The navigation system updates the focusables component
//!   according to the focus state of the navigation system.
//!   See `examples/cursor_navigation` directory for usage clues.
//! * Output `EventReader<NavEvent>`,
//!   contains specific information about what the navigation system is doing.
//!
//! [`Focusable`]: crate::resolve::Focusable
use bevy::{
    ecs::{
        entity::Entity,
        event::EventReader,
        query::{QueryData, QueryFilter, ReadOnlyQueryData},
        system::Query,
    },
    math::Vec2,
    prelude::Event,
};
use non_empty_vec::NonEmpty;

use crate::resolve::LockReason;

/// Requests to send to the navigation system to update focus.
#[derive(Debug, PartialEq, Clone, Copy, Event)]
pub enum NavRequest {
    /// Move in in provided direction according to the plugin's [navigation strategy].
    ///
    /// Typically used by gamepads.
    ///
    /// [navigation strategy]: crate::resolve::MenuNavigationStrategy.
    Move(Direction),

    /// Move within the encompassing [`MenuSetting::scope`].
    ///
    /// [`MenuSetting::scope`]: crate::prelude::MenuSetting::scope
    ScopeMove(ScopeDirection),

    /// Activate the currently focused [`Focusable`].
    ///
    /// If a menu is _[reachable from]_
    ///
    /// [`Focusable`]: crate::prelude::Focusable
    /// [reachable from]: crate::menu::MenuBuilder::NamedParent
    Action,

    /// Leave this submenu to enter the one it is _[reachable from]_.
    ///
    /// [reachable from]: crate::menu::MenuBuilder::NamedParent
    Cancel,

    /// Move the focus to any arbitrary [`Focusable`] entity.
    ///
    /// Note that resolving a `FocusOn` request is expensive,
    /// make sure you do not spam `FocusOn` messages in your input systems.
    /// Avoid sending FocusOn messages when you know the target entity is
    /// already focused.
    ///
    /// [`Focusable`]: crate::resolve::Focusable
    FocusOn(Entity),

    /// Locks the navigation system.
    ///
    /// A [`NavEvent::Locked`] will be emitted as a response if the
    /// navigation system was not already locked.
    Lock,

    /// Unlocks the navigation system.
    ///
    /// A [`NavEvent::Unlocked`] will be emitted as a response if the
    /// navigation system was indeed locked.
    Unlock,
}

/// Direction for movement in [`MenuSetting::scope`] menus.
///
/// [`MenuSetting::scope`]: crate::menu::MenuSetting
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ScopeDirection {
    /// The next focusable in menu, usually goes right.
    Next,

    /// The previous focusable in menu, usually goes left.
    Previous,
}

/// 2d direction to move in normal menus
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Direction {
    /// Down.
    South,
    /// Up.
    North,
    /// Right.
    East,
    /// Left.
    West,
}
impl Direction {
    /// Is `other` in direction `self` from `reference`?
    pub fn is_in(&self, reference: Vec2, other: Vec2) -> bool {
        let coord = other - reference;
        use Direction::*;
        match self {
            North => coord.y < coord.x && coord.y < -coord.x,
            South => coord.y > coord.x && coord.y > -coord.x,
            East => coord.y < coord.x && coord.y > -coord.x,
            West => coord.y > coord.x && coord.y < -coord.x,
        }
    }
}

/// Events emitted by the navigation system.
///
/// Useful if you want to react to [`NavEvent::NoChanges`] event, for example
/// when a "start game" button is focused and the [`NavRequest::Action`] is
/// pressed.
#[derive(Debug, Clone, Event)]
pub enum NavEvent {
    /// Tells the app which element is the first one to be focused.
    ///
    /// This will be sent whenever the number of focused elements go from 0 to 1.
    /// Meaning: whenever you spawn a new UI with [`Focusable`] elements.
    ///
    /// The order of selection when no [`Focusable`] is focused yet is as follow:
    /// - The prioritized `Focusable` of the root menu
    /// - Any prioritized `Focusable`
    /// - Any `Focusable` in the root menu
    /// - Any `Focusable`
    ///
    /// [`Focusable`]: crate::resolve::Focusable
    InitiallyFocused(Entity),

    /// Focus changed.
    ///
    /// ## Notes
    ///
    /// Both `to` and `from` are ascending, meaning that the focused and newly
    /// focused elements are the first of their respective vectors.
    ///
    /// [`NonEmpty`] enables you to safely check `to.first()` or `from.first()`
    /// without returning an option. It is guaranteed that there is at least
    /// one element.
    FocusChanged {
        /// The list of elements that has become active after the focus
        /// change
        to: NonEmpty<Entity>,
        /// The list of active elements from the focused one to the last
        /// active which is affected by the focus change
        from: NonEmpty<Entity>,
    },

    /// The [`NavRequest`] didn't lead to any change in focus.
    NoChanges {
        /// The active elements from the focused one to the last
        /// active which is affected by the focus change.
        from: NonEmpty<Entity>,
        /// The [`NavRequest`] that didn't do anything.
        request: NavRequest,
    },

    /// The navigation [lock] has been enabled.
    /// Either by a [lock focusable] or [`NavRequest::Lock`].
    ///
    /// Once the navigation plugin enters a locked state, the only way to exit
    /// it is to send a [`NavRequest::Unlock`].
    ///
    /// [lock]: crate::resolve::NavLock
    /// [lock focusable]: crate::resolve::Focusable::lock
    Locked(LockReason),

    /// The navigation [lock] has been released.
    ///
    /// The navigation system was in a locked state triggered [`Entity`],
    /// is now unlocked, and receiving events again.
    ///
    /// [lock]: crate::resolve::NavLock
    Unlocked(LockReason),
}
impl NavEvent {
    /// Create a `FocusChanged` with a single `to`
    ///
    /// Usually the `NavEvent::FocusChanged.to` field has a unique value.
    pub(crate) fn focus_changed(to: Entity, from: NonEmpty<Entity>) -> NavEvent {
        NavEvent::FocusChanged {
            from,
            to: NonEmpty::new(to),
        }
    }

    /// Whether this event is a [`NavEvent::NoChanges`]
    /// triggered by a [`NavRequest::Action`]
    /// if `entity` is the currently focused element.
    pub fn is_activated(&self, entity: Entity) -> bool {
        matches!(self, NavEvent::NoChanges { from,  request: NavRequest::Action } if *from.first() == entity)
    }
}

/// Extend [`EventReader<NavEvent>`] with methods
/// to simplify working with [`NavEvent`]s.
///
/// See the [`NavEventReader`] documentation for details.
///
/// [`EventReader<NavEvent>`]: EventReader
pub trait NavEventReaderExt<'w, 's> {
    /// Create a [`NavEventReader`] from this event reader.
    fn nav_iter(&mut self) -> NavEventReader<'w, 's, '_>;
}
impl<'w, 's> NavEventReaderExt<'w, 's> for EventReader<'w, 's, NavEvent> {
    fn nav_iter(&mut self) -> NavEventReader<'w, 's, '_> {
        NavEventReader { event_reader: self }
    }
}

/// A wrapper for `EventReader<NavEvent>` to simplify dealing with [`NavEvent`]s.
pub struct NavEventReader<'w, 's, 'a> {
    event_reader: &'a mut EventReader<'w, 's, NavEvent>,
}

impl NavEventReader<'_, '_, '_> {
    /// Iterate over [`NavEvent::NoChanges`] focused entity
    /// triggered by `request` type requests.
    pub fn with_request(&mut self, request: NavRequest) -> impl Iterator<Item = Entity> + '_ {
        self.event_reader
            .read()
            .filter_map(move |nav_event| match nav_event {
                NavEvent::NoChanges {
                    from,
                    request: event_request,
                } if *event_request == request => Some(*from.first()),
                _ => None,
            })
    }
    /// Iterate over _activated_ [`Focusable`]s.
    ///
    /// A [`Focusable`] is _activated_ when a [`NavRequest::Action`] is sent
    /// while it is focused, and it doesn't lead to a new menu.
    ///
    /// [`Focusable`]: crate::resolve::Focusable
    pub fn activated(&mut self) -> impl Iterator<Item = Entity> + '_ {
        self.with_request(NavRequest::Action)
    }

    /// Iterate over [`NavEvent`]s, associating them
    /// with the "relevant" entity of the event.
    pub fn types(&mut self) -> impl Iterator<Item = (&NavEvent, Entity)> + '_ {
        use NavEvent::{FocusChanged, InitiallyFocused, Locked, NoChanges, Unlocked};
        self.event_reader.read().filter_map(|event| {
            let entity = match event {
                NoChanges { from, .. } => Some(*from.first()),
                InitiallyFocused(initial) => Some(*initial),
                FocusChanged { from, .. } => Some(*from.first()),
                Locked(LockReason::Focusable(from)) => Some(*from),
                Unlocked(LockReason::Focusable(from)) => Some(*from),
                _ => None,
            };
            entity.map(|e| (event, e))
        })
    }

    /// Iterate over query items of _activated_ focusables.
    ///
    /// See [`Self::activated`] for meaning of _"activated"_.
    pub fn activated_in_query<'b, 'c: 'b, Q: ReadOnlyQueryData, F: QueryFilter>(
        &'b mut self,
        query: &'c Query<Q, F>,
    ) -> impl Iterator<Item = Q::Item<'c>> + 'b {
        query.iter_many(self.activated())
    }

    /// Run `for_each` with result of `query` for each _activated_ entity.
    ///
    /// Unlike [`Self::activated_in_query`] this works with mutable queries.
    /// see [`Self::activated`] for meaning of _"activated"_.
    pub fn activated_in_query_foreach_mut<Q: QueryData, F: QueryFilter>(
        &mut self,
        query: &mut Query<Q, F>,
        mut for_each: impl FnMut(Q::Item<'_>),
    ) {
        let mut iter = query.iter_many_mut(self.activated());
        while let Some(item) = iter.fetch_next() {
            for_each(item)
        }
    }
}
