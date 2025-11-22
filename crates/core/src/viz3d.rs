//! 3D Visualization Engine for Materials
//!
//! Backend-agnostic 3D visualization system for atomic structures, crystal lattices,
//! and material properties. Generates scenes that can be rendered via various frontends.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 3D Point in space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn distance(&self, other: &Point3D) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)).sqrt()
    }

    pub fn midpoint(&self, other: &Point3D) -> Point3D {
        Point3D {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
            z: (self.z + other.z) / 2.0,
        }
    }
}

/// RGB Color
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    // Element colors (CPK coloring)
    pub fn from_element(element: &str) -> Self {
        match element {
            "H" => Color::new(1.0, 1.0, 1.0),       // White
            "C" => Color::new(0.2, 0.2, 0.2),       // Dark gray
            "N" => Color::new(0.2, 0.2, 1.0),       // Blue
            "O" => Color::new(1.0, 0.0, 0.0),       // Red
            "F" => Color::new(0.5, 1.0, 0.5),       // Light green
            "P" => Color::new(1.0, 0.5, 0.0),       // Orange
            "S" => Color::new(1.0, 1.0, 0.0),       // Yellow
            "Cl" => Color::new(0.0, 1.0, 0.0),      // Green
            "Fe" => Color::new(1.0, 0.5, 0.0),      // Orange-brown
            "Co" => Color::new(0.5, 0.5, 1.0),      // Light blue
            "Ni" => Color::new(0.3, 0.8, 0.3),      // Green
            "Cu" => Color::new(0.8, 0.5, 0.2),      // Copper
            "Zn" => Color::new(0.5, 0.5, 0.5),      // Gray
            "Ag" => Color::new(0.8, 0.8, 0.8),      // Silver
            "Au" => Color::new(1.0, 0.8, 0.0),      // Gold
            "Ti" => Color::new(0.6, 0.6, 0.7),      // Gray-blue
            "Al" => Color::new(0.7, 0.7, 0.9),      // Light gray-blue
            "Li" => Color::new(0.8, 0.2, 1.0),      // Purple
            "Na" => Color::new(0.0, 0.0, 1.0),      // Blue
            "K" => Color::new(0.8, 0.4, 1.0),       // Violet
            "Ca" => Color::new(0.2, 1.0, 0.2),      // Light green
            "Mg" => Color::new(0.5, 1.0, 0.5),      // Light green
            "Si" => Color::new(0.8, 0.6, 0.4),      // Tan
            "Mo" => Color::new(0.3, 0.7, 0.7),      // Cyan-gray
            "B" => Color::new(1.0, 0.7, 0.7),       // Salmon
            _ => Color::new(0.8, 0.0, 0.8),         // Magenta (unknown)
        }
    }
}

/// Atom visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisAtom {
    pub id: usize,
    pub element: String,
    pub position: Point3D,
    pub color: Color,
    pub radius: f32,
    pub label: Option<String>,
}

impl VisAtom {
    pub fn new(id: usize, element: &str, position: Point3D) -> Self {
        let color = Color::from_element(element);
        let radius = get_atom_radius(element);

        Self {
            id,
            element: element.to_string(),
            position,
            color,
            radius,
            label: None,
        }
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }
}

/// Bond visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisBond {
    pub from_atom: usize,
    pub to_atom: usize,
    pub bond_type: BondVisualizationType,
    pub color: Color,
    pub thickness: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BondVisualizationType {
    Single,
    Double,
    Triple,
    Aromatic,
    Dashed,   // For weak bonds
}

/// Unit cell visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitCell {
    pub a: Point3D,
    pub b: Point3D,
    pub c: Point3D,
    pub alpha: f64,
    pub beta: f64,
    pub gamma: f64,
}

impl UnitCell {
    pub fn cubic(a: f64) -> Self {
        Self {
            a: Point3D::new(a, 0.0, 0.0),
            b: Point3D::new(0.0, a, 0.0),
            c: Point3D::new(0.0, 0.0, a),
            alpha: 90.0,
            beta: 90.0,
            gamma: 90.0,
        }
    }

    pub fn orthorhombic(a: f64, b: f64, c: f64) -> Self {
        Self {
            a: Point3D::new(a, 0.0, 0.0),
            b: Point3D::new(0.0, b, 0.0),
            c: Point3D::new(0.0, 0.0, c),
            alpha: 90.0,
            beta: 90.0,
            gamma: 90.0,
        }
    }

