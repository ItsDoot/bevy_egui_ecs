use bevy::{ecs::system::BoxedSystem, prelude::*};

pub struct CallbackHolder<In = (), Out = ()>(Option<Callback<In, Out>>);

impl<In: 'static, Out: 'static> CallbackHolder<In, Out> {
    pub fn new<M>(system: impl IntoSystem<In, Out, M>) -> Self {
        Self(Some(Callback::new(system)))
    }

    pub fn take(&mut self) -> Option<Callback<In, Out>> {
        self.0.take()
    }

    pub fn require(&mut self) -> Callback<In, Out> {
        self.take().unwrap_or_else(|| {
            panic!(
                "{} was empty. Was this recursively called?",
                std::any::type_name::<Self>()
            )
        })
    }

    pub fn insert(&mut self, callback: Callback<In, Out>) {
        if let Some(_) = self.0 {
            panic!("{} was already filled", std::any::type_name::<Self>());
        } else {
            self.0 = Some(callback);
        }
    }
}

pub struct Callback<In = (), Out = ()> {
    initialized: bool,
    system: BoxedSystem<In, Out>,
}

impl<In: 'static, Out: 'static> Callback<In, Out> {
    pub fn new<M>(system: impl IntoSystem<In, Out, M>) -> Self {
        Self {
            initialized: false,
            system: Box::new(IntoSystem::into_system(system)),
        }
    }

    pub fn run(&mut self, world: &mut World, input: In) -> Out {
        if !self.initialized {
            self.system.initialize(world);
            self.initialized = true;
        }

        let output = self.system.run(input, world);
        self.system.apply_deferred(world);

        output
    }
}

pub struct ROCallbackHolder<In = (), Out = ()>(Option<ROCallback<In, Out>>);

impl<In: 'static, Out: 'static> ROCallbackHolder<In, Out> {
    pub fn new<S, M>(system: S) -> Self
    where
        S: IntoSystem<In, Out, M>,
        S::System: ReadOnlySystem<In = In, Out = Out>,
    {
        Self(Some(ROCallback::new(system)))
    }

    pub fn take(&mut self) -> Option<ROCallback<In, Out>> {
        self.0.take()
    }

    pub fn require(&mut self) -> ROCallback<In, Out> {
        self.take().unwrap_or_else(|| {
            panic!(
                "{} was empty. Was this recursively called?",
                std::any::type_name::<Self>()
            )
        })
    }

    pub fn insert(&mut self, callback: ROCallback<In, Out>) {
        if let Some(_) = self.0 {
            panic!("{} was already filled", std::any::type_name::<Self>());
        } else {
            self.0 = Some(callback);
        }
    }
}

pub type BoxedROSystem<In, Out> = Box<dyn ReadOnlySystem<In = In, Out = Out>>;

/// A version of [`Callback`] for read-only systems that enables them to run in parallel.
///
/// Requires additional bookkeeping!
pub struct ROCallback<In = (), Out = ()> {
    initialized: bool,
    system: BoxedROSystem<In, Out>,
}

impl<In: 'static, Out: 'static> ROCallback<In, Out> {
    pub fn new<S, M>(system: S) -> Self
    where
        S: IntoSystem<In, Out, M>,
        S::System: ReadOnlySystem<In = In, Out = Out>,
    {
        Self {
            initialized: false,
            system: Box::new(IntoSystem::into_system(system)),
        }
    }

    pub fn run(&mut self, world: &mut World, input: In) -> Out {
        if !self.initialized {
            self.system.initialize(world);
            self.initialized = true;
        }

        let output = self.system.run(input, world);
        self.system.apply_deferred(world);

        output
    }

    /// MUST initialize systems before they are ran, and apply_deferred afterwards.
    pub fn run_readonly(&mut self, world: &World, input: In) -> Out {
        assert!(self.initialized, "Callback not initialized");
        self.system.run_readonly(input, world)
    }

    pub fn initialize(&mut self, world: &mut World) {
        if !self.initialized {
            self.system.initialize(world);
            self.initialized = true;
        }
    }

    pub fn apply_deferred(&mut self, world: &mut World) {
        assert!(self.initialized, "Callback not initialized");
        self.system.apply_deferred(world);
    }
}
