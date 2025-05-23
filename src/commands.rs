use bevy::ecs::{
    entity::Entity,
    prelude::{Command, World},
};

use crate::resolve::{FocusState, Focused};

pub(crate) fn set_focus_state(entity: Entity, new_state: FocusState) -> UpdateFocusable {
    UpdateFocusable { entity, new_state }
}
pub(crate) struct UpdateFocusable {
    entity: Entity,
    new_state: FocusState,
}
impl Command for UpdateFocusable {
    fn apply(self, world: &mut World) {
        let mut entity = world.entity_mut(self.entity);
        if matches!(self.new_state, FocusState::Focused) {
            entity.insert(Focused);
        } else {
            entity.remove::<Focused>();
        }
    }
}
