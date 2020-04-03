use super::Circuit;

/// Creates a string representation of the circuit
/// Tries to keep in in alphabetical order
pub fn stringify_circuit(_circuit: &Circuit) -> Result<String, String> {
    Ok(String::from("This is a potato"))
}

#[cfg(test)]
mod tests {
    use super::super::ComponentKind;
    use super::*;

    #[test]
    fn not_works() {
        let mut circuit = Circuit::new();
        let input = circuit.add_component("A", ComponentKind::Input);
        let output = circuit.add_component("B", ComponentKind::Output);
        let not = circuit.add_component("NOT_1", ComponentKind::Not);

        circuit.add_connection(&input, &not);
        circuit.add_connection(&not, &output);

        assert_eq!(stringify_circuit(&circuit).unwrap(), String::from("B = !A"));
    }

    #[test]
    fn and_works() {
        let mut circuit = Circuit::new();
        let input = circuit.add_component("A", ComponentKind::Input);
        let input2 = circuit.add_component("B", ComponentKind::Input);
        let output = circuit.add_component("C", ComponentKind::Output);
        let and = circuit.add_component("AND_1", ComponentKind::And);

        circuit.add_connection(&input, &and);
        circuit.add_connection(&input2, &and);
        circuit.add_connection(&and, &output);

        assert_eq!(stringify_circuit(&circuit).unwrap(), String::from("C = AB"));
    }

    #[test]
    fn or_works() {
        let mut circuit = Circuit::new();
        let input = circuit.add_component("A", ComponentKind::Input);
        let input2 = circuit.add_component("B", ComponentKind::Input);
        let output = circuit.add_component("C", ComponentKind::Output);
        let or = circuit.add_component("OR_1", ComponentKind::Or);

        circuit.add_connection(&input, &or);
        circuit.add_connection(&input2, &or);
        circuit.add_connection(&or, &output);

        assert_eq!(
            stringify_circuit(&circuit).unwrap(),
            String::from("C = A + B")
        );
    }
}
