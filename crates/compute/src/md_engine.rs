//! Molecular Dynamics Engine
//!
//! Classical molecular dynamics simulation engine for materials using:
//! - Velocity Verlet integration
//! - Lennard-Jones potentials
//! - Periodic boundary conditions
//! - Thermostats (NVE, NVT, NPT)
//! - Force field support

use crate::{ComputationMethod, Error, Result};
use materials_core::Material;
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{debug, info};

// ============================================================================
// TYPES
// ============================================================================

/// 3D vector for positions, velocities, forces
#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn dot(&self, other: &Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn distance_to(&self, other: &Vec3) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn add(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub fn sub(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub fn scale(&self, scalar: f64) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

/// Atom in MD simulation
#[derive(Debug, Clone)]
pub struct Atom {
    pub id: usize,
    pub element: String,
    pub mass: f64,  // amu
    pub position: Vec3,
    pub velocity: Vec3,
    pub force: Vec3,
}

impl Atom {
    pub fn new(id: usize, element: String, position: Vec3) -> Self {
        let mass = Self::atomic_mass(&element);
        Self {
            id,
            element,
            mass,
            position,
            velocity: Vec3::zero(),
            force: Vec3::zero(),
        }
    }

    fn atomic_mass(element: &str) -> f64 {
        match element {
            "H" => 1.008,
            "C" => 12.011,
            "N" => 14.007,
            "O" => 15.999,
            "F" => 18.998,
            "Na" => 22.990,
            "Mg" => 24.305,
            "Al" => 26.982,
            "Si" => 28.085,
            "P" => 30.974,
            "S" => 32.065,
            "Cl" => 35.453,
            "Ca" => 40.078,
            "Ti" => 47.867,
            "Fe" => 55.845,
            "Ni" => 58.693,
            "Cu" => 63.546,
            "Zn" => 65.38,
            _ => 1.0,
        }
    }
}

// ============================================================================
// POTENTIAL MODELS
// ============================================================================

/// Lennard-Jones potential parameters
#[derive(Debug, Clone)]
pub struct LJParams {
    pub epsilon: f64,  // Well depth (eV)
    pub sigma: f64,    // Distance parameter (Angstrom)
}

impl LJParams {
    /// Get default LJ parameters for element pairs
    pub fn for_pair(elem1: &str, elem2: &str) -> Self {
        // Simplified - in real implementation would use mixing rules
        let (eps1, sig1) = Self::get_params(elem1);
        let (eps2, sig2) = Self::get_params(elem2);

        // Lorentz-Berthelot mixing rules
        let epsilon = (eps1 * eps2).sqrt();
        let sigma = (sig1 + sig2) / 2.0;

        Self { epsilon, sigma }
    }

    fn get_params(element: &str) -> (f64, f64) {
        // (epsilon in eV, sigma in Angstrom)
        match element {
            "H" => (0.002, 2.5),
            "C" => (0.005, 3.4),
            "N" => (0.004, 3.3),
            "O" => (0.006, 3.0),
            "Fe" => (0.010, 2.5),
            "Cu" => (0.012, 2.6),
            "Al" => (0.008, 2.7),
            _ => (0.005, 3.0),
        }
    }

    /// Calculate Lennard-Jones energy
    pub fn energy(&self, r: f64) -> f64 {
        if r < 0.5 {
            // Avoid singularity
            return 1.0e10;
        }

        let sr = self.sigma / r;
        let sr6 = sr.powi(6);
        let sr12 = sr6 * sr6;

        4.0 * self.epsilon * (sr12 - sr6)
    }

    /// Calculate Lennard-Jones force (magnitude)
    pub fn force(&self, r: f64) -> f64 {
        if r < 0.5 {
            return 0.0;
        }

        let sr = self.sigma / r;
        let sr6 = sr.powi(6);
        let sr12 = sr6 * sr6;

        24.0 * self.epsilon * (2.0 * sr12 - sr6) / r
    }
}

// ============================================================================
// MD CONFIGURATION
// ============================================================================

/// Ensemble type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ensemble {
    NVE,  // Microcanonical (constant energy)
    NVT,  // Canonical (constant temperature)
    NPT,  // Isothermal-isobaric (constant pressure)
}

