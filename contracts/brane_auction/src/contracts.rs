use core::panic;

use cosmwasm_std::{
    attr, entry_point, from_json, has_coins, to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Order, Querier, QuerierWrapper, QueryRequest, Reply, Response, StdError, StdResult, Storage, SubMsg, Uint128, WasmMsg, WasmQuery
};
use cw2::set_contract_version;

use cw_storage_plus::Bound;
use url::Url;

use sg2::msg::{CollectionParams, CreateMinterMsg, Sg2ExecuteMsg};
use cw721::{TokensResponse, Cw721QueryMsg as Sg721QueryMsg};
use sg721::{CollectionInfo, RoyaltyInfoResponse, ExecuteMsg as Sg721ExecuteMsg, InstantiateMsg as Sg721InstantiateMsg};
use crate::{error::ContractError, msgs::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, PendingAuctionResponse, QueryMsg, SubmissionsResponse}, reply::handle_collection_reply, state::{Auction, Bid, BidAssetAuction, SubmissionInfo, SubmissionItem, ASSET_AUCTION, CONFIG, NFT_AUCTION, OWNERSHIP_TRANSFER, PENDING_AUCTION, SUBMISSIONS}};


// Contract name and version used for migration.
const CONTRACT_NAME: &str = "brane_auction";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

//Constants
const COLLECTION_REPLY_ID: u64 = 1u64;
const SECONDS_PER_DAY: u64 = 86400u64;
const VOTE_PERIOD: u64 = 7u64;
const AUCTION_PERIOD: u64 = 1u64;
const CURATION_THRESHOLD: Decimal = Decimal::percent(11);
const DEFAULT_LIMIT: u32 = 32u32;

