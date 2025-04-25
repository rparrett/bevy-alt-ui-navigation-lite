//! Automatic menu marker component propagation.
//!
//! Enables user to specify their own marker to add to [`Focusable`]s within
//! [menus](crate::menu::MenuSetting).
use std::iter;

use bevy::prelude::*;

use crate::{
    menu::NavMarker,
    resolve::{self, Focusable, TreeMenu},
};

pub(crate) fn mark_new_menus<T: Component + Clone>(
    mut cmds: Commands,
    new_markers: Query<(Entity, &NavMarker<T>), Added<TreeMenu>>,
    children: resolve::ChildQueries,
) {
    let mut to_insert = Vec::new();
    for (new_menu, marker) in new_markers.iter() {
        let repeat_marker = iter::repeat((marker.0.clone(),));
        let menu_children = children.focusables_of(new_menu);
        to_insert.extend(menu_children.into_iter().zip(repeat_marker));
    }
    cmds.insert_batch(to_insert);
}
pub(crate) fn mark_new_focusables<T: Component + Clone>(
    mut cmds: Commands,
    new_focusables: Query<Entity, Added<Focusable>>,
    markers: Query<&NavMarker<T>, With<TreeMenu>>,
    queries: resolve::NavQueries,
) {
    let mut to_insert = Vec::new();
    for new_focusable in new_focusables.iter() {
        let containing_menu = match resolve::parent_menu(new_focusable, &queries) {
            Some((c, _, _)) => c,
            None => continue,
        };
        let marker = match markers.get(containing_menu) {
            Ok(m) => m.0.clone(),
            Err(_) => continue,
        };
        to_insert.push((new_focusable, (marker,)));
    }
    cmds.insert_batch(to_insert);
}
