use crate::AssetServer;
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

use atelier_loader::{
    handle::{AssetHandle, GenericHandle as AtelierHandleUntyped},
    AtomicHandleAllocator, HandleAllocator, LoadHandle,
};
use bevy_ecs::FromResources;
use bevy_property::{
    property_serde::Serializable, DeserializeProperty, Properties, Property, PropertyTypeRegistry,
};
use serde::{Deserialize, Serialize};
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};
use uuid::Uuid;

/// The ID of the "default" asset
pub(crate) const DEFAULT_HANDLE_ID: HandleId = HandleId(LoadHandle(1));

pub(crate) static HANDLE_ALLOCATOR: AtomicHandleAllocator = AtomicHandleAllocator::new(2);
/// A unique id that corresponds to a specific asset in the [Assets](crate::Assets) collection.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct HandleId(pub LoadHandle);

impl Default for HandleId {
    fn default() -> Self {
        DEFAULT_HANDLE_ID
    }
}

/// A handle into a specific Asset of type `T`
///
/// Handles contain a unique id that corresponds to a specific asset in the [Assets](crate::Assets) collection.
#[derive(Serialize, Deserialize)]
pub struct Handle<T>
where
    T: 'static,
{
    pub handle: AtelierHandleUntyped,
    pub marker: PhantomData<T>,
}

impl<T> Handle<T> {
    /// Gets a handle for the given type that has this handle's id. This is useful when an
    /// asset is derived from another asset. In this case, a common handle can be used to
    /// correlate them.
    /// NOTE: This pattern might eventually be replaced by a more formal asset dependency system.
    pub fn as_handle<U>(&self) -> Handle<U> {
        Handle {
            handle: self.handle.clone(),
            marker: PhantomData,
        }
    }
    pub fn id(&self) -> HandleId {
        HandleId(self.handle.load_handle())
    }
}

impl<T> From<HandleUntyped> for Handle<T>
where
    T: 'static,
{
    fn from(handle: HandleUntyped) -> Self {
        Handle {
            handle: handle.handle.into(),
            marker: PhantomData,
        }
    }
}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.handle.hash(state);
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let name = std::any::type_name::<T>().split("::").last().unwrap();
        write!(f, "Handle<{}>({:?})", name, self.handle.load_handle())
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle {
            handle: self.handle.clone(),
            marker: PhantomData,
        }
    }
}

// SAFE: T is phantom data and Handle::id is an integer
unsafe impl<T> Send for Handle<T> {}
unsafe impl<T> Sync for Handle<T> {}

impl<T> DeserializeProperty for Handle<T> {
    fn deserialize(
        deserializer: &mut dyn bevy_property::erased_serde::Deserializer,
        property_type_registry: &PropertyTypeRegistry,
    ) -> Result<Box<dyn Property>, bevy_property::erased_serde::Error> {
        use bevy_property::serde::de::DeserializeSeed;
        let dynamic_properties_deserializer =
            bevy_property::property_serde::DynamicPropertiesDeserializer::new(
                property_type_registry,
            );
        let dynamic_properties: bevy_property::DynamicProperties =
            dynamic_properties_deserializer.deserialize(deserializer)?;
        Ok(Box::new(dynamic_properties))
    }
}

impl<T> Property for Handle<T> {
    #[inline]
    fn type_name(&self) -> &str {
        "Handle"
    }
    #[inline]
    fn any(&self) -> &dyn Any {
        self
    }
    #[inline]
    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }
    #[inline]
    fn clone_prop(&self) -> Box<dyn Property> {
        Box::new(self.clone())
    }
    fn set(&mut self, value: &dyn Property) {
        let value = value.any();
        if let Some(prop) = value.downcast_ref::<Self>() {
            *self = prop.clone();
        } else {
            panic!("prop value is not {}", std::any::type_name::<Self>());
        }
    }
    fn apply(&mut self, value: &dyn Property) {
        self.set(value)
    }
    fn serializable<'a>(&'a self, registry: &'a PropertyTypeRegistry) -> Serializable<'a> {
        Serializable::Borrowed(self)
    }
}
impl<T> Properties for Handle<T> {
    fn prop(&self, name: &str) -> Option<&dyn Property> {
        None
    }
    fn prop_mut(&mut self, name: &str) -> Option<&mut dyn Property> {
        None
    }
    fn prop_with_index(&self, index: usize) -> Option<&dyn Property> {
        None
    }
    fn prop_with_index_mut(&mut self, index: usize) -> Option<&mut dyn Property> {
        None
    }
    fn prop_name(&self, index: usize) -> Option<&str> {
        None
    }
    fn prop_len(&self) -> usize {
        0
    }
    fn iter_props(&self) -> bevy_property::PropertyIter {
        bevy_property::PropertyIter::new(self)
    }
}