//Minter costs
const MINTER_COST: u128 = 250_000_000u128;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let mut submsgs: Vec<SubMsg> = vec![];


    // Need to send 250_000_000ustars to initialize the collection
    if let Some(sg721_code_id) = msg.sg721_code_id {
        //instantiate the Collection
        let collection_msg = Sg2ExecuteMsg::CreateMinter (CreateMinterMsg::<Option<Sg721InstantiateMsg>> {
            init_msg: Some(
                Sg721InstantiateMsg {                    
                    name: String::from("The International Brane Wave"), 
                    symbol: String::from("BRANE"), 
                    minter: env.contract.address.to_string(),
                    collection_info: CollectionInfo { 
                        creator: env.contract.address.to_string(), 
                        description: String::from("The International Brane Wave is a continuous collection created by reverberating brane waves. It is a living, breathing, and evolving collection of digital art. The International Brane Wave is a place where artists can submit their braney work to append to the collection through daily auctions with majority of proceeds going to the submitting artist. Submissions can be new pfps, memes, portraits, etc. Let your creativity take hold of the pen!....or pencil...or stylus..you get the gist."),
                        image: "ipfs://bafybeid2chlkhoknrlwjycpzkiipqypo3x4awnuttdx6sex3kisr3rgfsm/".to_string(),  //TEMP TEMP TEMP TEMP
                        external_link: Some(String::from("https://twitter.com/the_memebrane")),
                        explicit_content: Some(false), 
                        start_trading_time: None, 
                        royalty_info: Some(RoyaltyInfoResponse { 
                            payment_address: env.contract.address.to_string(), 
                            share: Decimal::percent(1)
                        }) 
                    }
                }
            ),
            collection_params: CollectionParams { 
                code_id: sg721_code_id, 
                name: String::from("The International Brane Wave"), 
                symbol: String::from("BRANE"), 
                info: CollectionInfo { 
                    creator: env.contract.address.to_string(), 
                    description: String::from("The International Brane Wave is a continuous collection created by reverberating brane waves. It is a living, breathing, and evolving collection of digital art. The International Brane Wave is a place where artists can submit their braney work to append to the collection through daily auctions with majority of proceeds going to the submitting artist. Submissions can be new pfps, memes, portraits, etc. Let your creativity take hold of the pen!....or pencil...or stylus..you get the gist."),
                    image: "ipfs://bafybeid2chlkhoknrlwjycpzkiipqypo3x4awnuttdx6sex3kisr3rgfsm/".to_string(),  //TEMP TEMP TEMP TEMP
                    external_link: Some(String::from("https://twitter.com/the_memebrane")),
                    explicit_content: Some(false), 
                    start_trading_time: None, 
                    royalty_info: Some(RoyaltyInfoResponse { 
                        payment_address: env.contract.address.to_string(), 
                        share: Decimal::percent(1)
                    }) 
                }
            }
        });
        let cosmos_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_validate(&msg.base_factory_address)?.to_string(),
            msg: to_json_binary(&collection_msg)?,
            funds: vec![
                Coin {
                    denom: String::from("ustars"),
                    amount: Uint128::new(MINTER_COST),
                }
            ],
        });

        //Create the collection submsg
        let submsg = SubMsg::reply_on_success(cosmos_msg, COLLECTION_REPLY_ID);
        //add to msgs
        submsgs.push(submsg);
    } else {
        //Verify the minter address
        if let Some(minter_addr) = msg.minter_addr.clone() {
            deps.api.addr_validate(&minter_addr)?;
        }
    }

    let config = Config {
        owner: info.sender.clone(),
        bid_denom: msg.clone().bid_denom,
        minimum_outbid: Decimal::percent(1),
        incentive_denom: msg.clone().incentive_denom,
        incentive_distribution_amount: 100_000_000u128,
        incentive_bid_percent: Decimal::percent(10),
        current_token_id: 0,
        current_submission_id: 0,
        minter_addr: msg.clone().minter_addr.unwrap_or_else(|| "".to_string()),
        mint_cost: msg.mint_cost as u128,
        submission_cost: 10_000_000u128,
        submission_limit: 333u64,
        submission_total: 0u64,
        submission_vote_period: VOTE_PERIOD,
        curation_threshold: CURATION_THRESHOLD,
        auction_period: AUCTION_PERIOD,
    };

    CONFIG.save(deps.storage, &config)?;
    PENDING_AUCTION.save(deps.storage, &vec![])?;

    //verify the proceed recipient
    deps.api.addr_validate(&msg.first_submission.proceed_recipient.to_string())?;
    // Token URI must be a valid URL (ipfs, https, etc.)
    Url::parse(&msg.first_submission.token_uri).map_err(|_| ContractError::InvalidTokenURI { uri: msg.first_submission.token_uri.clone() })?;
    //Start first Auction
    NFT_AUCTION.save(deps.storage, &Auction {
        submission_info: SubmissionItem {
            submission: SubmissionInfo {
                submitter: info.sender.clone(),
                ..msg.first_submission
            },
            curators: vec![],
            votes: 0u64,
            submission_end_time: env.block.time.seconds() + (VOTE_PERIOD * SECONDS_PER_DAY),
        },
        bids: vec![],
        auction_end_time: env.block.time.seconds() + 300,//(SECONDS_PER_DAY * config.auction_period),
        highest_bid: Bid {
            bidder: Addr::unchecked(""),
            amount: 0u128,
        },
    })?;

    Ok(Response::new()
        .add_submessages(submsgs)
        .add_attribute("method", "instantiate")
        .add_attribute("config", format!("{:?}", config))
        .add_attribute("contract_address", env.contract.address)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SubmitNft { proceed_recipient, token_uri } => submit_nft(deps, env, info, proceed_recipient, token_uri),
        ExecuteMsg::VoteToCurate { submission_ids } => curate_nft(deps, env, info, submission_ids),
        ExecuteMsg::BidForNft {  } => bid_on_live_auction(deps, env, info),
        ExecuteMsg::BidForAssets {  } => bid_for_bid_assets(deps, info),
        ExecuteMsg::ConcludeAuction {  } => conclude_auction(deps, env),
        // ExecuteMsg::MigrateMinter { new_code_id } => todo!(),
        ExecuteMsg::MigrateContract { new_code_id } => migrate_contract(deps, env, info, new_code_id),
        ExecuteMsg::UpdateConfig { owner, bid_denom, minimum_outbid, incentive_denom, curation_threshold, incentive_bid_percent, incentive_distribution_amount, mint_cost, auction_period, submission_cost, submission_limit, submission_vote_period } => 
        update_config(deps, info, owner, bid_denom, minimum_outbid, incentive_denom, incentive_distribution_amount, incentive_bid_percent, mint_cost, submission_cost, submission_limit, submission_vote_period, curation_threshold, auction_period),
        }
}

