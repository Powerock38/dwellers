pub struct Object {
    filename: &'static str,
    blocking: bool,
    slot: ObjectSlot,
}

pub enum ObjectSlot {
    Uncarriable,
    Object,
    Tool,
    Armor(u32),
}

impl Object {
    const fn new(filename: &'static str, blocking: bool, slot: ObjectSlot) -> Self {
        Self {
            filename,
            blocking,
            slot,
        }
    }

    pub const fn passable(filename: &'static str) -> Self {
        Self::new(filename, false, ObjectSlot::Object)
    }

    pub const fn blocking(filename: &'static str) -> Self {
        Self::new(filename, true, ObjectSlot::Object)
    }

    pub const fn passable_non_carriable(filename: &'static str) -> Self {
        Self::new(filename, false, ObjectSlot::Uncarriable)
    }

    pub const fn blocking_non_carriable(filename: &'static str) -> Self {
        Self::new(filename, true, ObjectSlot::Uncarriable)
    }

    pub const fn tool(filename: &'static str) -> Self {
        Self::new(filename, false, ObjectSlot::Tool)
    }

    pub const fn armor(filename: &'static str, hp: u32) -> Self {
        Self::new(filename, false, ObjectSlot::Armor(hp))
    }

    pub fn is_carriable(&self) -> bool {
        !matches!(self.slot, ObjectSlot::Uncarriable)
    }

    pub fn is_blocking(&self) -> bool {
        self.blocking
    }

    pub fn slot(&self) -> &ObjectSlot {
        &self.slot
    }

    pub fn sprite_path(&self) -> String {
        format!("tiles/objects/{}.png", self.filename)
    }
}
