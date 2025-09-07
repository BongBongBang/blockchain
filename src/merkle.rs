use std::mem;

use sha2::Digest;

pub struct MerkleTree {
    pub root: MerkleNode,
}
#[derive(Default)]
pub struct MerkleNode {
    pub left: Box<Option<MerkleNode>>,
    pub right: Box<Option<MerkleNode>>,
    pub data: Vec<u8>,
}

impl MerkleTree {
    pub fn new(mut datas: Vec<Vec<u8>>) -> Self {
        // 补齐偶数
        if datas.len() % 2 != 0 {
            datas.push(datas[datas.len() - 1].clone());
        }

        let mut nodes: Vec<MerkleNode> = Vec::new();

        for data in datas {
            let node = MerkleNode::new(None, None, Some(data));
            nodes.push(node);
        }

        while nodes.len() > 1 {
            let mut current_level_nodes = vec![];

            for i in (0..nodes.len() - 1).step_by(2) {
                let left = mem::take(&mut nodes[i]);
                let right = mem::take(&mut nodes[i + 1]);
                let node = MerkleNode::new(Some(left), Some(right), None);
                current_level_nodes.push(node);
            }
            nodes = current_level_nodes;
        }

        Self {
            root: mem::take(&mut nodes[0])
        }
    }
}

impl MerkleNode {
    pub fn new(left: Option<MerkleNode>, right: Option<MerkleNode>, data: Option<Vec<u8>>) -> Self {
        let mut data_hash = Vec::default();
        // leaf
        if left.is_none() && right.is_none() {
            data_hash.extend(sha2::Sha256::digest(data.unwrap()));
        } else {
            // non-leaf
            let mut merged_data = vec![];
            merged_data.extend_from_slice(&left.as_ref().unwrap().data);
            merged_data.extend_from_slice(&right.as_ref().unwrap().data);
            data_hash.extend(sha2::Sha256::digest(merged_data));
        }

        Self {
            left: Box::new(left),
            right: Box::new(right),
            data: data_hash,
        }
    }
}