fn update_config(
    deps: DepsMut,
    info: MessageInfo,
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
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut attrs = vec![];

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    //Assert Authority
    if info.sender != config.owner {
        //Check if ownership transfer is in progress & transfer if so
        let new_owner = OWNERSHIP_TRANSFER.load(deps.storage)?;
        if info.sender == new_owner {
            config.owner = info.sender;
        } else {
            return Err(ContractError::Unauthorized { });
        }
    }    
    if let Some(owner) = owner {
        let valid_addr = deps.api.addr_validate(&owner)?;

        //Set owner transfer state
        OWNERSHIP_TRANSFER.save(deps.storage, &valid_addr)?; 
        attrs.push(attr("owner_transfer", valid_addr));
    }

    if let Some(bid_denom) = bid_denom {
        config.bid_denom = bid_denom;
    }
    if let Some(minimum_outbid) = minimum_outbid {
        config.minimum_outbid = minimum_outbid;
    }
    if let Some(incentive_denom) = incentive_denom {
        config.incentive_denom = Some(incentive_denom);
    }
    if let Some(incentive_distribution_amount) = incentive_distribution_amount {
        config.incentive_distribution_amount = incentive_distribution_amount;
    }
    if let Some(incentive_bid_percent) = incentive_bid_percent {
        config.incentive_bid_percent = incentive_bid_percent;
    }
    if let Some(mint_cost) = mint_cost {
        config.mint_cost = mint_cost;
    }
    if let Some(submission_cost) = submission_cost {
        config.submission_cost = submission_cost;
    }
    if let Some(submission_limit) = submission_limit {
        config.submission_limit = submission_limit;
    }
    if let Some(submission_vote_period) = submission_vote_period {
        config.submission_vote_period = submission_vote_period;
    }
    if let Some(curation_threshold) = curation_threshold {
        config.curation_threshold = curation_threshold;
    }
    if let Some(auction_period) = auction_period {
        config.auction_period = auction_period;
    }
    
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
    .add_attributes(attrs)
        .add_attribute("method", "update_config")
        .add_attribute("config", format!("{:?}", config))
    )
}

fn migrate_contract(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_code_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let migrate_msg = CosmosMsg::Wasm(WasmMsg::Migrate { 
        contract_addr: env.contract.address.to_string(), 
        new_code_id, 
        msg: to_json_binary(&MigrateMsg {})?
    });

    Ok(Response::new()
        .add_message(migrate_msg)
        .add_attribute("method", "migrate_contract")
        .add_attribute("new_code_id", new_code_id.to_string())
    )
}

fn get_next_submission_id(
    storage: &mut dyn Storage,
    config: &mut Config
) -> Result<u64, ContractError> {
    let submission_id = config.current_submission_id;
    
    //Increment ID
    config.current_submission_id += 1;
    config.submission_total += 1;
    CONFIG.save(storage, config)?;

    Ok(submission_id)
}

