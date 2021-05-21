use std::collections::HashMap;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub mod peer_finder {
  tonic::include_proto!("peerfinder");
}
use peer_finder::peerfinder_client::PeerfinderClient;
use peer_finder::peerfinder_server;
use peer_finder::{AnnounceReq, AnnounceResp};

#[derive(Debug, Default)]
pub struct NodeP2P {
  id: Uuid,
  // server: peerfinder_server::PeerfinderServer
  peers: HashMap<Uuid, PeerfinderClient<tonic::transport::Channel>>,
}

impl NodeP2P {
  pub fn new(id: Uuid) -> NodeP2P {
    NodeP2P {
      id: id,
      peers: HashMap::new(),
    }
  }
}

#[tonic::async_trait]
impl peerfinder_server::Peerfinder for NodeP2P {
  async fn announce(&self, _req: Request<AnnounceReq>) -> Result<Response<AnnounceResp>, Status> {
    let response = AnnounceResp {
      id: self.id.to_string().clone(),
      peers: vec![],
    };
    Ok(Response::new(response))
  }
}
