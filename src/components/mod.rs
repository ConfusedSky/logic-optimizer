use petgraph::graph::NodeIndex;
use std::cmp::Ordering;

mod circuit;
pub mod stringify;

use circuit::ConnectionDirection;

pub use circuit::{Circuit, ValidationError, ValidationErrorKind};

/// A single node in a logic graph
#[derive(Clone, Copy, Debug)]
pub enum ComponentKind {
    /// An input node
    Input,
    /// An output node
    Output,
    /// Takes the output of one node and negates it
    Not,
    /// Is high if all inputs are high otherwise it's low
    And,
    /// Is high if any of inputs are high otherwise it's low
    Or,
}

impl ComponentKind {
    /// The number of inputs required for a given component
    fn required_inputs(&self) -> (Ordering, isize) {
        match self {
            // Can't have any inputs
            ComponentKind::Input => (Ordering::Equal, 0),
            // 0 or 1
            ComponentKind::Output => (Ordering::Less, 2),
            // Exactly 1
            ComponentKind::Not => (Ordering::Equal, 1),
            // 2 or more
            ComponentKind::And => (Ordering::Greater, 1),
            // 2 or more
            ComponentKind::Or => (Ordering::Greater, 1),
        }
    }

    /// The number of outputs required for a given component
    fn required_outputs(&self) -> (Ordering, isize) {
        match self {
            // Can have 0 or more outputs
            ComponentKind::Input => (Ordering::Greater, -1),
            // Can't have any outputs
            ComponentKind::Output => (Ordering::Equal, 0),
            // 1 or more
            ComponentKind::Not => (Ordering::Equal, 1),
            // 1 or more
            ComponentKind::And => (Ordering::Greater, 0),
            // 1 or more
            ComponentKind::Or => (Ordering::Greater, 0),
        }
    }

    fn required_connections(&self, direction: ConnectionDirection) -> (Ordering, isize) {
        match direction {
            ConnectionDirection::Input => self.required_inputs(),
            ConnectionDirection::Output => self.required_outputs(),
        }
    }
}

#[derive(Debug, Clone)]
/// The data that is stored in the circuit tree
pub struct ComponentData {
    kind: ComponentKind,
    name: String,
}

#[derive(Debug)]
/// The public struct that is used to identify the component in the circuit using the public interface
pub struct Component {
    index: NodeIndex,
}
