#![allow(dead_code)]
use std::collections::HashMap;
use std::fmt::Display;

use fixedbitset::FixedBitSet;
use petgraph::graph::NodeIndex;
use petgraph::visit::Dfs;
use petgraph::Graph;

#[derive(Clone, Debug)]
pub struct Address {
    node: NodeIndex,
    human: AddressPath,
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.human)
    }
}

/// internal
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum AddressPath {
    Root,
    Leaf(Vec<Segment>),
}

impl From<&AddressPath> for String {
    fn from(path: &AddressPath) -> Self {
        match path {
            AddressPath::Root => ".".into(),
            AddressPath::Leaf(segments) => segments
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("."),
        }
    }
}

impl Display for AddressPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressPath::Root =>  write!(f, "$"),
            AddressPath::Leaf(segments) => {
                write!(f, "$")?;
                for segment in segments {
                    write!(f, ".{}", segment)?;
                };
                Ok(())
            }
        }
    }
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

#[derive(Debug)]
pub struct DependencyTracking<T, N> {
    tracked_resources: HashMap<AddressPath, T>,
    dependency_graph: Graph<AddressPath, N>,
    own_address: Address,
}

impl<'a, T, N> DependencyTracking<T, N> {
    pub fn iter(&self) -> DependencyIterator<'_, T, N> {
        let root_address = AddressPath::Root;
        let root = self
            .dependency_graph
            .node_indices()
            .find(|i| self.dependency_graph[*i] == root_address)
            .unwrap();

        let mut dfs = Dfs::new(&self.dependency_graph, root);
        // Skip the root itself:
        let _ = dfs.next(&self.dependency_graph);

        DependencyIterator { deps: self, dfs }
    }
}

pub struct DependencyIterator<'a, T, N> {
    deps: &'a DependencyTracking<T, N>,
    dfs: Dfs<NodeIndex, FixedBitSet>,
}

impl<'a, T, N> Iterator for DependencyIterator<'a, T, N> {
    type Item = (&'a T, &'a AddressPath);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(visited) = self.dfs.next(&self.deps.dependency_graph) {
            let address = self.deps.dependency_graph.node_weight(visited).unwrap();

            self.deps
                .tracked_resources
                .get(address)
                .map(|resource| (resource, address))
        } else {
            None
        }
    }
}

impl<T, N> Default for DependencyTracking<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, N> DependencyTracking<T, N> {
    pub fn new() -> DependencyTracking<T, N> {
        let root = AddressPath::Root;

        let mut dependency_graph = Graph::new();
        let idx = dependency_graph.add_node(root.clone());

        let own_address = Address {
            node: idx,
            human: root,
        };

        DependencyTracking {
            tracked_resources: HashMap::new(),
            dependency_graph,
            own_address,
        }
    }

    pub fn swap_own_address(&mut self, mut other: Address) -> Address {
        std::mem::swap(&mut self.own_address, &mut other);
        other
    }

    pub fn child(
        &mut self,
        kind: impl Into<String>,
        name: impl Into<String>,
        value: T,
        edge: N,
    ) -> Address {
        let segment = Segment {
            name: name.into(),
            kind: kind.into(),
        };

        let new_address_path = self.own_address.human.extend_with(segment);
        self.tracked_resources
            .insert(new_address_path.clone(), value);
        let idx = self.dependency_graph.add_node(new_address_path.clone());

        self.dependency_graph
            .add_edge(self.own_address.node, idx, edge);

        Address {
            node: idx,
            human: new_address_path,
        }
    }

    pub fn add_dependency(&mut self, from: &Address, to: &Address, edge: N) {
        self.dependency_graph.add_edge(from.node, to.node, edge);
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
