use std::str::FromStr;
use crate::{bitcoin::{Hash, MerkleTreeBitcoin, SingleBlock}, MtProof};

async fn ser_block_into_file(block: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://blockchain.info/rawblock/{block}");
    let resp = reqwest::get(url)
        .await?
        .json::<SingleBlock>()
        .await?;

    let json = serde_json::to_string_pretty(&resp)?;
    let path = format!("./src/bitcoin/test_files/block_{block}.json");
    std::fs::write(path, json)?;

    Ok(())
}

#[allow(unused)]
async fn pre_save() -> Result<(), Box<dyn std::error::Error>> {
    let blocks = [
        "000000000000000000018b116534521bcf05e65e73825ec24bea2450657f9307",
        "00000000000000000000308b748583d0301871e0c109123bb31b85573cd01ddd", // block #893_557
        "00000000000000000001cf409fb3adc143114d6aaaabb8899fef039fe6e1ca5f", // block #896_599
        "000000000000000000020a90c72e67d1165002c5cd319c0db57fd7b6e60c48ee", // block #896_000
        "00000000000000000000faae664d48708f4e974d18f61df3140ce5e537fe2bad", // block #896_001
    ];
    for block in blocks {
        ser_block_into_file(block).await?;
    }
    Ok(())
}

#[tokio::test]
async fn big_bitcoin_block_test() -> Result<(), Box<dyn std::error::Error>> {
    // just presaved json blocks; you can get them(with other names) by running `pre_save` fn
    let blocks = [
        "x",
        "893_557",
        "896_599",
        "896_000",
        "896_001",
    ];

    for block in blocks {
        let json_file = format!("./src/bitcoin/test_files/test_block_{block}.json");
        let file = std::fs::read(json_file)?;
        let single_block: SingleBlock = serde_json::from_slice(&file)?;
        
        let index_proof = 173;
        let hash_valid_proof = single_block.txs[index_proof].hash.clone();
        let hash_invalid_proof = single_block.txs[index_proof + 3].hash.clone();
        
        let iter = single_block.txs.into_iter().map(|tx|tx.hash);
        let tree = MerkleTreeBitcoin::new_by_leafs(iter);

        // test validness of merkle tree root
        assert_eq!(single_block.mrkl_root, tree.root());

        // test proof validity
        let proof = tree.proof_owned(crate::LeafId::new(index_proof));
        assert!(proof.verify(hash_valid_proof.clone(), &mut super::BitcoinHasher::new()));
        assert!(!proof.verify(hash_invalid_proof.clone(), &mut super::BitcoinHasher::new()));
        
        // test proof serde
        let proof = serde_json::to_string_pretty(&proof)?;
        let proof: MtProof<Hash, { MerkleTreeBitcoin::ARITY }> = serde_json::from_str(&proof)?;
        assert!(proof.verify(hash_valid_proof, &mut super::BitcoinHasher::new()));
        assert!(!proof.verify(hash_invalid_proof, &mut super::BitcoinHasher::new()));
        
        // test tree serialize / deserialize
        let tree_serde = serde_json::to_string(&tree)?;
        let tree_unser: MerkleTreeBitcoin = serde_json::from_str(&tree_serde)?;
        assert!(tree.eq_full(&tree_unser));
    }
    Ok(())
}

#[tokio::test]
async fn bitcoin_block_100000_test() -> Result<(), Box<dyn std::error::Error>> {
    // Bitcoin block #100_000

    let tx0 = "8c14f0db3df150123e6f3dbbf30f8b955a8249b62ac1d1ff16284aefa3d06d87";
    let tx1 = "fff2525b8931402dd09222c50775608f75787bd2b87e56995a7bdd30f79702c4";
    let tx2 = "6359f0868171b1d194cbee1af2f16ea598ae8fad666d9b012c8ed2b79a236ec4";
    let tx3 = "e9a66845e05d5abc0ad04ec80f774a7e585c6e8db975962d069a522137b80c1d";

    let iter = vec![tx0, tx1, tx2, tx3];
    let iter = iter.into_iter().map(|tx|Hash::from_str(tx).unwrap());
    let tree = MerkleTreeBitcoin::new_by_leafs(iter);

    let expected_root: Hash = "f3e94742aca4b5ef85488dc37c06c3282295ffec960994b2c0d5ac2a25a95766".parse()?;
    assert_eq!(expected_root, tree.root());
    Ok(())
}

#[test]
fn bitcoin_hash_test() {
    let hash_str = "abcdef01234567899abcdef012345678567abcdef01234567899abcdef012348";
    let hash = Hash::from_str(&hash_str).unwrap();
    let hash_str_from_hash = hash.to_string();
    assert_eq!(hash_str, hash_str_from_hash);
}