    pub fn tetragonal(a: f64, c: f64) -> Self {
        Self::orthorhombic(a, a, c)
    }

    pub fn hexagonal(a: f64, c: f64) -> Self {
        Self {
            a: Point3D::new(a, 0.0, 0.0),
            b: Point3D::new(-a/2.0, a * 3.0_f64.sqrt() / 2.0, 0.0),
            c: Point3D::new(0.0, 0.0, c),
            alpha: 90.0,
            beta: 90.0,
            gamma: 120.0,
        }
    }
}

/// Complete 3D Scene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene3D {
    pub id: Uuid,
    pub name: String,
    pub atoms: Vec<VisAtom>,
    pub bonds: Vec<VisBond>,
    pub unit_cell: Option<UnitCell>,
    pub camera: Camera,
    pub lighting: Lighting,
    pub annotations: Vec<Annotation>,
}

/// Camera configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub position: Point3D,
    pub target: Point3D,
    pub up: Point3D,
    pub fov: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point3D::new(10.0, 10.0, 10.0),
            target: Point3D::new(0.0, 0.0, 0.0),
            up: Point3D::new(0.0, 1.0, 0.0),
            fov: 45.0,
        }
    }
}

/// Lighting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lighting {
    pub ambient: f32,
    pub directional: Vec<DirectionalLight>,
    pub point: Vec<PointLight>,
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            ambient: 0.3,
            directional: vec![
                DirectionalLight {
                    direction: Point3D::new(1.0, -1.0, -1.0),
                    intensity: 0.7,
                    color: Color::new(1.0, 1.0, 1.0),
                }
            ],
            point: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub direction: Point3D,
    pub intensity: f32,
    pub color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointLight {
    pub position: Point3D,
    pub intensity: f32,
    pub color: Color,
}

/// Annotation/Label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub position: Point3D,
    pub text: String,
    pub color: Color,
    pub size: f32,
}

/// Visualization Engine
pub struct Viz3DEngine {
    scenes: HashMap<Uuid, Scene3D>,
}

impl Viz3DEngine {
    pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
        }
    }

    /// Create scene from atoms and bonds
    pub fn create_scene(
        &mut self,
        name: &str,
        atoms: Vec<VisAtom>,
        bonds: Vec<VisBond>,
    ) -> Uuid {
        let id = Uuid::new_v4();

        // Auto-center camera on atoms
        let center = self.calculate_center(&atoms);
        let camera = Camera {
            position: Point3D::new(center.x + 15.0, center.y + 15.0, center.z + 15.0),
            target: center,
            up: Point3D::new(0.0, 1.0, 0.0),
            fov: 45.0,
        };

        let scene = Scene3D {
            id,
            name: name.to_string(),
            atoms,
            bonds,
            unit_cell: None,
            camera,
            lighting: Lighting::default(),
            annotations: Vec::new(),
        };

        self.scenes.insert(id, scene);
        id
    }

    /// Create scene from crystal structure
    pub fn create_crystal_scene(
        &mut self,
        name: &str,
        atoms: Vec<VisAtom>,
        bonds: Vec<VisBond>,
        unit_cell: UnitCell,
        replicate: (usize, usize, usize),
    ) -> Uuid {
        let mut all_atoms = Vec::new();
        let mut all_bonds = Vec::new();
        let mut atom_id_offset = 0;

        // Replicate unit cell
        for i in 0..replicate.0 {
            for j in 0..replicate.1 {
                for k in 0..replicate.2 {
                    let offset = Point3D::new(
                        i as f64 * unit_cell.a.x + j as f64 * unit_cell.b.x + k as f64 * unit_cell.c.x,
                        i as f64 * unit_cell.a.y + j as f64 * unit_cell.b.y + k as f64 * unit_cell.c.y,
                        i as f64 * unit_cell.a.z + j as f64 * unit_cell.b.z + k as f64 * unit_cell.c.z,
                    );

                    // Clone atoms with offset
                    for atom in &atoms {
                        let mut new_atom = atom.clone();
                        new_atom.id += atom_id_offset;
                        new_atom.position = Point3D::new(
                            atom.position.x + offset.x,
                            atom.position.y + offset.y,
                            atom.position.z + offset.z,
                        );
                        all_atoms.push(new_atom);
                    }

                    // Clone bonds with offset
                    for bond in &bonds {
                        let new_bond = VisBond {
                            from_atom: bond.from_atom + atom_id_offset,
                            to_atom: bond.to_atom + atom_id_offset,
                            bond_type: bond.bond_type,
                            color: bond.color,
                            thickness: bond.thickness,
                        };
                        all_bonds.push(new_bond);
                    }

                    atom_id_offset += atoms.len();
                }
            }
        }

        self.create_scene(name, all_atoms, all_bonds)
    }

    /// Add unit cell box to scene
    pub fn add_unit_cell(&mut self, scene_id: Uuid, unit_cell: UnitCell) {
        if let Some(scene) = self.scenes.get_mut(&scene_id) {
            scene.unit_cell = Some(unit_cell);
        }
    }

    /// Add annotation
    pub fn add_annotation(&mut self, scene_id: Uuid, annotation: Annotation) {
        if let Some(scene) = self.scenes.get_mut(&scene_id) {
            scene.annotations.push(annotation);
        }
    }

    /// Get scene
    pub fn get_scene(&self, scene_id: Uuid) -> Option<&Scene3D> {
        self.scenes.get(&scene_id)
    }

    /// Export scene to JSON
    pub fn export_json(&self, scene_id: Uuid) -> Result<String, String> {
        let scene = self.scenes.get(&scene_id)
            .ok_or("Scene not found")?;

        serde_json::to_string_pretty(scene)
            .map_err(|e| format!("JSON serialization error: {}", e))
    }

    /// Calculate geometric center of atoms
    fn calculate_center(&self, atoms: &[VisAtom]) -> Point3D {
        if atoms.is_empty() {
            return Point3D::new(0.0, 0.0, 0.0);
        }

        let sum = atoms.iter().fold(Point3D::new(0.0, 0.0, 0.0), |acc, atom| {
            Point3D::new(
                acc.x + atom.position.x,
                acc.y + atom.position.y,
                acc.z + atom.position.z,
            )
        });

        let n = atoms.len() as f64;
        Point3D::new(sum.x / n, sum.y / n, sum.z / n)
    }

    /// List all scenes
    pub fn list_scenes(&self) -> Vec<(Uuid, String)> {
        self.scenes.iter()
            .map(|(id, scene)| (*id, scene.name.clone()))
            .collect()
    }
}