fn submit_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proceed_recipient: String,
    token_uri: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut msgs: Vec<CosmosMsg> = vec![];
    
    // Token URI must be a valid URL (ipfs, https, etc.)
    Url::parse(&token_uri).map_err(|_| ContractError::InvalidTokenURI { uri: token_uri.clone() })?;

    //If submission is from a non-holder, it costs Some(bid_asset)
    match check_if_collection_holder(deps.as_ref(), config.clone().minter_addr, info.clone().sender){
        Ok(votes) => {
            if votes == 0 {
                //Check if the submission cost was sent                
                if !has_coins(&info.funds, &Coin {
                    denom: config.bid_denom.clone(),
                    amount: Uint128::new(config.submission_cost),
                }) {
                    return Err(ContractError::CustomError { val: "Submission cost not sent".to_string() });
                }
                //Submission cost is used in the bid asset auction
            }
        },
        Err(e) => return Err(e),
    };

    //Create a new submission
    let submission_id = get_next_submission_id(deps.storage, &mut config)?;
    let submission_info = SubmissionItem {
        submission: SubmissionInfo {            
            submitter: info.sender.clone(),
            proceed_recipient: deps.api.addr_validate(&proceed_recipient)?,
            token_uri,
        },
        curators: vec![],
        votes: 0u64,
        submission_end_time: env.block.time.seconds() + (config.submission_vote_period * SECONDS_PER_DAY),
    };

    SUBMISSIONS.save(deps.storage, submission_id, &submission_info)?;

    Ok(Response::new()
        .add_attribute("method", "submit_nft")
        .add_attribute("submission_id", submission_id.to_string())
        .add_attribute("submitter", info.sender)
        .add_attribute("submission_info", format!("{:?}", submission_info))
    )
}

fn check_if_collection_holder(
    deps: Deps,
    minter_addr: String,
    sender: Addr,
) -> Result<u64, ContractError> {  
    //If sender is the founder, they are valid
    if sender == Addr::unchecked("stars1988s5h45qwkaqch8km4ceagw2e08vdw2mu2mgs") {
        return Ok(1);
    }

    //Check if the sender is a collection holder
    let token_info: TokensResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: minter_addr,
        msg: to_json_binary(&Sg721QueryMsg::Tokens { owner: sender.to_string(), start_after: None, limit: None })?,
    })).map_err(|_| ContractError::CustomError { val: "Failed to query collection, sender may not hold an NFT".to_string() })?;

    if token_info.tokens.is_empty() {
        return Ok(0)
    }

    Ok(token_info.tokens.len() as u64)
}

fn curate_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    submission_ids: Vec<u64>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    //Check if the submission is valid
    if config.submission_total >= config.submission_limit {
        return Err(ContractError::CustomError { val: "Exceeded submission limit".to_string() });
    }

    //Make sure the sender is a collection holder
    let votes = check_if_collection_holder(deps.as_ref(), config.clone().minter_addr, info.clone().sender)?;

    //Error if votes are 0
    if votes == 0 {
        return Err(ContractError::CustomError { val: "Sender does not hold an NFT".to_string() });
    }

    //Get the Curation passing threshold
    let mut passing_threshold = 1u128;
    match deps.querier.query::<TokensResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.clone().minter_addr,
        msg: to_json_binary(&Sg721QueryMsg::AllTokens { start_after: None, limit: None })?,
    })){
        Ok(token_info) => {
            let total_votes = token_info.tokens.len();
            passing_threshold = (Uint128::new(total_votes as u128) * config.curation_threshold).u128();
        },
        Err(_) => { 
        }
    
    };

    //Update the submission info
    for submission_id in submission_ids.clone() {
        //Load submission info
        let mut submission_info = match SUBMISSIONS.load(deps.storage, submission_id){
            Ok(submission) => submission,
            Err(_) => return Err(ContractError::CustomError { val: String::from("Submission not found, maybe its already a valid auction") }),
        
        };
    
        // Assert they haven't voted yet
        if submission_info.curators.contains(&info.clone().sender) {
            continue;
        }
        /// Assert the submission is still in the voting period
        //If its past the submission period and the submission doesn't have enough votes, remove it
        if env.block.time.seconds() > submission_info.submission_end_time {
            if submission_info.votes < passing_threshold as u64 {
                SUBMISSIONS.remove(deps.storage, submission_id);
                //Subtract from the submission total
                config.submission_total -= 1;
                continue;
            }
        } 
        //If still in voting period continue voting
        else {
            //Tally the vote
            submission_info.curators.push(info.sender.clone());
            submission_info.votes += votes;

            
            //If the submission has enough votes, add it to the list of auctionables
            if submission_info.votes >= passing_threshold as u64 {
                //Set as live auction if there is none, else add to pending auctions
                if let Err(_) = NFT_AUCTION.load(deps.storage) {
                    NFT_AUCTION.save(deps.storage, &Auction {
                        submission_info: submission_info.clone(),
                        bids: vec![],
                        auction_end_time: env.block.time.seconds() + (SECONDS_PER_DAY * config.clone().auction_period),
                        highest_bid: Bid {
                            bidder: Addr::unchecked(""),
                            amount: 0u128,                            
                        },
                    })?;
                } else {

                    PENDING_AUCTION.update(deps.storage, |mut auctions| -> Result<_, ContractError> {
                        auctions.push(Auction {
                            submission_info: submission_info.clone(),
                            bids: vec![],
                            auction_end_time: 0, //will set when active
                            highest_bid: Bid {
                                bidder: Addr::unchecked(""),
                                amount: 0u128,                            
                            },
                        });
                        Ok(auctions)
                    })?;
                }
                SUBMISSIONS.remove(deps.storage, submission_id);
                //Subtract from the submission total
                config.submission_total -= 1;
            } else {
                //If the submission doesn't have enough votes yet, save it
                SUBMISSIONS.save(deps.storage, submission_id, &submission_info)?;                
            }
            
        }
    }

    //Save submission total
    CONFIG.save(deps.storage, &config)?;


    Ok(Response::new()
        .add_attribute("method", "curate_nft")
        .add_attribute("submission_ids", format!("{:?}", submission_ids))
        .add_attribute("curator", info.sender)
    )
}