/// MD simulation configuration
#[derive(Debug, Clone)]
pub struct MDConfig {
    pub timestep: f64,           // fs
    pub num_steps: usize,
    pub temperature: f64,        // K
    pub pressure: Option<f64>,   // GPa
    pub ensemble: Ensemble,
    pub output_freq: usize,      // Output every N steps
    pub cutoff_radius: f64,      // Angstrom
    pub use_pbc: bool,           // Periodic boundary conditions
    pub box_size: [f64; 3],      // Angstrom
}

impl Default for MDConfig {
    fn default() -> Self {
        Self {
            timestep: 1.0,  // 1 fs
            num_steps: 1000,
            temperature: 300.0,  // K
            pressure: None,
            ensemble: Ensemble::NVE,
            output_freq: 100,
            cutoff_radius: 10.0,  // Angstrom
            use_pbc: true,
            box_size: [20.0, 20.0, 20.0],
        }
    }
}

// ============================================================================
// MD STATE
// ============================================================================

/// Current state of MD simulation
#[derive(Debug)]
pub struct MDState {
    pub atoms: Vec<Atom>,
    pub config: MDConfig,
    pub current_step: usize,
    pub total_energy: f64,
    pub kinetic_energy: f64,
    pub potential_energy: f64,
    pub temperature: f64,
    pub lj_params: HashMap<(String, String), LJParams>,
}

impl MDState {
    pub fn new(atoms: Vec<Atom>, config: MDConfig) -> Self {
        let mut state = Self {
            atoms,
            config,
            current_step: 0,
            total_energy: 0.0,
            kinetic_energy: 0.0,
            potential_energy: 0.0,
            temperature: 0.0,
            lj_params: HashMap::new(),
        };

        // Initialize LJ parameters for all atom pairs
        state.initialize_lj_params();

        state
    }

    fn initialize_lj_params(&mut self) {
        let elements: Vec<String> = self.atoms.iter()
            .map(|a| a.element.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for elem1 in &elements {
            for elem2 in &elements {
                let key = if elem1 <= elem2 {
                    (elem1.clone(), elem2.clone())
                } else {
                    (elem2.clone(), elem1.clone())
                };

                let params = LJParams::for_pair(elem1, elem2);
                self.lj_params.insert(key, params);
            }
        }
    }

    /// Apply periodic boundary conditions
    fn apply_pbc(&self, pos: &mut Vec3) {
        if !self.config.use_pbc {
            return;
        }

        let [lx, ly, lz] = self.config.box_size;

        pos.x = pos.x - (pos.x / lx).floor() * lx;
        pos.y = pos.y - (pos.y / ly).floor() * ly;
        pos.z = pos.z - (pos.z / lz).floor() * lz;
    }

    /// Minimum image convention for PBC
    fn minimum_image(&self, dx: f64, box_length: f64) -> f64 {
        if !self.config.use_pbc {
            return dx;
        }

        dx - box_length * (dx / box_length).round()
    }
}

// ============================================================================
// MD ENGINE
// ============================================================================

pub struct MDEngine {
    config: MDConfig,
}

impl MDEngine {
    pub fn new() -> Self {
        Self {
            config: MDConfig::default(),
        }
    }

    pub fn with_config(config: MDConfig) -> Self {
        Self { config }
    }

    /// Run MD simulation
    pub fn run_simulation(&self, mut state: MDState) -> Result<MDTrajectory> {
        info!("Starting MD simulation: {} steps, T={} K, dt={} fs",
            state.config.num_steps, state.config.temperature, state.config.timestep);

        let mut trajectory = MDTrajectory::new();

        // Initialize velocities
        self.initialize_velocities(&mut state);

        // Main MD loop
        for step in 0..state.config.num_steps {
            state.current_step = step;

            // Velocity Verlet integration
            self.velocity_verlet_step(&mut state)?;

            // Apply thermostat if needed
            match state.config.ensemble {
                Ensemble::NVT => {
                    self.apply_berendsen_thermostat(&mut state);
                }
                Ensemble::NPT => {
                    self.apply_berendsen_thermostat(&mut state);
                    // TODO: Add barostat
                }
                Ensemble::NVE => {
                    // No thermostat for NVE
                }
            }

            // Calculate energies and temperature
            self.calculate_energies(&mut state);

            // Save snapshot
            if step % state.config.output_freq == 0 {
                trajectory.add_frame(state.atoms.clone(), state.total_energy, state.temperature);
                debug!("Step {}: E_tot={:.4} eV, T={:.2} K",
                    step, state.total_energy, state.temperature);
            }
        }

        info!("MD simulation completed: {} frames", trajectory.frames.len());

        Ok(trajectory)
    }

