use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;

use crate::entity::{Entity, GetError, Getter, Identifiable, Saver};

#[derive(Debug)]
struct Backend<T>
where
    T: Entity,
{
    entities: HashMap<T::Id, T>,
}

impl<T> Default for Backend<T>
where
    T: Entity,
{
    fn default() -> Self {
        Self {
            entities: HashMap::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InMemory<T>
where
    T: Entity,
    <T as Identifiable>::Id: Debug,
{
    backend: Arc<RwLock<Backend<T>>>,
}

impl<T> Default for InMemory<T>
where
    T: Entity,
    <T as Identifiable>::Id: Debug,
{
    fn default() -> Self {
        Self {
            backend: Arc::default(),
        }
    }
}

#[async_trait]
impl<T> Getter<T> for InMemory<T>
where
    T: Entity + Clone,
    <T as Identifiable>::Id: Debug + Hash,
{
    type Error = std::convert::Infallible;

    async fn get(&self, id: &T::Id) -> Result<T, GetError<Self::Error>> {
        let backend = self.backend.read().expect("acquire read lock on backend");

        backend
            .entities
            .get(id)
            .cloned()
            .ok_or_else(|| GetError::EntityNotFound)
    }
}

#[async_trait]
impl<T> Saver<T> for InMemory<T>
where
    T: Entity + Clone,
    <T as Identifiable>::Id: Debug + Hash,
{
    type Error = std::convert::Infallible;

    async fn save(&self, entity: &mut T) -> Result<(), Self::Error> {
        todo!()
    }
}