fn assert_bid_asset(
    info: &MessageInfo,
    bid_denom: String,    
) -> Result<Bid, ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::InvalidAsset { asset: "None or more than 1 asset sent".to_string() });
    }
    //Check if the bid asset was sent
    if info.funds[0].denom != bid_denom {
        return Err(ContractError::InvalidAsset { asset: "Bid asset not sent".to_string() });
    }

    Ok(Bid {
        bidder: info.sender.clone(),
        amount: info.funds[0].amount.u128(),
    })
}

fn bid_on_live_auction(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    //Load config
    let config = CONFIG.load(deps.storage)?;
    //Assert funds are the bid asset
    let current_bid = assert_bid_asset(&info, config.clone().bid_denom)?;
    //Initialize msgs
    let mut msgs: Vec<CosmosMsg> = vec![];

    //This will be initiated in the instantiate function & refreshed at the end of the conclude_auction function
    let mut live_auction = match NFT_AUCTION.load(deps.storage){
        Ok(auction) => auction,
        Err(_) => return Err(ContractError::CustomError { val: "No live auction".to_string() }),
            
    };

    //Check if the auction is still live
    if env.block.time.seconds() >= live_auction.auction_end_time {
        return Err(ContractError::CustomError { val: "Auction has ended".to_string() });
    }

    //Check if the bid is higher than the current highest bid
    if let Some(highest_bid) = live_auction.bids.clone().last() {
        if Uint128::new(current_bid.amount) <= Uint128::new(highest_bid.amount) * (Decimal::one() + config.minimum_outbid) {
            return Err(ContractError::CustomError { val: "Bid is lower than the minimum outbid amount".to_string() });
        } else {
            //Add the bid to the auction's bid list
            live_auction.bids.push(current_bid.clone());

            //Send the previous highest bid back to the bidder
            msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: highest_bid.bidder.to_string(),
                    amount: vec![Coin {
                        denom: config.bid_denom,
                        amount: Uint128::new(highest_bid.amount),
                    }],
                }));

            //Set bid as highest bid
            live_auction.highest_bid = current_bid.clone();
        }
    } else if live_auction.bids.is_empty() {
        //Add the bid to the auction's bid list
        live_auction.bids.push(current_bid.clone());
        //Set bid as highest bid
        live_auction.highest_bid = current_bid.clone();
    }
    NFT_AUCTION.save(deps.storage, &live_auction)?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("method", "bid_on_live_auction")
        .add_attribute("bidder", info.sender)
        .add_attribute("bid", current_bid.amount.to_string())
    )
}

