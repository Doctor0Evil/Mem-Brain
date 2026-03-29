// crate: mem-brain
// path: mem-brain/src/mem_brain/spec.rs

use std::collections::HashMap;

/// Core 5-D coordinate: 3D space, time, and one extra "code" axis.
/// This does not assume invertibility; it is a descriptive index.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coord5D {
    pub x_mm: f32,
    pub y_mm: f32,
    pub z_mm: f32,
    pub t_ms: f32,
    pub code_axis: f32,
}

/// Biophysical layer tags.
/// These correspond to the 5 biophysical layers in the README specification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BioLayer {
    Molecular,
    Synaptic,
    Microcircuit,
    MesoscopicNetwork,
    SystemicContext,
}

/// A local state vector for one layer at one 5-D coordinate.
/// `state` is intentionally abstract: it can be a small fixed-length vector
/// or a compressed code derived from real data.
#[derive(Clone, Debug)]
pub struct LayerState {
    pub layer: BioLayer,
    pub coord: Coord5D,
    pub state: Vec<f32>,
}

/// A memory trace is a finite collection of layer states over 5-D coordinates,
/// with non-invertible compression guarantees at the aggregate level.
/// No reverse mapping to raw brain data is provided or allowed.
#[derive(Clone, Debug)]
pub struct MemoryTrace {
    pub id: String,
    pub layers: Vec<BioLayer>,
    pub states: Vec<LayerState>,
    pub invariants: HashMap<String, f32>,
}

impl MemoryTrace {
    /// Construct a new empty memory trace anchored to given layers.
    pub fn new(id: impl Into<String>, layers: Vec<BioLayer>) -> Self {
        Self {
            id: id.into(),
            layers,
            states: Vec::new(),
            invariants: HashMap::new(),
        }
    }

    /// Append a new layer state. This is a monotonic growth operation:
    /// it never deletes or overwrites existing states.
    pub fn append_state(&mut self, s: LayerState) {
        self.states.push(s);
    }

    /// Register or update an invariant, such as total energy budget or
    /// sparsity level. Invariants are summary statistics that can be
    /// checked by external tools.
    pub fn set_invariant(&mut self, name: impl Into<String>, value: f32) {
        self.invariants.insert(name.into(), value);
    }

    /// Compute a simple monotone complexity score: it must never decrease
    /// as more states are appended. This can be used by external systems
    /// to enforce non-degrading updates.
    pub fn complexity_score(&self) -> f32 {
        let n = self.states.len() as f32;
        let unique_layers = self.layers.len() as f32;
        // Simple monotone function: grows with number of states and layers.
        (n.ln().max(0.0) + 1.0) * unique_layers
    }
}

/// A "projection" mapping from an arbitrary feature tensor into a
/// MemoryTrace. In practice, this would be implemented to consume
/// real imaging or EEG features, but here we just define the interface.
pub trait MemoryProjector {
    /// Project raw features into a MemoryTrace.
    /// This operation must be non-invertible by design: there is no
    /// access to the original raw feature store once projected.
    fn project(&self, features: &[f32]) -> MemoryTrace;
}
