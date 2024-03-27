use cosmwasm_std::{
    entry_point, has_coins, to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, 
    QueryRequest, Reply, Response, StdError, StdResult, Storage, SubMsg, Uint128, WasmMsg, WasmQuery, attr
};
use cw2::set_contract_version;

use url::Url;

use sg2::msg::{CollectionParams, CreateMinterMsg, Sg2ExecuteMsg};
use cw721::{TokensResponse, Cw721QueryMsg as Sg721QueryMsg};
use sg721::{CollectionInfo, RoyaltyInfoResponse, ExecuteMsg as Sg721ExecuteMsg};
use crate::{error::ContractError, msgs::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg}, reply::handle_collection_reply, state::{Auction, Bid, BidAssetAuction, SubmissionInfo, SubmissionItem, ASSET_AUCTION, CONFIG, NFT_AUCTION, PENDING_AUCTION, SUBMISSIONS, OWNERSHIP_TRANSFER}};


// Contract name and version used for migration.
const CONTRACT_NAME: &str = "brane_auction";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

//Constants
const COLLECTION_REPLY_ID: u64 = 1u64;
const SECONDS_PER_DAY: u64 = 86400u64;
const VOTE_PERIOD: u64 = 7u64;
const AUCTION_PERIOD: u64 = 1u64;
const CURATION_THRESHOLD: Decimal = Decimal::percent(11);

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

    //instantiate the Collection
    let collection_msg = Sg2ExecuteMsg::CreateMinter (CreateMinterMsg::<Option<String>> {
        init_msg: None,
        collection_params: CollectionParams { 
            code_id: msg.sg721_code_id, 
            name: String::from("The International Brane Wave"), 
            symbol: String::from("BRANE"), 
            info: CollectionInfo { 
                creator: String::from("Reverberating Brane Waves"), 
                description: String::from("The International Brane Wave is a continuous collection created by reverberating brane waves. It is a living, breathing, and evolving collection of digital art. The International Brane Wave is a place where artists can submit their braney work to append to the collection through daily auctions with majority of proceeds going to the submitting artist. Submissions can be new pfps, memes, portraits, etc. Let your creativity take hold of the pen!....or pencil...or stylus..you get the gist."),
                image: todo!(), //"ipfs://CREATE AN IPFS LINK".to_string(), 
                external_link: Some(String::from("https://twitter.com/insneinthebrane")),
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

    let config = Config {
        owner: info.sender.clone(),
        bid_denom: msg.bid_denom,
        minimum_outbid: Decimal::percent(1),
        memecoin_denom: msg.memecoin_denom,
        memecoin_distribution_amount: 100_000_000u128,
        memecoin_bid_percent: Decimal::percent(10),
        current_token_id: 0,
        current_submission_id: 0,
        minter_addr: "".to_string(),
        mint_cost: msg.mint_cost,
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
            curation_votes: vec![],
            submission_end_time: env.block.time.seconds() + (VOTE_PERIOD * SECONDS_PER_DAY),
        },
        bids: vec![],
        auction_end_time: env.block.time.seconds() + (SECONDS_PER_DAY * config.auction_period),
        highest_bid: Bid {
            bidder: Addr::unchecked(""),
            amount: 0u128,
        },
    })?;

    Ok(Response::new()
        .add_submessage(submsg)
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
        ExecuteMsg::SubmitNFT { proceed_recipient, token_uri } => submit_nft(deps, env, info, proceed_recipient, token_uri),
        ExecuteMsg::VoteToCurate { submission_ids, vote } => curate_nft(deps, env, info, submission_ids, vote),
        ExecuteMsg::BidForNFT {  } => bid_on_live_auction(deps, env, info),
        ExecuteMsg::BidForAssets {  } => bid_for_bid_assets(deps, info),
        ExecuteMsg::ConcludeAuction {  } => conclude_auction(deps, env),
        // ExecuteMsg::MigrateMinter { new_code_id } => todo!(),
        ExecuteMsg::MigrateContract { new_code_id } => migrate_contract(deps, env, info, new_code_id),
        ExecuteMsg::UpdateConfig { owner, bid_denom, minimum_outbid, memecoin_denom, curation_threshold, memecoin_bid_percent, memecoin_distribution_amount, mint_cost, auction_period, submission_cost, submission_limit, submission_vote_period } => 
        update_config(deps, info, owner, bid_denom, minimum_outbid, memecoin_denom, memecoin_distribution_amount, memecoin_bid_percent, mint_cost, submission_cost, submission_limit, submission_vote_period, curation_threshold, auction_period),
        }
}

fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    bid_denom: Option<String>,
    minimum_outbid: Option<Decimal>,
    memecoin_denom: Option<String>,
    memecoin_distribution_amount: Option<u128>,
    memecoin_bid_percent: Option<Decimal>,
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
    if let Some(memecoin_denom) = memecoin_denom {
        config.memecoin_denom = Some(memecoin_denom);
    }
    if let Some(memecoin_distribution_amount) = memecoin_distribution_amount {
        config.memecoin_distribution_amount = memecoin_distribution_amount;
    }
    if let Some(memecoin_bid_percent) = memecoin_bid_percent {
        config.memecoin_bid_percent = memecoin_bid_percent;
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
    if let Err(_) = check_if_collection_holder(deps.as_ref(), config.clone().minter_addr, info.clone().sender){
        if !has_coins(&info.funds, &Coin {
            denom: config.bid_denom.clone(),
            amount: Uint128::new(config.submission_cost),
        }) {
            return Err(ContractError::CustomError { val: "Submission cost not sent".to_string() });
        }

        //Send the bid asset to the owner
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.owner.to_string(),
            amount: vec![Coin {
                denom: config.bid_denom.clone(),
                amount: Uint128::new(config.submission_cost),
            }],
        }));
    };

    //Create a new submission
    let submission_id = get_next_submission_id(deps.storage, &mut config)?;
    let submission_info = SubmissionItem {
        submission: SubmissionInfo {            
            submitter: info.sender.clone(),
            proceed_recipient: deps.api.addr_validate(&proceed_recipient)?,
            token_uri,
        },
        curation_votes: vec![],
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
) -> Result<(), ContractError> {  

    //Check if the sender is a collection holder
    let token_info: TokensResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: minter_addr,
        msg: to_json_binary(&Sg721QueryMsg::Tokens { owner: sender.to_string(), start_after: None, limit: None })?,
    })).map_err(|_| ContractError::CustomError { val: "Failed to query collection, sender may not hold an NFT".to_string() })?;

    if token_info.tokens.is_empty() {
        return Err(ContractError::CustomError { val: "Sender does not hold an NFT in the collection".to_string() });
    }

    Ok(())
}