impl<T> FromResources for Handle<T> {
    fn from_resources(resources: &bevy_ecs::Resources) -> Self {
        let sender = resources
            .get::<AssetServer>()
            .expect("No AssetServer in resources")
            .ref_op_tx();
        Self {
            handle: AtelierHandleUntyped::new(sender, DEFAULT_HANDLE_ID.0),
            marker: PhantomData,
        }
    }
}

/// A non-generic version of [Handle]
///
/// This allows handles to be mingled in a cross asset context. For example, storing `Handle<A>` and `Handle<B>` in the same `HashSet<HandleUntyped>`.
#[derive(Hash, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct HandleUntyped {
    pub handle: AtelierHandleUntyped,
}

impl<T> From<Handle<T>> for HandleUntyped
where
    T: 'static,
{
    fn from(handle: Handle<T>) -> Self {
        HandleUntyped {
            handle: handle.handle.into(),
        }
    }
}

impl DeserializeProperty for HandleUntyped {
    fn deserialize(
        deserializer: &mut dyn bevy_property::erased_serde::Deserializer,
        property_type_registry: &PropertyTypeRegistry,
    ) -> Result<Box<dyn Property>, bevy_property::erased_serde::Error> {
        use bevy_property::serde::de::DeserializeSeed;
        let dynamic_properties_deserializer =
            bevy_property::property_serde::DynamicPropertiesDeserializer::new(
                property_type_registry,
            );
        let dynamic_properties: bevy_property::DynamicProperties =
            dynamic_properties_deserializer.deserialize(deserializer)?;
        Ok(Box::new(dynamic_properties))
    }
}

impl Property for HandleUntyped {
    #[inline]
    fn type_name(&self) -> &str {
        "HandleUntyped"
    }
    #[inline]
    fn any(&self) -> &dyn Any {
        self
    }
    #[inline]
    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }
    #[inline]
    fn clone_prop(&self) -> Box<dyn Property> {
        Box::new(self.clone())
    }
    fn set(&mut self, value: &dyn Property) {
        let value = value.any();
        if let Some(prop) = value.downcast_ref::<Self>() {
            *self = prop.clone();
        } else {
            panic!("prop value is not {}", std::any::type_name::<Self>());
        }
    }
    fn apply(&mut self, value: &dyn Property) {
        self.set(value)
    }
    fn serializable<'a>(&'a self, registry: &'a PropertyTypeRegistry) -> Serializable<'a> {
        Serializable::Borrowed(self)
    }
}
impl Properties for HandleUntyped {
    fn prop(&self, name: &str) -> Option<&dyn Property> {
        None
    }
    fn prop_mut(&mut self, name: &str) -> Option<&mut dyn Property> {
        None
    }
    fn prop_with_index(&self, index: usize) -> Option<&dyn Property> {
        None
    }
    fn prop_with_index_mut(&mut self, index: usize) -> Option<&mut dyn Property> {
        None
    }
    fn prop_name(&self, index: usize) -> Option<&str> {
        None
    }
    fn prop_len(&self) -> usize {
        0
    }
    fn iter_props(&self) -> bevy_property::PropertyIter {
        bevy_property::PropertyIter::new(self)
    }
}

impl FromResources for HandleUntyped {
    fn from_resources(resources: &bevy_ecs::Resources) -> Self {
        let sender = resources
            .get::<AssetServer>()
            .expect("No AssetServer in resources")
            .ref_op_tx();
        Self {
            handle: AtelierHandleUntyped::new(sender, DEFAULT_HANDLE_ID.0),
        }
    }
}