    /// Initialize Maxwell-Boltzmann velocities
    fn initialize_velocities(&self, state: &mut MDState) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let kb = 8.617333e-5;  // Boltzmann constant in eV/K
        let target_temp = state.config.temperature;

        for atom in &mut state.atoms {
            // Sample from Maxwell-Boltzmann distribution
            let sigma = (kb * target_temp / atom.mass).sqrt();

            atom.velocity = Vec3::new(
                rng.gen::<f64>() * sigma * 2.0 - sigma,
                rng.gen::<f64>() * sigma * 2.0 - sigma,
                rng.gen::<f64>() * sigma * 2.0 - sigma,
            );
        }

        // Remove center-of-mass motion
        self.remove_com_motion(state);
    }

    /// Remove center of mass motion
    fn remove_com_motion(&self, state: &mut MDState) {
        let total_mass: f64 = state.atoms.iter().map(|a| a.mass).sum();
        let mut com_velocity = Vec3::zero();

        for atom in &state.atoms {
            com_velocity = com_velocity.add(&atom.velocity.scale(atom.mass));
        }

        com_velocity = com_velocity.scale(1.0 / total_mass);

        for atom in &mut state.atoms {
            atom.velocity = atom.velocity.sub(&com_velocity);
        }
    }

    /// Velocity Verlet integration step
    fn velocity_verlet_step(&self, state: &mut MDState) -> Result<()> {
        let dt = state.config.timestep * 1.0e-15;  // Convert fs to seconds
        let dt2 = dt * dt;

        // Step 1: Update positions
        let use_pbc = state.config.use_pbc;
        let box_size = state.config.box_size;

        for atom in &mut state.atoms {
            let acc = atom.force.scale(1.0 / atom.mass);

            atom.position.x += atom.velocity.x * dt + 0.5 * acc.x * dt2;
            atom.position.y += atom.velocity.y * dt + 0.5 * acc.y * dt2;
            atom.position.z += atom.velocity.z * dt + 0.5 * acc.z * dt2;

            // Apply PBC manually here to avoid borrowing issues
            if use_pbc {
                atom.position.x = atom.position.x - (atom.position.x / box_size[0]).floor() * box_size[0];
                atom.position.y = atom.position.y - (atom.position.y / box_size[1]).floor() * box_size[1];
                atom.position.z = atom.position.z - (atom.position.z / box_size[2]).floor() * box_size[2];
            }
        }

        // Step 2: Calculate forces at new positions
        let old_forces: Vec<Vec3> = state.atoms.iter().map(|a| a.force).collect();
        self.calculate_forces(state);

        // Step 3: Update velocities
        for (i, atom) in state.atoms.iter_mut().enumerate() {
            let acc_old = old_forces[i].scale(1.0 / atom.mass);
            let acc_new = atom.force.scale(1.0 / atom.mass);

            atom.velocity.x += 0.5 * (acc_old.x + acc_new.x) * dt;
            atom.velocity.y += 0.5 * (acc_old.y + acc_new.y) * dt;
            atom.velocity.z += 0.5 * (acc_old.z + acc_new.z) * dt;
        }

        Ok(())
    }

