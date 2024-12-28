use std::{collections::HashMap, fs, thread::{self, JoinHandle}};

use crossbeam_channel::{unbounded, Receiver, Sender};
use wg_2024::{config::Config, drone::Drone, network::NodeId, packet::Packet};

pub trait Runnable: Send {
    fn run(&mut self);
}

impl<T: Drone + Send> Runnable for T {
    fn run(&mut self) {
        self.run();
    }
}

pub trait DroneCreator {
    fn create_drone(
        &mut self,
        id: NodeId,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        pdr: f32,
    ) -> Box<dyn Runnable>;
}

impl<F> DroneCreator for F
where
    F: FnMut(NodeId, Receiver<Packet>, HashMap<NodeId, Sender<Packet>>, f32) -> Box<dyn Runnable>,
{
    fn create_drone(
        &mut self,
        id: NodeId,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        pdr: f32,
    ) -> Box<dyn Runnable> {
        self(id, packet_recv, packet_send, pdr)
    }
}

pub trait ClientServerCreator {
    fn create_client_server(
        &mut self,
        id: NodeId,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Box<dyn Runnable>;
}

impl<F> ClientServerCreator for F
where
    F: FnMut(NodeId, Receiver<Packet>, HashMap<NodeId, Sender<Packet>>) -> Box<dyn Runnable>,
{
    fn create_client_server(
        &mut self,
        id: NodeId,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Box<dyn Runnable> {
        self(id, packet_recv, packet_send)
    }
}

pub fn create_topology_from_path(
    path: &str,
    drone_creator: impl DroneCreator,
    client_creator: impl ClientServerCreator,
    server_creator: impl ClientServerCreator,
) -> HashMap<NodeId, Box<dyn Runnable>> {
    let config = parse_topology_file(path);
    create_topology_from_config(&config, drone_creator, client_creator, server_creator)
}

pub fn parse_topology_file(path: &str) -> Config {
    let config_data = fs::read_to_string(path).expect("Unable to read config file");
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");
    config
}

pub fn create_topology_from_config(
    config: &Config,
    mut drone_creator: impl DroneCreator,
    mut client_creator: impl ClientServerCreator,
    mut server_creator: impl ClientServerCreator,
) -> HashMap<NodeId, Box<dyn Runnable>> {
    let (packet_senders, mut packet_receivers) = create_packet_channels(config);

    let mut nodes: HashMap<NodeId, Box<dyn Runnable>> = HashMap::new();

    for drone in config.drone.iter() {
        let packet_recv = packet_receivers.remove(&drone.id).unwrap();
        let packet_send = find_packet_send(&drone.connected_node_ids, &packet_senders);
        let pdr = drone.pdr;
        let runnable = drone_creator.create_drone(drone.id, packet_recv, packet_send, pdr);
        nodes.insert(drone.id, runnable);
    }

    for client in config.client.iter() {
        let packet_recv = packet_receivers.remove(&client.id).unwrap();
        let packet_send = find_packet_send(&client.connected_drone_ids, &packet_senders);
        let runnable = client_creator.create_client_server(client.id, packet_recv, packet_send);
        nodes.insert(client.id, runnable);
    }

    for server in config.server.iter() {
        let packet_recv = packet_receivers.remove(&server.id).unwrap();
        let packet_send = find_packet_send(&server.connected_drone_ids, &packet_senders);
        let runnable = server_creator.create_client_server(server.id, packet_recv, packet_send);
        nodes.insert(server.id, runnable);
    }

    nodes
}

pub fn create_packet_channels(
    config: &Config,
) -> (HashMap<NodeId, Sender<Packet>>, HashMap<NodeId, Receiver<Packet>>) {
    let mut packet_senders: HashMap<NodeId, Sender<Packet>> = HashMap::new();
    let mut packet_receivers: HashMap<NodeId, Receiver<Packet>> = HashMap::new();

    for drone in config.drone.iter() {
        let (snd, rcv) = unbounded();
        packet_receivers.insert(drone.id, rcv);
        packet_senders.insert(drone.id, snd);
    }

    for client in config.client.iter() {
        let (snd, rcv) = unbounded();
        packet_receivers.insert(client.id, rcv);
        packet_senders.insert(client.id, snd);
    }

    for server in config.server.iter() {
        let (snd, rcv) = unbounded();
        packet_receivers.insert(server.id, rcv);
        packet_senders.insert(server.id, snd);
    }

    (packet_senders, packet_receivers)
}

pub fn find_packet_send(
    connected_node_ids: &[NodeId],
    packet_senders: &HashMap<NodeId, Sender<Packet>>,
) -> HashMap<NodeId, Sender<Packet>> {
    let mut packet_send = HashMap::with_capacity(connected_node_ids.len());
    for neighbor_id in connected_node_ids.iter() {
        if let Some(snd) = packet_senders.get(neighbor_id) {
            packet_send.insert(*neighbor_id, snd.clone());
        }
    }
    packet_send
}

pub fn spawn_threads(nodes: HashMap<NodeId, Box<dyn Runnable>>) -> HashMap<NodeId, JoinHandle<()>> {
    let mut handles = HashMap::new();
    for (id, mut node) in nodes {
        let spawn = thread::spawn(move || {
            node.run();
        });
        let handle = spawn;
        handles.insert(id, handle);
    }
    handles
}

