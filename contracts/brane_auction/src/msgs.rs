use cosmwasm_std::{Addr, Decimal};
use cosmwasm_schema::cw_serde;

use crate::state::{Auction, SubmissionInfo, SubmissionItem};

#[cw_serde]
pub struct InstantiateMsg {
    ///Collection Code ID
    /// making this an option makes testing easier & allows pre-existing collections to be added if they give the contract mint ability
    pub sg721_code_id: Option<u64>, //testnet: 2595, mainnet: 180
    /// Minter address
    /// If you have an existing collection, pass the minter here to skip the instantiation
    pub minter_addr: Option<String>, 
    pub base_factory_address: String, //testnet: stars1a45hcxty3spnmm2f0papl8v4dk5ew29s4syhn4efte8u5haex99qlkrtnx, mainnet: stars1klnzgwfvca8dnjeasx00v00f49l6nplnvnsxyc080ph2h8qxe4wss4d3ga
    /// Bid denom
    pub bid_denom: String,
    /// Memecoin denom
    pub incentive_denom: Option<String>,
    /// First submission for the first NFT auction of the collection
    pub first_submission: SubmissionInfo,
    ///Mint cost
    /// testnet: 50_000_000u64
    /// mainnet: 5_000_000u64
    pub mint_cost: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    SubmitNft { 
        proceed_recipient: String,
        token_uri: String,
    },
    /// Submissions have 7 days to get votes, after 7 days any votes will delete the submission
    VoteToCurate { submission_ids: Vec<u64> },
    BidForNft { },
    BidForAssets { },
    /// Transfer NFT to highest bidder & handle incentive distributions
    ConcludeAuction { },
    ////These are all controlled by the owner who will be a DAODAO NFT staking contract
    // MigrateMinter { new_code_id: u64 },
    MigrateContract { new_code_id: u64 },
    UpdateConfig {
        owner: Option<String>,
        bid_denom: Option<String>,
        minimum_outbid: Option<Decimal>,
        incentive_denom: Option<String>,
        incentive_distribution_amount: Option<u128>,
        incentive_bid_percent: Option<Decimal>,
        mint_cost: Option<u128>,
        submission_cost: Option<u128>,
        submission_limit: Option<u64>,
        submission_vote_period: Option<u64>,
        curation_threshold: Option<Decimal>,
        auction_period: Option<u64>,
    },
    //////
}

#[cw_serde]
pub enum QueryMsg {
    /// Return contract config
    Config {},
    /// Return list of submissions
    Submissions { 
        submission_id: Option<u64>,
        limit: Option<u32>,
        start_after: Option<u64>
    },
    /// Return list of pending auctions
    PendingAuctions { 
        limit: Option<u32>,
        start_after: Option<u64>
    },
    /// Return live auction info
    LiveNFTAuction {},
    /// Return bid asset auction info
    LiveBidAssetAuction {},
}

#[cw_serde]
pub struct Config {
    /// Contract owner
    pub owner: Addr,
    /// Bid denom
    pub bid_denom: String,
    /// Minimum percent to increase bid by
    pub minimum_outbid: Decimal,
    /// Memecoin denom
    pub incentive_denom: Option<String>,
    /// Memecoin distribution amount
    pub incentive_distribution_amount: u128,
    /// Percent of Bid to distribute to incentive holders
    pub incentive_bid_percent: Decimal,
    /// Current token ID
    pub current_token_id: u64,
    /// Current submission ID
    pub current_submission_id: u64,
    /// Minter address
    pub minter_addr: String,
    /// Stargaze Mint cost 
    /// Testnet: 50_000_000u128
    /// Mainnet: 5_000_000_000u128
    pub mint_cost: u128,
    /// Submission cost for non-holders in the bid_denom
    pub submission_cost: u128,
    /// Submission limit
    pub submission_limit: u64,
    /// Current submission total
    pub submission_total: u64,
    /// Submission vote period (in days)
    pub submission_vote_period: u64,
    /// Curation threshold (i.e. % of Yes votes)
    pub curation_threshold: Decimal,
    /// Auction period (in days)
    pub auction_period: u64, 
}

#[cw_serde]
pub struct SubmissionsResponse {
    pub submissions: Vec<SubmissionItem>,
}

#[cw_serde]
pub struct PendingAuctionResponse {
    pub pending_auctions: Vec<Auction>,
}

#[cw_serde]
pub struct MigrateMsg {}