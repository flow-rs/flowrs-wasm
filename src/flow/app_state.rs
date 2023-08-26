use std::{
    collections::HashMap,
    sync::{mpsc::channel, Arc, Mutex},
    time::Duration,
};

use anyhow::Error;

use flowrs::{
    connection::{connect, Input, Output, RuntimeNode},
    exec::{
        execution::{Executor, StandardExecutor},
        node_updater::SingleThreadedNodeUpdater,
    },
    flow_impl::Flow,
    node::{ChangeObserver, Context, InitError},
    sched::round_robin::RoundRobinScheduler,
    version::Version,
};
use flowrs_img::transform::DecodeImageNode;
use flowrs_std::{
    add::AddNode,
    debug::DebugNode,
    value::{ValueNode, ValueUpdateNode},
    vec::{CountWindow, MergeToVecNode},
};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

use super::dynamic_flow::FlowType;

pub struct AppState {
    // For a yet TBD reason a HashMap of dyn types looses track of channel pointers.
    // As a workaround Nodes are resolved in a two step process and stored in a Vec.
    pub nodes: Vec<Arc<Mutex<dyn RuntimeNode + Send>>>,
    pub node_idc: HashMap<String, usize>,
    pub context: Context,
    pub change_observer: ChangeObserver,
    pub threads: usize,
    pub max_duration: Duration,
}

impl AppState {
    pub fn new(threads: usize, max_duration: Duration) -> AppState {
        AppState {
            nodes: Vec::new(),
            node_idc: HashMap::new(),
            context: Context::new(),
            change_observer: ChangeObserver::new(),
            threads,
            max_duration,
        }
    }

    pub fn add_node(
        &mut self,
        name: &str,
        kind: String,
        props: Value,
    ) -> Result<String, InitError> {
        let node: Arc<Mutex<dyn RuntimeNode + Send>> = match kind.as_str() {
            "std.binops.add" => Arc::new(Mutex::new(AddNode::<FlowType, FlowType, FlowType>::new(
                Some(&self.change_observer),
            ))),
            "std.value" => Arc::new(Mutex::new(ValueNode::new(
                FlowType(Arc::new(props)),
                Some(&self.change_observer),
            ))),
            "std.valueupdate" => Arc::new(Mutex::new(ValueUpdateNode::new(
                FlowType(Arc::new(props.as_u64().unwrap() as u8)),
                Some(&self.change_observer),
            ))),
            "std.debug" => Arc::new(Mutex::new(DebugNode::<FlowType>::new(Some(
                &self.change_observer,
            )))),
            "img.decode" => Arc::new(Mutex::new(DecodeImageNode::new(Some(
                &self.change_observer,
            )))),
            "std.transform.vec" => Arc::new(Mutex::new(MergeToVecNode::<
                FlowType,
                FlowType,
                CountWindow<FlowType>,
            >::new(
                Some(&self.change_observer),
                CountWindow::<FlowType>::new(props.as_u64().unwrap() as usize),
            ))),
            _ => return Err(InitError::Other(Error::msg("Nodetype not yet supported"))),
        };
        self.nodes.push(node);
        self.node_idc.insert(name.to_owned(), self.nodes.len() - 1);
        Ok(name.to_owned())
    }

    pub fn connect_at(
        &mut self,
        lhs: String,
        rhs: String,
        index_in: usize,
        index_out: usize,
    ) -> Result<(), Error> {
        let lhs_idx = self.node_idc.get(&lhs).unwrap().clone();
        let rhs_idx = self.node_idc.get(&rhs).unwrap().clone();
        // TODO: RefCell is not an ideal solution here.
        let out_edge = self.nodes[lhs_idx]
            .lock()
            .unwrap()
            .output_at(index_out)
            .downcast_ref::<Output<FlowType>>()
            .expect(&format!(
                "{} Nodes output at {} couldn't be downcasted",
                lhs, index_in
            ))
            .clone();
        let in_edge = self.nodes[rhs_idx]
            .lock()
            .unwrap()
            .input_at(index_in)
            .downcast_ref::<Input<FlowType>>()
            .unwrap()
            .to_owned();
        connect(out_edge, in_edge);
        Ok(())
    }

    pub fn run(self) -> Result<(), Error> {
        let node_map: HashMap<u128, Arc<Mutex<(dyn RuntimeNode + Send + 'static)>>> = self
            .nodes
            .into_iter()
            .enumerate()
            .map(|n| (n.0 as u128, n.1))
            .collect();
        let flow = Flow::new("wasm", Version::new(0, 0, 1), node_map);
        let node_updater = SingleThreadedNodeUpdater::new(None);
        let mut executor = StandardExecutor::new(self.change_observer);
        let scheduler = RoundRobinScheduler::new();
        executor.run(flow, scheduler, node_updater)?;
        Ok(())
    }
}

#[derive(Deserialize)]
struct JsonNode {
    name: String,
    kind: String,
    props: Value,
}

#[derive(Deserialize)]
struct JsonConnection {
    node: String,
    index: usize,
}

#[derive(Deserialize)]
struct JsonEdge {
    source: JsonConnection,
    dest: JsonConnection,
}

#[derive(Deserialize)]
struct JsonData {
    threads: usize,
    duration: u64,
    nodes: Vec<JsonNode>,
    edges: Vec<JsonEdge>,
}

impl<'de> Deserialize<'de> for AppState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json_data: JsonData = JsonData::deserialize(deserializer)?;
        let mut app_state =
            AppState::new(json_data.threads, Duration::from_secs(json_data.duration));
        json_data.nodes.iter().for_each(|node| {
            let _ = app_state.add_node(&node.name, node.kind.clone(), node.props.to_owned());
        });
        json_data.edges.iter().for_each(|edge| {
            let _ = app_state.connect_at(
                edge.source.node.clone(),
                edge.dest.node.clone(),
                edge.dest.index,
                edge.source.index,
            );
        });

        Ok(app_state)
    }
}
