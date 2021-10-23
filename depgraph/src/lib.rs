#![allow(dead_code)]
use std::collections::HashMap;

use petgraph::graph::NodeIndex;
use petgraph::Graph;

/// Public, I believe...
#[derive(Clone)]
pub struct Address {
    node: NodeIndex,
    human: AddressPath,
}

/// internal
#[derive(Clone, Hash, Eq, PartialEq)]
enum AddressPath {
    Root,
    Leaf(Vec<Segment>),
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Segment {
    pub name: String,
    pub kind: String,
}

impl std::fmt::Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.kind, self.name)
    }
}

impl std::fmt::Debug for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.kind, self.name)
    }
}

struct ThingWithoutName<T, N> {
    tracked_resources: HashMap<AddressPath, T>,
    dependency_graph: Graph<AddressPath, N>,
    own_address: Address,
}

impl<T, N> ThingWithoutName<T, N> {
    pub fn new() -> ThingWithoutName<T, N> {
        let root = AddressPath::Root;

        let mut dependency_graph = Graph::new();
        let idx = dependency_graph.add_node(root.clone());

        let own_address = Address {
            node: idx,
            human: root,
        };

        ThingWithoutName {
            tracked_resources: HashMap::new(),
            dependency_graph,
            own_address,
        }
    }

    pub fn child(&mut self, kind: impl Into<String>, name: impl Into<String>, value: T) -> Address {
        let segment = Segment {
            name: name.into(),
            kind: kind.into(),
        };

        let new_address_path = self.own_address.human.extend_with(segment);
        self.tracked_resources
            .insert(new_address_path.clone(), value);
        let idx = self.dependency_graph.add_node(new_address_path.clone());

        Address {
            node: idx,
            human: new_address_path,
        }
    }
}
impl AddressPath {
    fn extend_with(&self, segment: Segment) -> AddressPath {
        match self {
            AddressPath::Root => AddressPath::Leaf(vec![segment]),
            AddressPath::Leaf(segments) => {
                let mut segments = segments.clone();
                segments.push(segment);
                AddressPath::Leaf(segments)
            }
        }
    }
}
