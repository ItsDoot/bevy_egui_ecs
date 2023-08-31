#![cfg_attr(feature = "nightly", feature(auto_traits))]
#![cfg_attr(feature = "nightly", feature(negative_impls))]

use std::hash::{Hash, Hasher};

use bevy::{
    prelude::*,
    utils::{AHasher, HashMap},
    window::PrimaryWindow,
};
use bevy_egui::{
    egui::{self, Ui},
    EguiContext,
};

use crate::util::Prepend;

pub mod callback;
pub mod util;

pub trait WorldEcsExt {
    fn run_system_id<S, In, Out, M>(&mut self, system: S, input: In, id: impl Hash) -> Out
    where
        S: IntoSystem<In, Out, M>;

    fn run_root_widget_system<S, In, Out, M>(&mut self, system: S, input: In, id: impl Hash) -> Out
    where
        In: Prepend,
        S: IntoSystem<<In as Prepend>::Out<egui::Context>, Out, M>;

    fn run_widget_system<S, In, M>(&mut self, ui: &mut Ui, system: S, input: In, id: impl Hash)
    where
        In: Prepend,
        S: IntoSystem<<In as Prepend>::Out<egui::Ui>, egui::Ui, M>;

    fn primary_egui_context(&mut self) -> egui::Context;
}

impl WorldEcsExt for World {
    fn run_system_id<S, In, Out, M>(&mut self, system: S, input: In, id: impl Hash) -> Out
    where
        S: IntoSystem<In, Out, M>,
    {
        let id = SystemId::new(id);

        // fetch the system cache
        let mut cache =
            self.get_resource_or_insert_with(|| CachedSystemsById::<S::System>::default());

        // grab our system, initializing a new one if it doesn't exist
        let mut system = cache
            .instances
            .get_mut(&id)
            .map(|holder| {
                // take out the system from the cache, panicking if it isn't stored
                holder.take().unwrap_or_else(|| {
                    panic!(
                        "Holder for system with id {:?} was empty. Was it recursively called?",
                        id
                    )
                })
            })
            .unwrap_or_else(|| {
                // the system isn't currently cached
                // so take the passed in one, initialize it, and use it instead
                let mut new = IntoSystem::into_system(system);
                new.initialize(self);
                new
            });

        // run our system
        let output = system.run(input, self);

        // apply deferred changes (like Commands) afterwards
        system.apply_deferred(self);

        // reaccess the system cache
        let mut cache =
            self.get_resource_or_insert_with(|| CachedSystemsById::<S::System>::default());

        // insert our system back into it
        let _ = cache.instances.entry(id).or_default().insert(system);

        output
    }

    fn run_root_widget_system<S, In, Out, M>(&mut self, system: S, input: In, id: impl Hash) -> Out
    where
        In: Prepend,
        S: IntoSystem<<In as Prepend>::Out<egui::Context>, Out, M>,
    {
        let ctx = self.primary_egui_context();

        self.run_system_id(system, input.prepend(ctx), id)
    }

    fn run_widget_system<S, In, M>(
        &mut self,
        ui: &mut egui::Ui,
        system: S,
        input: In,
        id: impl Hash,
    ) where
        In: Prepend,
        S: IntoSystem<<In as Prepend>::Out<egui::Ui>, egui::Ui, M>,
    {
        let child_ui = ui.child_ui(ui.available_rect_before_wrap(), *ui.layout());
        let child_ui = self.run_system_id(system, input.prepend(child_ui), id);
        ui.allocate_space(child_ui.min_size());
    }

    fn primary_egui_context(&mut self) -> egui::Context {
        let mut cache =
            self.query_filtered::<&mut EguiContext, (With<EguiContext>, With<PrimaryWindow>)>();
        let mut ctx = cache.single_mut(self);

        ctx.get_mut().clone()
    }
}

#[derive(Resource)]
struct CachedSystemsById<S: System> {
    instances: HashMap<SystemId, Option<S>>,
}

impl<S: System> Default for CachedSystemsById<S> {
    fn default() -> Self {
        Self {
            instances: Default::default(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct SystemId(u64);

impl SystemId {
    pub fn new(id: impl Hash) -> Self {
        let mut hasher = AHasher::default();
        id.hash(&mut hasher);
        SystemId(hasher.finish())
    }
}
