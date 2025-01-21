use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::data::{ObjectId, TileId};

#[derive(Clone, Copy, Encode, Decode, Reflect, Default, Debug)]
pub struct TilePlaced {
    pub id: TileId,
    pub object: Option<ObjectId>,
}

impl TilePlaced {
    pub fn is_blocking(self) -> bool {
        self.id.data().is_wall()
            || self
                .object
                .is_some_and(|o| ObjectId::data(&o).is_blocking())
    }

    pub fn is_floor_free(self) -> bool {
        !self.is_blocking() && self.object.is_none()
    }
}

pub struct TileData {
    filename: &'static str,
    wall: bool,
}

impl TileData {
    const fn new(filename: &'static str, wall: bool) -> Self {
        Self { filename, wall }
    }

    pub const fn floor(filename: &'static str) -> Self {
        Self::new(filename, false)
    }

    pub const fn wall(filename: &'static str) -> Self {
        Self::new(filename, true)
    }

    #[inline]
    pub fn is_wall(&self) -> bool {
        self.wall
    }

    #[inline]
    pub fn filename(&self) -> &'static str {
        self.filename
    }
}

impl TileId {
    pub fn with(self, object_id: ObjectId) -> TilePlaced {
        TilePlaced {
            id: self,
            object: Some(object_id),
        }
    }

    pub fn place(self) -> TilePlaced {
        TilePlaced {
            id: self,
            object: None,
        }
    }

    pub fn s(self) -> Option<TilePlaced> {
        Some(self.place())
    }

    pub fn i(self, object_id: ObjectId) -> Option<TilePlaced> {
        Some(self.with(object_id))
    }

    pub fn is_transparent(self) -> bool {
        !self.data().is_wall() || self == Self::Water
    }
}

pub struct ObjectData {
    filename: &'static str,
    blocking: bool,
    carriable: bool,
}

impl ObjectData {
    const fn new(filename: &'static str, blocking: bool, carriable: bool) -> Self {
        Self {
            filename,
            blocking,
            carriable,
        }
    }

    pub const fn passable(filename: &'static str) -> Self {
        Self::new(filename, false, true)
    }

    pub const fn blocking(filename: &'static str) -> Self {
        Self::new(filename, true, true)
    }

    pub const fn passable_non_carriable(filename: &'static str) -> Self {
        Self::new(filename, false, false)
    }

    pub const fn blocking_non_carriable(filename: &'static str) -> Self {
        Self::new(filename, true, false)
    }

    #[inline]
    pub fn is_carriable(&self) -> bool {
        self.carriable
    }

    #[inline]
    pub fn is_blocking(&self) -> bool {
        self.blocking
    }

    #[inline]
    pub fn filename(&self) -> &'static str {
        self.filename
    }
}