    /// Calculate forces using Lennard-Jones potential
    fn calculate_forces(&self, state: &mut MDState) {
        // Reset forces
        for atom in &mut state.atoms {
            atom.force = Vec3::zero();
        }

        let cutoff = state.config.cutoff_radius;
        let num_atoms = state.atoms.len();

        // Pairwise force calculation
        for i in 0..num_atoms {
            for j in (i + 1)..num_atoms {
                let elem1 = state.atoms[i].element.clone();
                let elem2 = state.atoms[j].element.clone();

                let key = if elem1 <= elem2 {
                    (elem1, elem2)
                } else {
                    (elem2, elem1)
                };

                let lj = state.lj_params.get(&key).unwrap();

                // Calculate distance with PBC
                let mut dx = state.atoms[j].position.x - state.atoms[i].position.x;
                let mut dy = state.atoms[j].position.y - state.atoms[i].position.y;
                let mut dz = state.atoms[j].position.z - state.atoms[i].position.z;

                dx = state.minimum_image(dx, state.config.box_size[0]);
                dy = state.minimum_image(dy, state.config.box_size[1]);
                dz = state.minimum_image(dz, state.config.box_size[2]);

                let r = (dx * dx + dy * dy + dz * dz).sqrt();

                if r < cutoff {
                    let f_mag = lj.force(r);

                    let fx = f_mag * dx / r;
                    let fy = f_mag * dy / r;
                    let fz = f_mag * dz / r;

                    // Newton's third law
                    state.atoms[i].force.x += fx;
                    state.atoms[i].force.y += fy;
                    state.atoms[i].force.z += fz;

                    state.atoms[j].force.x -= fx;
                    state.atoms[j].force.y -= fy;
                    state.atoms[j].force.z -= fz;
                }
            }
        }
    }

    /// Calculate kinetic and potential energies
    fn calculate_energies(&self, state: &mut MDState) {
        // Kinetic energy
        let mut ke = 0.0;
        for atom in &state.atoms {
            let v2 = atom.velocity.dot(&atom.velocity);
            ke += 0.5 * atom.mass * v2;
        }
        state.kinetic_energy = ke;

        // Potential energy (Lennard-Jones)
        let mut pe = 0.0;
        let num_atoms = state.atoms.len();
        for i in 0..num_atoms {
            for j in (i + 1)..num_atoms {
                let r = state.atoms[i].position.distance_to(&state.atoms[j].position);

                if r < state.config.cutoff_radius {
                    let elem1 = &state.atoms[i].element;
                    let elem2 = &state.atoms[j].element;

                    let key = if elem1 <= elem2 {
                        (elem1.clone(), elem2.clone())
                    } else {
                        (elem2.clone(), elem1.clone())
                    };

                    if let Some(lj) = state.lj_params.get(&key) {
                        pe += lj.energy(r);
                    }
                }
            }
        }
        state.potential_energy = pe;

        // Total energy
        state.total_energy = ke + pe;

        // Temperature from kinetic energy
        let kb = 8.617333e-5;  // eV/K
        let num_dof = 3.0 * state.atoms.len() as f64 - 3.0;  // 3N - 3 (remove COM translation)
        state.temperature = 2.0 * ke / (kb * num_dof);
    }

    /// Berendsen thermostat for NVT
    fn apply_berendsen_thermostat(&self, state: &mut MDState) {
        let tau = 100.0;  // Coupling time in fs
        let target_temp = state.config.temperature;
        let current_temp = state.temperature;

        if current_temp > 0.0 {
            let lambda = (1.0 + state.config.timestep / tau * (target_temp / current_temp - 1.0)).sqrt();

            for atom in &mut state.atoms {
                atom.velocity = atom.velocity.scale(lambda);
            }
        }
    }
}

impl Default for MDEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ComputationMethod for MDEngine {
    async fn calculate_energy(&self, material: &Material) -> Result<f64> {
        // Run a short MD simulation and return average energy
        let atoms: Vec<Atom> = material.elements()
            .into_iter()
            .enumerate()
            .map(|(i, elem)| {
                let pos = Vec3::new(
                    (i as f64 * 2.0) % 10.0,
                    ((i / 5) as f64 * 2.0) % 10.0,
                    ((i / 25) as f64 * 2.0) % 10.0,
                );
                Atom::new(i, elem, pos)
            })
            .collect();

        let config = MDConfig {
            num_steps: 100,
            ..Default::default()
        };

        let state = MDState::new(atoms, config.clone());
        let engine = MDEngine::with_config(config);
        let trajectory = engine.run_simulation(state)?;

        // Return average energy from last half of trajectory
        let half = trajectory.frames.len() / 2;
        let avg_energy: f64 = trajectory.frames[half..]
            .iter()
            .map(|f| f.total_energy)
            .sum::<f64>() / (trajectory.frames.len() - half) as f64;

        Ok(avg_energy)
    }