impl Default for Viz3DEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Get atomic radius for visualization (in Angstroms, scaled for display)
fn get_atom_radius(element: &str) -> f32 {
    match element {
        "H" => 0.5,
        "C" => 0.7,
        "N" => 0.65,
        "O" => 0.6,
        "F" => 0.5,
        "Si" => 1.1,
        "P" => 1.0,
        "S" => 1.0,
        "Cl" => 1.0,
        "Fe" => 1.2,
        "Co" => 1.2,
        "Ni" => 1.2,
        "Cu" => 1.3,
        "Zn" => 1.3,
        "Ag" => 1.6,
        "Au" => 1.6,
        "Ti" => 1.4,
        "Al" => 1.2,
        "Li" => 1.5,
        "Na" => 1.8,
        "K" => 2.2,
        "Ca" => 1.8,
        "Mg" => 1.5,
        "Mo" => 1.4,
        "B" => 0.8,
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point3d_distance() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(3.0, 4.0, 0.0);
        assert!((p1.distance(&p2) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_scene_creation() {
        let mut engine = Viz3DEngine::new();

        let atoms = vec![
            VisAtom::new(0, "Fe", Point3D::new(0.0, 0.0, 0.0)),
            VisAtom::new(1, "O", Point3D::new(1.0, 0.0, 0.0)),
        ];

        let bonds = vec![
            VisBond {
                from_atom: 0,
                to_atom: 1,
                bond_type: BondVisualizationType::Single,
                color: Color::new(0.5, 0.5, 0.5),
                thickness: 0.2,
            }
        ];

        let scene_id = engine.create_scene("Test", atoms, bonds);
        let scene = engine.get_scene(scene_id).unwrap();

        assert_eq!(scene.atoms.len(), 2);
        assert_eq!(scene.bonds.len(), 1);
    }

    #[test]
    fn test_unit_cell_cubic() {
        let cell = UnitCell::cubic(5.0);
        assert_eq!(cell.a.x, 5.0);
        assert_eq!(cell.alpha, 90.0);
    }

    #[test]
    fn test_color_from_element() {
        let color = Color::from_element("Fe");
        assert!(color.r > 0.0);
        assert!(color.a == 1.0);
    }
}
