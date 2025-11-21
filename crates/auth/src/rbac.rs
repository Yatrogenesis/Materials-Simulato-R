//! Role-Based Access Control

use crate::Role;

pub fn can_create_material(role: Role) -> bool {
    matches!(role, Role::Admin | Role::Researcher)
}

pub fn can_delete_material(role: Role) -> bool {
    matches!(role, Role::Admin)
}

pub fn can_read_material(role: Role) -> bool {
    matches!(role, Role::Admin | Role::Researcher | Role::ReadOnly)
}
