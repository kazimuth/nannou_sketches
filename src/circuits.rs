use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::HashMap;

/// The type carried by wires.
pub type Value = bool;

/// A gate.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Gate {
    Or,
    And,
    Xor,
    Not,
    Output,
    Input,
    MetaInput, // inserted before all inputs
}

/// A simulated digital "circuit". Must be a DAG.
///
/// Input values come from a single MetaInput; their values can be changed using the `set_input` method.
///
/// Provides methods to build up a circuit programmatically. Methods to create some circuit node
/// return a `NodeIndex` which can be used to read the output of that node.
pub struct Circuit(pub DiGraph<Gate, Value>);

impl Circuit {
    // -- helpers --
    pub fn meta_input() -> NodeIndex {
        NodeIndex::new(0)
    }

    // -- construction functions; check invariants frequently, slow
    pub fn new() -> Circuit {
        let mut graph = DiGraph::new();
        graph.add_node(Gate::MetaInput);
        let result = Circuit(graph);
        result.check_invariants();
        result
    }

    /// Check a graph's invariants, panicking if they fail.
    pub fn check_invariants(&self) {
        let meta_type = self.0[Circuit::meta_input()];
        assert_eq!(meta_type, Gate::MetaInput, "meta input is the wrong type");
        assert!(
            !petgraph::algo::is_cyclic_directed(&self.0),
            "graph is cyclic"
        );
        assert!(
            self.0
                .edges_directed(Circuit::meta_input(), Direction::Incoming)
                .next()
                .is_none(),
            "meta input has inputs"
        );
    }

    pub fn add_input(&mut self) -> NodeIndex {
        let input = self.0.add_node(Gate::Input);
        self.0.update_edge(Circuit::meta_input(), input, false);
        self.check_invariants();
        input
    }
    pub fn add_or(&mut self, a: NodeIndex, b: NodeIndex) -> NodeIndex {
        let result = self.0.add_node(Gate::Or);
        self.0.update_edge(a, result, false);
        self.0.update_edge(b, result, false);
        self.check_invariants();
        result
    }
    pub fn add_xor(&mut self, a: NodeIndex, b: NodeIndex) -> NodeIndex {
        let result = self.0.add_node(Gate::Xor);
        self.0.update_edge(a, result, false);
        self.0.update_edge(b, result, false);
        self.check_invariants();
        result
    }
    pub fn add_and(&mut self, a: NodeIndex, b: NodeIndex) -> NodeIndex {
        let result = self.0.add_node(Gate::And);
        self.0.update_edge(a, result, false);
        self.0.update_edge(b, result, false);
        self.check_invariants();
        result
    }
    pub fn add_not(&mut self, a: NodeIndex) -> NodeIndex {
        let result = self.0.add_node(Gate::Not);
        self.0.update_edge(a, result, false);
        self.check_invariants();
        result
    }
    pub fn add_output(&mut self, a: NodeIndex) -> NodeIndex {
        let result = self.0.add_node(Gate::Output);
        self.0.update_edge(a, result, false);
        self.check_invariants();
        result
    }

    // -- slow processing algorithms --

    /// Compute a series of ranks.
    /// Each rank has inputs only from previous ranks.
    pub fn ranks(&self) -> HashMap<NodeIndex, u32> {
        self.check_invariants();

        let new_graph = self.0.map(|_, _| (), |_, _| -1.0f32);

        let (path_lens, _) =
            petgraph::algo::bellman_ford(&new_graph, Circuit::meta_input()).unwrap();
        let mut ranks = HashMap::new();
        for (node, path_len) in self.0.node_indices().zip(&path_lens) {
            ranks.insert(node, (-*path_len) as u32);
        }

        ranks
    }

    // -- fast processing algorithms --

    /// Set a single input.
    pub fn set_input(&mut self, input: NodeIndex, value: Value) {
        assert_eq!(self.0[input], Gate::Input);
        self.0.update_edge(Circuit::meta_input(), input, value);
    }

