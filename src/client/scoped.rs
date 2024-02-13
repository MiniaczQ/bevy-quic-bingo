use bevy::prelude::*;

#[derive(Component)]
pub struct Scoped<S: States>(pub S);

pub fn despawn_scoped<S: States>(
    mut commands: Commands,
    state: Res<State<S>>,
    query: Query<(Entity, &Scoped<S>)>,
) {
    if !state.is_changed() {
        return;
    }
    let state = state.get();
    for (entity, on_state) in query.iter() {
        if &on_state.0 == state {
            continue;
        }
        commands.entity(entity).despawn_recursive();
    }
}

pub trait ScopedExt {
    fn entity_scope<S: States>(&mut self) -> &mut Self;
}

impl ScopedExt for App {
    fn entity_scope<S: States>(&mut self) -> &mut Self {
        self.add_systems(
            StateTransition,
            despawn_scoped::<S>.after(apply_state_transition::<S>),
        );
        self
    }
}
