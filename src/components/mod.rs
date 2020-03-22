use petgraph::{graph::NodeIndex, Directed, Graph, Incoming, Outgoing};

/// A single node in a logic graph
#[derive(Clone, Copy, Debug)]
pub enum ComponentKind {
    /// An input node
    Input,
    /// An output node
    Output,
    /// Takes the output of one node and negates it
    Not,
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

/// Circuit is a representation of a logic circuit using an underlying graph structure
///
/// ## Examples
/// ``` rust
/// use logic_optimizer::components::{Circuit, ComponentKind};
/// // Create a new circuit that negates an input
/// let mut circuit = Circuit::new();
/// let input = circuit.add_component("A", ComponentKind::Input);
/// let output = circuit.add_component("B", ComponentKind::Output);
/// let not = circuit.add_component("NOT_1", ComponentKind::Not);
/// // Order matters here
/// circuit.add_connection(&input, &not);
/// circuit.add_connection(&not, &output);
///
/// // Make sure that the circuit is in a valid state
/// circuit.validate().unwrap();
///
/// ```
#[derive(Debug)]
pub struct Circuit {
    graph: Graph<ComponentData, (), Directed>,
}

/// Types of error that may be returned from a failed validation
#[derive(Debug)]
pub enum ValidationError {
    IncorrectInputs(ComponentData),
    IncorrectOutputs(ComponentData),
}

impl Circuit {
    pub fn new() -> Self {
        Self {
            graph: Default::default(),
        }
    }

    pub fn add_component<S: Into<String>>(&mut self, name: S, kind: ComponentKind) -> Component {
        let index = self.graph.add_node(ComponentData {
            kind,
            name: name.into(),
        });

        Component { index }
    }

    pub fn add_connection(&mut self, from: &Component, to: &Component) {
        self.graph.add_edge(from.index, to.index, ());
    }

    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        use petgraph::visit::IntoNodeReferences;
        let mut errors = Vec::new();

        for (index, data) in self.graph.node_references() {
            match data.kind {
                ComponentKind::Input => {
                    // Input nodes expect that they don't have an inputs
                    if self.count_inputs(index) > 0 {
                        errors.push(ValidationError::IncorrectInputs(data.clone()));
                    }
                }
                ComponentKind::Output => {
                    // Output nodes expect that they don't have any outputs
                    if self.count_outputs(index) > 0 {
                        errors.push(ValidationError::IncorrectOutputs(data.clone()));
                    }
                }
                ComponentKind::Not => {
                    // Nots should have exactly one input and exactly one output
                    // subject to change
                    if self.count_inputs(index) != 1 {
                        errors.push(ValidationError::IncorrectInputs(data.clone()));
                    }
                    if self.count_outputs(index) != 1 {
                        errors.push(ValidationError::IncorrectOutputs(data.clone()));
                    }
                }
            };
        }

        if errors.len() == 0 {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn count_inputs(&self, index: NodeIndex) -> usize {
        self.graph.edges_directed(index, Incoming).count()
    }
    fn count_outputs(&self, index: NodeIndex) -> usize {
        self.graph.edges_directed(index, Outgoing).count()
    }
}