    async fn calculate_forces(&self, _material: &Material) -> Result<Vec<[f64; 3]>> {
        // Would need to extract forces from MD state
        Err(Error::Other("Force calculation not implemented in MDEngine".to_string()))
    }

    fn cost_estimate(&self, material: &Material) -> f64 {
        let num_atoms = material.num_atoms() as f64;
        // O(N^2) for pairwise forces, scaled by number of steps
        num_atoms * num_atoms * self.config.num_steps as f64 * 0.001
    }

    fn name(&self) -> &str {
        "MDEngine"
    }
}

// ============================================================================
// TRAJECTORY
// ============================================================================

/// MD trajectory (time series of configurations)
#[derive(Debug)]
pub struct MDTrajectory {
    pub frames: Vec<MDFrame>,
}

impl MDTrajectory {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    pub fn add_frame(&mut self, atoms: Vec<Atom>, total_energy: f64, temperature: f64) {
        self.frames.push(MDFrame {
            atoms,
            total_energy,
            temperature,
        });
    }

    /// Calculate radial distribution function g(r)
    pub fn calculate_rdf(&self, max_r: f64, num_bins: usize) -> Vec<(f64, f64)> {
        // Simplified RDF calculation
        let mut rdf = vec![0.0; num_bins];
        let dr = max_r / num_bins as f64;

        // Average over all frames
        for frame in &self.frames {
            let num_atoms = frame.atoms.len();

            for i in 0..num_atoms {
                for j in (i + 1)..num_atoms {
                    let r = frame.atoms[i].position.distance_to(&frame.atoms[j].position);

                    if r < max_r {
                        let bin = (r / dr) as usize;
                        if bin < num_bins {
                            rdf[bin] += 2.0;  // Count both i-j and j-i
                        }
                    }
                }
            }
        }

        // Normalize
        let num_frames = self.frames.len() as f64;
        rdf.iter()
            .enumerate()
            .map(|(i, &count)| {
                let r = (i as f64 + 0.5) * dr;
                let volume = 4.0 / 3.0 * std::f64::consts::PI * ((r + dr).powi(3) - r.powi(3));
                let g = count / (num_frames * volume);
                (r, g)
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct MDFrame {
    pub atoms: Vec<Atom>,
    pub total_energy: f64,
    pub temperature: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_operations() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);

        let sum = v1.add(&v2);
        assert_eq!(sum.x, 5.0);
        assert_eq!(sum.y, 7.0);
        assert_eq!(sum.z, 9.0);

        let dot = v1.dot(&v2);
        assert_eq!(dot, 32.0);
    }

    #[test]
    fn test_lj_potential() {
        let lj = LJParams {
            epsilon: 0.005,
            sigma: 3.0,
        };

        let energy = lj.energy(3.5);
        assert!(energy.is_finite());
        assert!(energy < 0.0);  // Attractive at equilibrium

        let force = lj.force(3.5);
        assert!(force.is_finite());
    }

    #[test]
    fn test_md_state_creation() {
        let atoms = vec![
            Atom::new(0, "C".to_string(), Vec3::new(0.0, 0.0, 0.0)),
            Atom::new(1, "O".to_string(), Vec3::new(1.5, 0.0, 0.0)),
        ];

        let config = MDConfig::default();
        let state = MDState::new(atoms, config);

        assert_eq!(state.atoms.len(), 2);
        assert!(!state.lj_params.is_empty());
    }

    #[test]
    fn test_md_engine() {
        let engine = MDEngine::new();
        let atoms = vec![
            Atom::new(0, "Ar".to_string(), Vec3::new(0.0, 0.0, 0.0)),
            Atom::new(1, "Ar".to_string(), Vec3::new(3.5, 0.0, 0.0)),
        ];

        let mut config = MDConfig::default();
        config.num_steps = 10;
        config.output_freq = 5;

        let state = MDState::new(atoms, config.clone());
        let result = engine.with_config(config).run_simulation(state);

        assert!(result.is_ok());
        let trajectory = result.unwrap();
        assert!(!trajectory.frames.is_empty());
    }
}