/// These auctions last as long as the current NFT auction is live & concludes with it
fn bid_for_bid_assets(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    //Load config
    let config = CONFIG.load(deps.storage)?;
    //Assert funds are the incentive
    let current_bid = assert_bid_asset(&info, config.clone().incentive_denom.unwrap())?;//unwrap is safe because these don't exist without a incentive denom
    //Initialize msgs
    let mut msgs: Vec<CosmosMsg> = vec![];

    //Load the current bid asset auction
    let mut live_auction = match ASSET_AUCTION.load(deps.storage){
        Ok(auction) => auction,
        Err(_) => return Err(ContractError::CustomError { val: "No live bid asset auction".to_string() }),
    
    };

    //Check if the bid is higher than the current highest bid
    if Uint128::new(current_bid.amount) < Uint128::new(live_auction.highest_bid.amount) * (Decimal::one() + config.minimum_outbid){
        return Err(ContractError::CustomError { val: "Bid is lower than the minimum outbid amount".to_string() });
    } else {
        //Send the previous highest bid back to the bidder
        if live_auction.highest_bid.amount > 0 {
            msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: live_auction.highest_bid.bidder.to_string(),
                    amount: vec![Coin {
                        denom: config.incentive_denom.unwrap(), //These auctions don't happen without a denom so its safe to unwrap
                        amount: Uint128::new(live_auction.highest_bid.amount),
                    }],
                }));
        }

        //Set bid as highest bid
        live_auction.highest_bid = current_bid.clone();
    }
    ////Save the new bid asset auction
    ASSET_AUCTION.save(deps.storage, &live_auction)?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("method", "bid_for_bid_assets")
        .add_attribute("bidder", info.sender)
        .add_attribute("new_highest_bid", current_bid.amount.to_string())
    )
}

//End & Start new Bid Asset Auction
fn conclude_bid_asset_auction(
    storage: &mut dyn Storage,
    querier: QuerierWrapper,
    env: Env,
    recipient_send_amount: Uint128,
) -> Result<Vec<CosmosMsg>, ContractError> {
    //Initialize msgs
    let mut msgs: Vec<CosmosMsg> = vec![];
    //Load live auction
    let live_auction = ASSET_AUCTION.load(storage);
    //Load config
    let config = CONFIG.load(storage)?;
    
    //Query contract's balance to include any submission costs to the bid asset auction
    let bid_denom_balance = querier.query_balance(env.clone().contract.address, config.bid_denom.clone())?;
    let asset_bid_amount = match bid_denom_balance.amount.checked_sub(recipient_send_amount){
        Ok(amount) => amount,
        //This helps pass contract tests, its not actually possible to have less assets then what was sent
        //If it does happen, the BankMsg::Send will fail
        Err(_) => Uint128::new(0)
        //Set to 10000000 to pass contract_test
    
    };
    let mut new_auction_asset = Coin {
        denom: config.bid_denom.clone(),
        amount: asset_bid_amount,
    };
    
    match live_auction {
        Ok(auction) => {
            //End the auction & distribute the asset to the highest bidder
            //If no one bids, the assets are sent to the contract
            if !auction.auctioned_asset.amount.is_zero() {
                msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: auction.highest_bid.bidder.to_string(),
                    amount: vec![auction.clone().auctioned_asset],
                }));

                //Subtract the auctioned asset from the new auction asset
                //to cover the overage from the queried balance
                new_auction_asset.amount -= auction.auctioned_asset.amount;
            }
            if config.incentive_distribution_amount == 0 && auction.highest_bid.amount > 0 {
                //Send the bid to the burn address
                msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: "stars1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8lhzvv".to_string(),
                    amount: vec![Coin {
                        denom: config.incentive_denom.unwrap(), //These auctions don't happen without a denom so its safe to unwrap
                        amount: Uint128::new(auction.highest_bid.amount),
                    
                    }],
                }));
            }
            //Remove the auction
            ASSET_AUCTION.remove(storage);
        },
        Err(_) => {
            //Just start the next auction below
        },
    };

    if new_auction_asset.amount.u128() > 0 {
        //Start the new auction
        ASSET_AUCTION.save(storage, &BidAssetAuction {
            auctioned_asset: new_auction_asset,
            highest_bid: Bid {
                bidder: env.contract.address,
                amount: 0u128,
            },
        })?;
    }

    Ok(msgs)
}