    /// Get 1 signal into a gate. There *must* be only 1 signal.
    pub fn get_1_in(&self, gate: NodeIndex) -> Value {
        let gate_type = self.0[gate];
        assert!(
            gate_type == Gate::Input || gate_type == Gate::Output || gate_type == Gate::Not,
            "gate {:?} should be Input, Output, or Not, is {:?}",
            gate,
            gate_type
        );

        let mut edges = self.0.edges_directed(gate, Direction::Incoming);

        let edge = edges.next();
        let none = edges.next();

        match (edge, none) {
            (Some(edge), None) => *edge.weight(),
            _ => panic!("gate {} should only have 1 input"),
        }
    }
    /// Get 2 signals into a gate. There *must* be precisely 2 signals.
    pub fn get_2_in(&self, gate: NodeIndex) -> (Value, Value) {
        let gate_type = self.0[gate];
        assert!(
            gate_type == Gate::Or || gate_type == Gate::Xor || gate_type == Gate::And,
            "gate {:?} should be Or or Xor, is {:?}",
            gate,
            gate_type
        );

        let mut edges = self.0.edges_directed(gate, Direction::Incoming);

        let a = edges.next();
        let b = edges.next();
        let none = edges.next();

        match (a, b, none) {
            (Some(a), Some(b), None) => (*a.weight(), *b.weight()),
            _ => panic!("gate {} should have precisely 2 inputs"),
        }
    }
    /// Compute the order to update nodes in.
    pub fn update_order(&self) -> Vec<NodeIndex> {
        let mut result = petgraph::algo::toposort(&self.0, None).unwrap();
        result.reverse();
        result
    }
    /// Propagate signals a single step forward.
    pub fn update_signals_once(&mut self, order: &[NodeIndex]) {
        let mut edges = vec![];
        for gate in order {
            let gate = *gate;
            let gate_type = self.0[gate];

            let value = match gate_type {
                Gate::Or => {
                    let (a, b) = self.get_2_in(gate);
                    a | b
                }
                Gate::Xor => {
                    let (a, b) = self.get_2_in(gate);
                    a ^ b
                }
                Gate::And => {
                    let (a, b) = self.get_2_in(gate);
                    a & b
                }
                Gate::Not => !self.get_1_in(gate),
                Gate::Input | Gate::Output => self.get_1_in(gate),
                Gate::MetaInput => continue,
            };

            edges.extend(
                self.0
                    .edges_directed(gate, Direction::Outgoing)
                    .map(|e| e.id()),
            );
            for edge in &edges {
                let w = &mut self.0[*edge];
                *w = value;
            }
            edges.clear();
        }
    }

    /// Build a half adder. Returns nodes (sum, carry).
    /// returns (s, c)
    pub fn half_adder(&mut self, a: NodeIndex, b: NodeIndex) -> (NodeIndex, NodeIndex) {
        let s = self.add_xor(a, b);
        let c = self.add_and(a, b);
        (s, c)
    }
    /// Build a full adder. Returns nodes (sum, carry_out).
    /// returns (s, c_out)
    pub fn full_adder(
        &mut self,
        a: NodeIndex,
        b: NodeIndex,
        c_in: NodeIndex,
    ) -> (NodeIndex, NodeIndex) {
        let a_x_b = self.add_xor(a, b);
        let s = self.add_xor(a_x_b, c_in);

        let i1 = self.add_and(a, b);
        let i2 = self.add_and(c_in, a_x_b);
        let c_out = self.add_or(i1, i2);
        (s, c_out)
    }
    /// Build a ripple-carry adder.
    /// Returns a vector of sum bits and the final carry bit.
    /// Sum bits are ordered by magnitude, i.e. `v[0]` corresponds to to `2**0`, `v[1]` to `2**1`, etc.
    /// Inputs should be ordered similarly.
    pub fn ripple_carry(
        &mut self,
        a: &[NodeIndex],
        b: &[NodeIndex],
    ) -> (Vec<NodeIndex>, NodeIndex) {
        assert_eq!(a.len(), b.len());

        let (s0, c0) = self.half_adder(a[0], b[0]);
        let mut out = vec![s0];
        let mut c = c0;

        for i in 1..a.len() {
            let (si, ci) = self.full_adder(a[i], b[i], c);
            out.push(si);
            c = ci;
        }
        assert_eq!(a.len(), out.len());
        (out, c)
    }
}