fn curate_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    submission_ids: Vec<u64>,
    vote: bool,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    //Check if the submission is valid
    if config.submission_total >= config.submission_limit {
        return Err(ContractError::CustomError { val: "Exceeded submission limit".to_string() });
    }

    //Make sure the sender is a collection holder
    check_if_collection_holder(deps.as_ref(), config.clone().minter_addr, info.clone().sender)?;


    //Get total votes
    let all_token_info: TokensResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.clone().minter_addr,
        msg: to_json_binary(&Sg721QueryMsg::AllTokens { start_after: None, limit: None })?,
    }))?;
    let total_votes = all_token_info.tokens.len();
    let passing_threshold = (Uint128::new(total_votes as u128) * config.curation_threshold).u128();

    //Update the submission info
    for submission_id in submission_ids.clone() {
        //Load submission info
        let mut submission_info = SUBMISSIONS.load(deps.storage, submission_id)?;
    
        // Assert they haven't voted yet
        if submission_info.curation_votes.contains(&info.clone().sender) {
            continue;
        }
        /// Assert the submission is still in the voting period
        //If its past the submission period and the submission doesn't have enough votes, remove it
        if env.block.time.seconds() > submission_info.submission_end_time {
            if submission_info.curation_votes.len() < passing_threshold as usize {
                SUBMISSIONS.remove(deps.storage, submission_id);
                //Subtract from the submission total
                config.submission_total -= 1;
                continue;
            }
        } 
        //If still in voting period continue voting
        else {            
            //Tally the vote
            if vote {
                submission_info.curation_votes.push(info.sender.clone());
                
                //If the submission has enough votes, add it to the list of auctionables
                if submission_info.curation_votes.len() < passing_threshold as usize {
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
    }

    //Save submission total
    CONFIG.save(deps.storage, &config)?;


    Ok(Response::new()
        .add_attribute("method", "curate_nft")
        .add_attribute("submission_ids", format!("{:?}", submission_ids))
        .add_attribute("curator", info.sender)
        .add_attribute("vote", vote.to_string())
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
    let mut live_auction = NFT_AUCTION.load(deps.storage)?;

    //Check if the auction is still live
    if env.block.time.seconds() > live_auction.auction_end_time {
        return Err(ContractError::CustomError { val: "Auction has ended".to_string() });
    }

    //Check if the bid is higher than the current highest bid
    if let Some(highest_bid) = live_auction.bids.clone().last() {
        if Uint128::new(current_bid.amount) <= Uint128::new(highest_bid.amount) * config.minimum_outbid {
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
    //Assert funds are the memecoin
    let current_bid = assert_bid_asset(&info, config.memecoin_denom.unwrap())?;//unwrap is safe because these don't exist without a memecoin denom
    //Initialize msgs
    let mut msgs: Vec<CosmosMsg> = vec![];

    //Load the current bid asset auction
    let mut live_auction = ASSET_AUCTION.load(deps.storage)?;

    //Check if the bid is higher than the current highest bid
    if Uint128::new(current_bid.amount) <= Uint128::new(live_auction.highest_bid.amount) * config.minimum_outbid {
        return Err(ContractError::CustomError { val: "Bid is lower than the minimum outbid amount".to_string() });
    } else {
        //Send the previous highest bid back to the bidder
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: live_auction.highest_bid.bidder.to_string(),
                amount: vec![Coin {
                    denom: config.bid_denom,
                    amount: Uint128::new(live_auction.highest_bid.amount),
                }],
            }));

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

fn get_bid_ratios(
    bids: &Vec<Bid>
) -> Vec<(Addr, Decimal)> {
    let mut bid_ratios: Vec<(Addr, Decimal)> = vec![];
    let total_bids = bids.iter().fold(0u128, |acc, bid| acc + bid.amount);

    //Aggregate bids of the same bidder
    let bids = bids.iter().fold(vec![], |mut acc, bid| {
        if let Some(bid_index) = acc.iter().position(|x: &Bid| x.bidder == bid.bidder) {
            acc[bid_index].amount += bid.amount;
        } else {
            acc.push(bid.clone());
        }
        acc
    });

    //Get ratios
    for bid in bids {
        bid_ratios.push((bid.bidder.clone(), Decimal::from_ratio(bid.amount, total_bids)));
    }

    bid_ratios
}

//End & Start new Bid Asset Auction
fn conclude_bid_asset_auction(
    storage: &mut dyn Storage,
    env: Env,
    new_auction_asset: Coin,
) -> Result<Vec<CosmosMsg>, ContractError> {
    //Initialize msgs
    let mut msgs: Vec<CosmosMsg> = vec![];
    //Load live auction
    let live_auction = ASSET_AUCTION.load(storage);
    //Load config
    let config = CONFIG.load(storage)?;
    
    match live_auction {
        Ok(auction) => {
            //End the auction & distribute the asset to the highest bidder
            //If no one bids, the assets are sent to the contract
            msgs.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: auction.highest_bid.bidder.to_string(),
                amount: vec![auction.auctioned_asset],
            }));
            
            //Send the bid to the burn address
            msgs.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: "stars1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8lhzvv".to_string(),
                amount: vec![Coin {
                    denom: config.memecoin_denom.unwrap(), //These auctions don't happen without a denom so its safe to unwrap
                    amount: Uint128::new(auction.highest_bid.amount),
                
                }],
            }));
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

        //////Split the highest bid to the proceed_recipient & memecoin holders////
        if config.memecoin_denom.is_none() {
            config.memecoin_bid_percent = Decimal::percent(0);
        }
        let meme_bid_amount = config.memecoin_bid_percent * Uint128::new(live_auction.highest_bid.amount);
        let recipient_send_amount = live_auction.highest_bid.amount - meme_bid_amount.u128();
        if recipient_send_amount > 0 {
            msgs.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: live_auction.submission_info.submission.proceed_recipient.to_string(),
                amount: vec![Coin {
                    denom: config.clone().bid_denom,
                    amount: Uint128::new(recipient_send_amount),
                }],
            }));
        }

        /////Send memecoins to Bidders & curators
        if let Some(meme_denom) = config.memecoin_denom {
            //Get memecoin distribution amount
            let memecoin_distribution_amount = match deps.querier.query_balance(env.clone().contract.address, meme_denom.clone()){
                Ok(balance) => {
                    //We distribute the config amount or half of the balance, whichever is lower
                    if balance.amount.u128() / 2 < config.memecoin_distribution_amount {
                        balance.amount.u128() / 2
                    } else {
                        config.memecoin_distribution_amount
                    }
                },
                Err(_) => config.memecoin_distribution_amount,
            };

            //Get bidder pro_rata distribution
            let bid_ratios = get_bid_ratios(&live_auction.bids);            
            //Split total memecoins between bidders (pro_rata to bid_amount)
            let meme_to_bidders = bid_ratios.iter().map(|bidder| {
                let meme_amount = (Uint128::new(memecoin_distribution_amount) * bidder.1).u128();
                (bidder.0.clone(), Coin {
                    denom: meme_denom.clone(),
                    amount: Uint128::new(meme_amount),
                })
            }).collect::<Vec<(Addr, Coin)>>();
            //Split total memecoins between curators (1/len)
            let meme_to_curators = live_auction.submission_info.curation_votes.iter().map(|curator| {
                let meme_amount = (Uint128::new(memecoin_distribution_amount) * Decimal::from_ratio(Uint128::new(1), live_auction.submission_info.curation_votes.len() as u128)).u128();
                (curator.clone(), Coin {
                    denom: meme_denom.clone(),
                    amount: Uint128::new(meme_amount),
                })
            }).collect::<Vec<(Addr, Coin)>>();

            //Create the memecoin distribution msgs
            for (bidder, coin) in meme_to_bidders {
                msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: bidder.to_string(),
                    amount: vec![coin],
                }));
            }
            for (curator, coin) in meme_to_curators {
                msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: curator.to_string(),
                    amount: vec![coin],
                }));
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

    //Conclude the current Bid Asset Auction
    //Initiate the next Bid Asset Auction
    let new_auction_asset = Coin {
        denom: config.bid_denom.clone(),
        amount: Uint128::new(live_auction.highest_bid.amount),
    };
    msgs.extend(conclude_bid_asset_auction(deps.storage, env.clone(), new_auction_asset)?);

    //Set the new auction to the next pending auction
    let mut pending_auctions = PENDING_AUCTION.load(deps.storage)?;
    if let Some(mut next_auction) = pending_auctions.pop() {
        //set auction end time
        next_auction.auction_end_time = env.block.time.seconds() + (SECONDS_PER_DAY * config.auction_period);
        //Save as live auction
        NFT_AUCTION.save(deps.storage, &next_auction)?;
    }
    //Save the pending auctions
    PENDING_AUCTION.save(deps.storage, &pending_auctions)?;
    //Remove the concluded auction
    NFT_AUCTION.remove(deps.storage);    

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("method", "conclude_auction")
        .add_attribute("highest_bidder", live_auction.highest_bid.bidder)
        .add_attribute("highest_bid", live_auction.highest_bid.amount.to_string())
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        COLLECTION_REPLY_ID => handle_collection_reply(deps, env, msg),
        id => Err(StdError::generic_err(format!("invalid reply id: {}", id))),
    }
}