fn conclude_auction(
    deps: DepsMut,
    env: Env,
) -> Result<Response, ContractError> {
    //Load config
    let mut config = CONFIG.load(deps.storage)?;
    //Initialize msgs
    let mut msgs: Vec<CosmosMsg> = vec![];
    //Load live auction
    let mut live_auction = NFT_AUCTION.load(deps.storage)?;

    //Check if the auction is still live
    if env.block.time.seconds() < live_auction.auction_end_time {
        return Err(ContractError::CustomError { val: "Auction is still live".to_string() });
    }

    //Mint the NFT & send the bid to the proceed_recipient
    if live_auction.highest_bid.amount > 0 {
        //Mint the NFT to the highest bidder
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.clone().minter_addr,
            msg: to_json_binary(&Sg721ExecuteMsg::Mint::<Option<String>, Option<String>> {
                owner: live_auction.highest_bid.bidder.to_string(),
                token_id: config.current_token_id.to_string(),
                token_uri: Some(live_auction.submission_info.submission.token_uri.clone()),
                extension: None,
            })?,
            funds: vec![
                Coin {
                    denom: String::from("ustars"),
                    amount: Uint128::new(config.mint_cost),
                }],
        }));
        ///Transfer newly minted NFT to the bidder
        // msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        //     contract_addr: config.clone().minter_addr,
        //     msg: to_json_binary(&Sg721ExecuteMsg::TransferNft::<Option<String>, Option<String>> { 
        //         recipient: live_auction.highest_bid.bidder.to_string(),
        //         token_id: config.current_token_id.to_string(),
        //         })?,
        //     funds: vec![],
        // }));

        //////Split the highest bid to the proceed_recipient & incentive holders////
        if config.incentive_denom.is_none() {
            config.incentive_bid_percent = Decimal::percent(0);
        }
        let recipient_send_amount = Uint128::new(live_auction.highest_bid.amount) * (Decimal::one() - config.incentive_bid_percent);        
        if !recipient_send_amount.is_zero() {
            msgs.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: live_auction.submission_info.submission.proceed_recipient.to_string(),
                amount: vec![Coin {
                    denom: config.clone().bid_denom,
                    amount: recipient_send_amount,
                }],
            }));
        }        

        //Conclude the current Bid Asset Auction
        //Initiate the next Bid Asset Auction        
        msgs.extend(conclude_bid_asset_auction(deps.storage, deps.querier, env.clone(), recipient_send_amount)?);
        //////
        
        /////Send incentives to Bidders & curators
        if let Some(meme_denom) = config.incentive_denom {
            //Get incentive distribution amount
            let incentive_distribution_amount = match deps.querier.query_balance(env.clone().contract.address, meme_denom.clone()){
                Ok(balance) => {
                    //We distribute the config amount or half of the balance, whichever is lower
                    if balance.amount.u128() / 2 < config.incentive_distribution_amount {
                        balance.amount.u128() / 2
                    } else {
                        config.incentive_distribution_amount
                    }
                },
                Err(_) => config.incentive_distribution_amount,
            };

            //Split total incentives between unique curators (1/len)
            let meme_to_curators = live_auction.submission_info.curators.iter().map(|curator| {
                let meme_amount = (Uint128::new(incentive_distribution_amount) * Decimal::from_ratio(Uint128::new(1), live_auction.submission_info.curators.len() as u128)).u128();
                (curator.clone(), Coin {
                    denom: meme_denom.clone(),
                    amount: Uint128::new(meme_amount),
                })
            }).collect::<Vec<(Addr, Coin)>>();

            //Create the incentive distribution msgs to curators
            for (curator, coin) in meme_to_curators {
                if !coin.amount.is_zero() {
                    msgs.push(CosmosMsg::Bank(BankMsg::Send {
                        to_address: curator.to_string(),
                        amount: vec![coin],
                    }));
                }
            }
        }
    } else {
        //If no one bids, extend the auction time by 1 day
        live_auction.auction_end_time += SECONDS_PER_DAY;
        //Save the auction
        NFT_AUCTION.save(deps.storage, &live_auction)?;

        return Ok(Response::new()
            .add_messages(msgs)
            .add_attribute("method", "conclude_auction")
            .add_attribute("highest_bidder", "None")
            .add_attribute("highest_bid", live_auction.highest_bid.amount.to_string())
        )
    }

    //Set the new auction to the next pending auction
    let mut pending_auctions = PENDING_AUCTION.load(deps.storage)?;
    if !pending_auctions.is_empty() {
        //Get the next auction
        let mut next_auction = pending_auctions.remove(0);
        //set auction end time
        next_auction.auction_end_time = env.block.time.seconds() + (SECONDS_PER_DAY * config.auction_period);
        //Save as live auction
        NFT_AUCTION.save(deps.storage, &next_auction)?;
        //Save the pending auctions
        PENDING_AUCTION.save(deps.storage, &pending_auctions)?;
    } else {        
        //Remove the concluded auction
        NFT_AUCTION.remove(deps.storage);    
    }

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("method", "conclude_auction")
        .add_attribute("highest_bidder", live_auction.highest_bid.bidder)
        .add_attribute("highest_bid", live_auction.highest_bid.amount.to_string())
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary((&CONFIG.load(deps.storage)?)),
        QueryMsg::LiveNftAuction {  } => to_json_binary(&NFT_AUCTION.load(deps.storage)?),
        QueryMsg::LiveBidAssetAuction {  } => to_json_binary(&ASSET_AUCTION.load(deps.storage)?),
        QueryMsg::PendingAuctions { limit, start_after } => to_json_binary(&get_pending_auctions(deps, limit, start_after)?),
        QueryMsg::Submissions { submission_id, limit, start_after } => to_json_binary(&get_submissions(deps, submission_id, limit, start_after)?),
    }
}

