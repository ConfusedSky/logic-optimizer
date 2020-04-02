use petgraph::{graph::NodeIndex, Directed, Graph, Incoming, Outgoing};
use std::cmp::Ordering;
use std::convert::TryInto;

/// The direction of a connection
#[derive(Clone, Copy, Debug)]
enum ConnectionDirection {
    Input,
    Output,
}

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
            // 1 or more
            ComponentKind::And => (Ordering::Greater, 1),
            // 1 or more
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
            ComponentKind::And => (Ordering::Greater, 1),
            // 1 or more
            ComponentKind::Or => (Ordering::Greater, 1),
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
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ValidationErrorKind {
    IncorrectInputs,
    IncorrectOutputs,
    DuplicateName,
}

#[derive(Debug)]
pub struct ValidationError {
    component: ComponentData,
    kind: ValidationErrorKind,
}

impl ValidationError {
    pub fn new(kind: ValidationErrorKind, component: ComponentData) -> Self {
        Self { kind, component }
    }
}

impl Circuit {
    pub fn new() -> Self {
        Self {
            graph: Default::default(),
        }
    }

    /// Adds a component to the circuit
    pub fn add_component<S: Into<String>>(&mut self, name: S, kind: ComponentKind) -> Component {
        let index = self.graph.add_node(ComponentData {
            kind,
            name: name.into(),
        });

        Component { index }
    }

    /// Adds a connection between two components in the circuit
    pub fn add_connection(&mut self, from: &Component, to: &Component) {
        self.graph.add_edge(from.index, to.index, ());
    }

    /// Validates the circuit
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        use petgraph::visit::IntoNodeReferences;
        use std::collections::HashSet;

        let mut errors = Vec::new();
        let mut unique_names = HashSet::new();

        for (index, data) in self.graph.node_references() {
            // If the name of the currently viewed item is a duplicate push an error
            if !unique_names.insert(data.name.clone()) {
                errors.push(ValidationError::new(
                    ValidationErrorKind::DuplicateName,
                    data.clone(),
                ));
            }

            self.validate_component_connections(index, data.kind, ConnectionDirection::Input)
                .unwrap_or_else(|error| {
                    errors.push(ValidationError::new(error, data.clone()));
                });

            self.validate_component_connections(index, data.kind, ConnectionDirection::Output)
                .unwrap_or_else(|error| {
                    errors.push(ValidationError::new(error, data.clone()));
                });
        }

