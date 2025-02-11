use std::collections::HashMap;

use crate::ecs::component::{Component as C, ComponentArray as A};

pub struct SystemId(usize);

type fn_0 = fn((), ());

type fn_1<R0: C> = fn(&mut A<R0>, ());
type fn_2<O0: C> = fn((), &mut A<O0>);

type fn_3<R0: C, R1: C> = fn((&mut A<R0>, &mut A<R1>), ());
type fn_4<R0: C, O0: C> = fn(&mut A<R0>, &mut A<O0>);
type fn_5<O0: C, O1: C> = fn((), (&mut A<O0>, &mut A<O1>));

type FnTypeIndex = u16;

pub(in crate::ecs) struct SystemManager {
    // TODO
    id_counter: usize,
    systems_0: HashMap<usize, (fn_0, FnTypeIndex, FnTypeIndex)>,
    systems_1: HashMap<usize, (fn_1<dyn C>, FnTypeIndex, FnTypeIndex)>, // TODO: what types?
}

const INVALID_FUNCTION_TYPE_INDEX: FnTypeIndex = FnTypeIndex::MAX;

impl SystemManager {
    pub(in crate::ecs) fn register_system_0<R0>(system: fn(R0, ())) -> SystemId {
        // TODO
    }

    pub(in crate::ecs) fn register_system_1<O0>(system: fn((), O0)) -> SystemId {
        // TODO
    }
}