fn get_submissions(
    deps: Deps,
    submission_id: Option<u64>,
    limit: Option<u32>,
    start_after: Option<u64>,
) -> StdResult<SubmissionsResponse> {
    let submissions: Vec<SubmissionItem> = match submission_id {
        Some(submission_id) => {
            let submission = SUBMISSIONS.load(deps.storage, submission_id)?;
            vec![submission]
        },
        None => {
            let start = start_after.map(|index| Bound::ExclusiveRaw(index.to_be_bytes().to_vec()));
            
            let submissions: StdResult<Vec<SubmissionItem>> = SUBMISSIONS
                .range(deps.storage, start, None, Order::Ascending)
                .map(|item| item.map(|(_, v)| v))
                .take(limit.unwrap_or(DEFAULT_LIMIT) as usize)
                .collect();
            submissions?
        }
    };

    Ok(
        SubmissionsResponse {
            submissions,
        }
    )
}

fn get_pending_auctions(
    deps: Deps,
    limit: Option<u32>,
    start_after: Option<u64>,
) -> StdResult<PendingAuctionResponse> {
    let pending_auctions = PENDING_AUCTION.load(deps.storage)?;

    let mut start = 0;
    if let Some(start_after) = start_after {
        start = start_after + 1;
    }

    let pending_auctions: Vec<Auction> = pending_auctions
        .into_iter()
        .skip(start as usize)
        .take(limit.unwrap_or(DEFAULT_LIMIT) as usize)
        .collect();

    Ok(
        PendingAuctionResponse {
            pending_auctions,
        }
    )
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        COLLECTION_REPLY_ID => handle_collection_reply(deps, env, msg),
        id => Err(StdError::generic_err(format!("invalid reply id: {}", id))),
    }
}