        if errors.len() == 0 {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn count_connections(&self, index: NodeIndex, direction: ConnectionDirection) -> usize {
        let direction = match direction {
            ConnectionDirection::Input => Incoming,
            ConnectionDirection::Output => Outgoing,
        };
        self.graph.edges_directed(index, direction).count()
    }

    fn validate_component_connections(
        &self,
        index: NodeIndex,
        kind: ComponentKind,
        direction: ConnectionDirection,
    ) -> Result<(), ValidationErrorKind> {
        let connections: isize = self.count_connections(index, direction).try_into().unwrap();

        if match kind.required_connections(direction) {
            (Ordering::Equal, x) => connections == x,
            (Ordering::Greater, x) => connections > x,
            (Ordering::Less, x) => connections < x,
        } {
            Ok(())
        } else {
            let error = match direction {
                ConnectionDirection::Input => ValidationErrorKind::IncorrectInputs,
                ConnectionDirection::Output => ValidationErrorKind::IncorrectOutputs,
            };
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_error_in_errors(
        errors: &Vec<ValidationError>,
        kind: ValidationErrorKind,
    ) -> (usize, &ValidationError) {
        errors
            .iter()
            .enumerate()
            .find(|x| x.1.kind == kind)
            .expect(format!("Didn't find error of kind: {:?}", kind).as_str())
    }

    /// Validates that there are exactly as many errors of each kind as expected
    fn validate_errors(mut errors: Vec<ValidationError>, expected: &[ValidationErrorKind]) {
        // Display all errors if they are not equal
        assert_eq!(errors.len(), expected.len(), "{:#?}", errors);

        for e in expected {
            let (index, _) = find_error_in_errors(&errors, *e);
            errors.remove(index);
        }

        assert_eq!(errors.len(), 0, "{:#?}", errors);
    }

    #[test]
    fn not_validation_works() {
        // Create a new circuit that negates an input
        let mut circuit = Circuit::new();
        let input = circuit.add_component("A", ComponentKind::Input);
        let output = circuit.add_component("B", ComponentKind::Output);
        let not = circuit.add_component("NOT_1", ComponentKind::Not);
        // Order matters here
        circuit.add_connection(&input, &not);
        circuit.add_connection(&not, &output);
        // Make sure that the circuit is in a valid state
        circuit.validate().unwrap();
    }

    #[test]
    fn not_extra_input_fails() {
        let mut circuit = Circuit::new();
        let input = circuit.add_component("A", ComponentKind::Input);
        let input2 = circuit.add_component("B", ComponentKind::Input);
        let output = circuit.add_component("C", ComponentKind::Output);
        let not = circuit.add_component("NOT_1", ComponentKind::Not);

        circuit.add_connection(&input, &not);
        circuit.add_connection(&input2, &not);
        circuit.add_connection(&not, &output);

        let errors = circuit
            .validate()
            .expect_err("Error expected when multiple inputs are attatched to a not");

        validate_errors(errors, &[ValidationErrorKind::IncorrectInputs]);
    }

    #[test]
    fn not_extra_output_fails() {
        let mut circuit = Circuit::new();
        let input = circuit.add_component("A", ComponentKind::Input);
        let output2 = circuit.add_component("B", ComponentKind::Output);
        let output = circuit.add_component("C", ComponentKind::Output);
        let not = circuit.add_component("NOT_1", ComponentKind::Not);

        circuit.add_connection(&input, &not);
        circuit.add_connection(&not, &output);
        circuit.add_connection(&not, &output2);

        let errors = circuit
            .validate()
            .expect_err("Error expected when multiple outputs are attatched to a not");

        validate_errors(errors, &[ValidationErrorKind::IncorrectOutputs]);
    }

    #[test]
    fn reports_multiple_errors() {
        let mut circuit = Circuit::new();
        let input = circuit.add_component("A", ComponentKind::Input);
        let input2 = circuit.add_component("B", ComponentKind::Input);
        let output2 = circuit.add_component("C", ComponentKind::Output);
        let output = circuit.add_component("D", ComponentKind::Output);
        let not = circuit.add_component("NOT_1", ComponentKind::Not);

        circuit.add_connection(&input, &not);
        circuit.add_connection(&input2, &not);
        circuit.add_connection(&not, &output);
        circuit.add_connection(&not, &output2);

        let errors = circuit
            .validate()
            .expect_err("Error expected when multiple outputs are attatched to a not");

        validate_errors(
            errors,
            &[
                ValidationErrorKind::IncorrectInputs,
                ValidationErrorKind::IncorrectOutputs,
            ],
        );
    }

    #[test]
    fn validate_input_inputs() {
        let mut circuit = Circuit::new();
        let input = circuit.add_component("A", ComponentKind::Input);
        let input2 = circuit.add_component("B", ComponentKind::Input);

        circuit.add_connection(&input, &input2);

        let errors = circuit
            .validate()
            .expect_err("Error expected when an input node has an input");
        validate_errors(errors, &[ValidationErrorKind::IncorrectInputs]);
    }

    #[test]
    fn validate_output_outputs() {
        let mut circuit = Circuit::new();
        let output = circuit.add_component("A", ComponentKind::Output);
        let output2 = circuit.add_component("B", ComponentKind::Output);

        circuit.add_connection(&output, &output2);

        let errors = circuit
            .validate()
            .expect_err("Error expected when an output node has an output");

        validate_errors(errors, &[ValidationErrorKind::IncorrectOutputs]);
    }

    #[test]
    fn valid_output_single_input() {
        let mut circuit = Circuit::new();
        let input = circuit.add_component("A", ComponentKind::Input);
        let input2 = circuit.add_component("A2", ComponentKind::Input);
        let output = circuit.add_component("B", ComponentKind::Output);

        circuit.add_connection(&input, &output);
        circuit.add_connection(&input2, &output);

        let errors = circuit
            .validate()
            .expect_err("Error expected when output node has mutliple inputs");

        validate_errors(errors, &[ValidationErrorKind::IncorrectInputs]);
    }

    #[test]
    fn test_duplicate_name() {
        let mut circuit = Circuit::new();
        let input = circuit.add_component("A", ComponentKind::Input);
        let output = circuit.add_component("A", ComponentKind::Output);

        circuit.add_connection(&input, &output);

        let errors = circuit
            .validate()
            .expect_err("Error expected two components have the same name");

        validate_errors(errors, &[ValidationErrorKind::DuplicateName]);
    }
}
