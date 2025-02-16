use secp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use crate::blockchain::Block;
use crate::blockchain::transaction::Transaction;
use crate::network::peers::Peer;

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkMessage {
    RequestBlockchain,
    SendBlockchain(Vec<Block>),
    RequestPeers,
    SendPeers(Vec<Peer>),
    NewTransaction(Transaction, PublicKey),
    MineNewBlock,
}