/// Given a hash table mapping nodes to their rank in the circuit,
/// return a vector of ranks, where each rank is a vector of the nodes in that rank.
pub fn flip_ranks(ranks: &HashMap<NodeIndex, u32>) -> Vec<Vec<NodeIndex>> {
    let n = ranks.values().max().unwrap();
    let mut result = vec![];
    for _ in 0..n + 1 {
        result.push(vec![]);
    }
    for (n, rank) in ranks {
        result[*rank as usize].push(*n)
    }
    for rank in &mut result {
        rank.sort_by_key(|n| n.index());
    }
    result
}

pub fn get_bit(v: usize, b: usize) -> bool {
    ((v >> b) & 1) == 1
}
pub fn set_bit(v: usize, b: usize, on: bool) -> usize {
    if on {
        v | (1 << b)
    } else {
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_circuit() {
        let mut circuit = Circuit::new();
        let a = circuit.add_input();
        let b = circuit.add_input();
        let x = circuit.add_xor(a, b);
        let out = circuit.add_output(x);
        circuit.set_input(a, true);

        let order = circuit.update_order();
        for _ in 0..5 {
            circuit.update_signals_once(&order);
        }

        assert_eq!(circuit.get_1_in(out), true);

        let ranks = circuit.ranks();

        let flipped = flip_ranks(&ranks);
        assert_eq!(&flipped[0], &[Circuit::meta_input()]);
        assert_eq!(&flipped[1], &[a, b]);
        assert_eq!(&flipped[2], &[x]);
        assert_eq!(&flipped[3], &[out]);
    }

    #[test]
    fn test_full_adder() {
        let mut circuit = Circuit::new();
        let a = circuit.add_input();
        let b = circuit.add_input();
        let c_in = circuit.add_input();

        // S = A ⊕ B ⊕ Cin
        // Cout = (A ⋅ B) + (Cin ⋅ (A ⊕ B)).
        let (s, c_out) = circuit.full_adder(a, b, c_in);
        let s = circuit.add_output(s);
        let c_out = circuit.add_output(c_out);

        let order = circuit.update_order();

        for a_ in [false, true].iter() {
            for b_ in [false, true].iter() {
                for c_in_ in [false, true].iter() {
                    let (a_, b_, c_in_) = (*a_, *b_, *c_in_);
                    circuit.set_input(a, a_);
                    circuit.set_input(b, b_);
                    circuit.set_input(c_in, c_in_);
                    for _ in 0..32 {
                        circuit.update_signals_once(&order);
                    }
                    assert_eq!(circuit.get_1_in(s), a_ ^ b_ ^ c_in_);
                    assert_eq!(circuit.get_1_in(c_out), (a_ & b_) | (c_in_ & (a_ ^ b_)));
                }
            }
        }
    }

    #[test]
    fn test_ripple_carry() {
        let mut circuit = Circuit::new();
        let n: usize = 4;
        let a = (0..n)
            .into_iter()
            .map(|_| circuit.add_input())
            .collect::<Vec<_>>();
        let b = (0..n)
            .into_iter()
            .map(|_| circuit.add_input())
            .collect::<Vec<_>>();
        let (s, c) = circuit.ripple_carry(&a, &b);
        let c = circuit.add_output(c);
        let s = s
            .into_iter()
            .map(|si| circuit.add_output(si))
            .collect::<Vec<_>>();

        let ranks = circuit.ranks();
        let steps = flip_ranks(&ranks).len() + 1;

        let order = circuit.update_order();

        for a_ in 0..(2usize).pow(n as u32) {
            for b_ in 0..(2usize).pow(n as u32) {
                for i in 0..n {
                    circuit.set_input(a[i], get_bit(a_, i));
                    circuit.set_input(b[i], get_bit(b_, i));
                }
                for _ in 0..steps {
                    circuit.update_signals_once(&order);
                }
                let (s_, _) = a_.overflowing_add(b_);
                let (s_) = ((s_ << (64 - n)) >> (64 - n));
                let mut s__ = 0;
                for i in 0..n {
                    s__ = set_bit(s__, i, circuit.get_1_in(s[i]));
                }
                //s__ = set_bit(s__, n, circuit.get_1_in(c));
                assert_eq!(
                    s__, s_,
                    "{:0b} + {:0b} = {:0b} [correct: {:0b}]",
                    a_, b_, s__, s_
                );
            }
        }
    }
}
