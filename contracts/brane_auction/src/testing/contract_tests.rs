
#[cfg(test)]
mod tests {
    use crate::contracts::{query, instantiate, execute};
    use crate::msgs::{Config, ExecuteMsg, InstantiateMsg, PendingAuctionResponse, QueryMsg, SubmissionsResponse};
    use crate::state::{Auction, Bid, BidAssetAuction, SubmissionInfo, SubmissionItem};

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, from_json, Addr, Coin, CosmosMsg, Decimal, SubMsg, Uint128, WasmMsg};

    #[test]
    fn submit_nft(){
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            sg721_code_id: None,
            minter_addr: Some(String::from("some_minter_address")),
            base_factory_address: String::from("stars1a45hcxty3spnmm2f0papl8v4dk5ew29s4syhn4efte8u5haex99qlkrtnx"),
            bid_denom: String::from("cdt"),
            incentive_denom: Some(String::from("meme")),
            first_submission: SubmissionInfo {
                submitter: Addr::unchecked(""),
                proceed_recipient: Addr::unchecked("proceed_recipient0000"),
                token_uri: String::from("ipfs://imageFolderCID/1.png"),
            },
            mint_cost: 1000u128,
        };
        //Instantiating contract
        let v_info = mock_info("sender88", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), v_info, msg).unwrap();

        //Query live auction
        let query_msg = QueryMsg::LiveNFTAuction { };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        let resp: Auction = from_json(&res).unwrap();
        assert_eq!(resp, Auction {
            submission_info: SubmissionItem {
                submission: SubmissionInfo {
                    submitter: Addr::unchecked("sender88"),
                    proceed_recipient: Addr::unchecked("proceed_recipient0000"),
                    token_uri: String::from("ipfs://imageFolderCID/1.png"),
                },
                curators: vec![],
                votes: 0u64,
                submission_end_time: 1572402219,
            },
            bids: vec![],
            highest_bid: Bid {
                bidder: Addr::unchecked(""),
                amount: 0u128,
            },
            auction_end_time: 1571883819,
        
        } );


        //Submit NFT
        let submit_msg = ExecuteMsg::SubmitNft {
            proceed_recipient: String::from("proceed_recipient0000"),
            token_uri: String::from("ipfs://imageFolderCID/1.png"),
        };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("stars1988s5h45qwkaqch8km4ceagw2e08vdw2mu2mgs", &[]),
            submit_msg,
        )
        .unwrap();
        //Submit NFT
        let submit_msg = ExecuteMsg::SubmitNft {
            proceed_recipient: String::from("proceed_recipient0000"),
            token_uri: String::from("ipfs://imageFolderCID/2.png"),
        };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("stars1988s5h45qwkaqch8km4ceagw2e08vdw2mu2mgs", &[]),
            submit_msg,
        )
        .unwrap();
        //Submit NFT
        let submit_msg = ExecuteMsg::SubmitNft {
            proceed_recipient: String::from("proceed_recipient0000"),
            token_uri: String::from("ipfs://imageFolderCID/3.png"),
        };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("stars1988s5h45qwkaqch8km4ceagw2e08vdw2mu2mgs", &[]),
            submit_msg,
        )
        .unwrap();

        //Query Submissions
        let query_msg = QueryMsg::Submissions { submission_id: None, limit: None, start_after: None };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        let resp: SubmissionsResponse = from_json(&res).unwrap();
        assert_eq!(resp.submissions.len().to_string(), String::from("3"));
        //Query Submissions, a single submission
        let query_msg = QueryMsg::Submissions { submission_id: Some(1), limit: None, start_after: None };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        let resp: SubmissionsResponse = from_json(&res).unwrap();
        assert_eq!(resp.submissions.len().to_string(), String::from("1"));
        assert_eq!(resp.submissions[0].submission.token_uri, String::from("ipfs://imageFolderCID/2.png"));
        //Query Submissions
        let query_msg = QueryMsg::Submissions { submission_id: None, limit: None, start_after: Some(0) };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        let resp: SubmissionsResponse = from_json(&res).unwrap();
        assert_eq!(resp.submissions.len().to_string(), String::from("2"));
        assert_eq!(resp.submissions[0].submission.token_uri, String::from("ipfs://imageFolderCID/2.png"));
        assert_eq!(resp.submissions[1].submission.token_uri, String::from("ipfs://imageFolderCID/3.png"));
    }

    #[test]
    fn curate_nft(){        
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            sg721_code_id: None,
            minter_addr: Some(String::from("some_minter_address")),
            base_factory_address: String::from("stars1a45hcxty3spnmm2f0papl8v4dk5ew29s4syhn4efte8u5haex99qlkrtnx"),
            bid_denom: String::from("cdt"),
            incentive_denom: Some(String::from("meme")),
            first_submission: SubmissionInfo {
                submitter: Addr::unchecked(""),
                proceed_recipient: Addr::unchecked("proceed_recipient0000"),
                token_uri: String::from("ipfs://imageFolderCID/1.png"),
            },
            mint_cost: 1000u128,
        };
        //Instantiating contract
        let v_info = mock_info("sender88", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), v_info, msg).unwrap();

        //Submit NFT
        let submit_msg = ExecuteMsg::SubmitNft {
            proceed_recipient: String::from("proceed_recipient0000"),
            token_uri: String::from("ipfs://imageFolderCID/1.png"),
        };
        //Submission 1
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("stars1988s5h45qwkaqch8km4ceagw2e08vdw2mu2mgs", &[]),
            submit_msg.clone(),
        )
        .unwrap();
        //Submission 2
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("stars1988s5h45qwkaqch8km4ceagw2e08vdw2mu2mgs", &[]),
            submit_msg.clone(),
        )
        .unwrap();
        //Curate Multiple NFTs
        let curate_msg = ExecuteMsg::VoteToCurate {
            submission_ids: vec![0,1],
        };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("stars1988s5h45qwkaqch8km4ceagw2e08vdw2mu2mgs", &[]),
            curate_msg.clone(),
        )
        .unwrap();
        
        //Error bc submission got bumped to the auction state
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("stars1988s5h45qwkaqch8km4ceagw2e08vdw2mu2mgs", &[]),
            curate_msg,
        )
        .unwrap_err();

        //Query Submissions to check if they were deleted
        let query_msg = QueryMsg::Submissions { submission_id: None, limit: None, start_after: None };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        let resp: SubmissionsResponse = from_json(&res).unwrap();
        assert_eq!(resp.submissions.len().to_string(), String::from("0"));

        //Query Pending Auctions, there should be 2 waiting to be auctioned
        let query_msg = QueryMsg::PendingAuctions { limit: None, start_after: None };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        let resp: PendingAuctionResponse = from_json(&res).unwrap();
        assert_eq!(resp.pending_auctions.len().to_string(), String::from("2"));

    }
    
    #[test]
    fn bid_for_nft(){

        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            sg721_code_id: None,
            minter_addr: Some(String::from("some_minter_address")),
            base_factory_address: String::from("stars1a45hcxty3spnmm2f0papl8v4dk5ew29s4syhn4efte8u5haex99qlkrtnx"),
            bid_denom: String::from("cdt"),
            incentive_denom: Some(String::from("meme")),
            first_submission: SubmissionInfo {
                submitter: Addr::unchecked(""),
                proceed_recipient: Addr::unchecked("proceed_recipient0000"),
                token_uri: String::from("ipfs://imageFolderCID/1.png"),
            },
            mint_cost: 1000u128,
        };
        //Instantiating contract
        let v_info = mock_info("sender88", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), v_info, msg).unwrap();

        //Bid for NFT
        let bid_msg = ExecuteMsg::BidForNft { };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder0000", &[coin(10_000_000, "cdt")]),
            bid_msg,
        )
        .unwrap();    


        //Query live auction to confirm bid
        let query_msg = QueryMsg::LiveNFTAuction { };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        let resp: Auction = from_json(&res).unwrap();
        assert_eq!(resp, Auction {
            submission_info: SubmissionItem {
                submission: SubmissionInfo {
                    submitter: Addr::unchecked("sender88"),
                    proceed_recipient: Addr::unchecked("proceed_recipient0000"),
                    token_uri: String::from("ipfs://imageFolderCID/1.png"),
                },
                curators: vec![],
                votes: 0u64,
                submission_end_time: 1572402219,
            },
            bids: vec![
                Bid {
                bidder: Addr::unchecked("bidder0000"),
                amount: 10000000,
            }],
            highest_bid: Bid {
                bidder: Addr::unchecked("bidder0000"),
                amount: 10000000,
            },
            auction_end_time: 1571883819,
        
        } );

        /////Use integration tests to test that the replaced bid is sent back to the user
    }
    
    #[test]
    fn conclude_auction(){
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            sg721_code_id: None,
            minter_addr: Some(String::from("some_minter_address")),
            base_factory_address: String::from("stars1a45hcxty3spnmm2f0papl8v4dk5ew29s4syhn4efte8u5haex99qlkrtnx"),
            bid_denom: String::from("cdt"),
            incentive_denom: Some(String::from("meme")),
            first_submission: SubmissionInfo {
                submitter: Addr::unchecked(""),
                proceed_recipient: Addr::unchecked("proceed_recipient0000"),
                token_uri: String::from("ipfs://imageFolderCID/1.png"),
            },
            mint_cost: 1000u128,
        };
        //Instantiating contract
        let v_info = mock_info("sender88", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), v_info, msg).unwrap();

        //Submit NFT
        let submit_msg = ExecuteMsg::SubmitNft {
            proceed_recipient: String::from("proceed_recipient0000"),
            token_uri: String::from("ipfs://imageFolderCID/submission1.png"),
        };
        //Submission 1
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("stars1988s5h45qwkaqch8km4ceagw2e08vdw2mu2mgs", &[coin(10_000_000, "cdt")]),
            submit_msg.clone(),).unwrap();
        //Curate to send Submission 1 to pending auction
        let curate_msg = ExecuteMsg::VoteToCurate {
            submission_ids: vec![0],
        };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("stars1988s5h45qwkaqch8km4ceagw2e08vdw2mu2mgs", &[]),
            curate_msg.clone(),
        )
        .unwrap();

        //Conclude live auction: Error
        let conclude_msg = ExecuteMsg::ConcludeAuction { };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sender88", &[]),
            conclude_msg.clone(),
        ).unwrap_err();

        //Push env to auction end time
        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(86400);

        //Conclude live auction: Success
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("bidder0000", &[]),
            conclude_msg.clone(),
        ).unwrap();

        //Bid for NFT works because auction was extended
        let bid_msg = ExecuteMsg::BidForNft { };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder0000", &[coin(10_000_000, "cdt")]),
            bid_msg,
        )
        .unwrap();

        //Push env to auction end time
        env.block.time = env.block.time.plus_seconds(86400 );

        //Conclude live auction: Success
        let _res = execute(
            deps.as_mut(),
            env,
            mock_info("bidder0000", &[]),
            conclude_msg,
        ).unwrap();

        //Query live auction to confirm pending auction was started
        let query_msg = QueryMsg::LiveNFTAuction { };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        let resp: Auction = from_json(&res).unwrap();
        assert_eq!(resp, Auction {
            submission_info: SubmissionItem {
                submission: SubmissionInfo {
                    submitter: Addr::unchecked("stars1988s5h45qwkaqch8km4ceagw2e08vdw2mu2mgs"),
                    proceed_recipient: Addr::unchecked("proceed_recipient0000"),
                    token_uri: String::from("ipfs://imageFolderCID/submission1.png"),
                },
                curators: vec![Addr::unchecked("stars1988s5h45qwkaqch8km4ceagw2e08vdw2mu2mgs")],
                votes: 1u64,
                submission_end_time: 1572402219,
            },
            bids: vec![],
            highest_bid: Bid {
                bidder: Addr::unchecked(""),
                amount: 0,
            },
            auction_end_time: 1572056619,
        
        } );

        //Query to assert current bid asset auction
        let query_msg = QueryMsg::LiveBidAssetAuction { };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        let resp: BidAssetAuction = from_json(&res).unwrap();
        assert_eq!(resp, BidAssetAuction { 
            auctioned_asset: Coin { denom: String::from("cdt"), amount: Uint128::new(10000000) },
            highest_bid: Bid { bidder: Addr::unchecked("cosmos2contract"), amount: 0u128 }, 
        } )

    }
    
    #[test]
    fn bid_for_bid_asset(){
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            sg721_code_id: None,
            minter_addr: Some(String::from("some_minter_address")),
            base_factory_address: String::from("stars1a45hcxty3spnmm2f0papl8v4dk5ew29s4syhn4efte8u5haex99qlkrtnx"),
            bid_denom: String::from("cdt"),
            incentive_denom: Some(String::from("meme")),
            first_submission: SubmissionInfo {
                submitter: Addr::unchecked(""),
                proceed_recipient: Addr::unchecked("proceed_recipient0000"),
                token_uri: String::from("ipfs://imageFolderCID/1.png"),
            },
            mint_cost: 1000u128,
        };
        //Instantiating contract
        let v_info = mock_info("sender88", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), v_info, msg).unwrap();

        
        //Submit NFT
        let submit_msg = ExecuteMsg::SubmitNft {
            proceed_recipient: String::from("proceed_recipient0000"),
            token_uri: String::from("ipfs://imageFolderCID/submission1.png"),
        };
        //Submission 1
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("stars1988s5h45qwkaqch8km4ceagw2e08vdw2mu2mgs", &[]),
            submit_msg.clone(),).unwrap();
        //Curate to send Submission 1 to pending auction
        let curate_msg = ExecuteMsg::VoteToCurate {
            submission_ids: vec![0],
        };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("stars1988s5h45qwkaqch8km4ceagw2e08vdw2mu2mgs", &[]),
            curate_msg.clone(),
        )
        .unwrap();
    
        //Bid for NFT
        let bid_msg = ExecuteMsg::BidForNft { };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder0000", &[coin(20_000_000, "cdt")]),
            bid_msg,
        )
        .unwrap();

        //Bid Auction: Error - no live bid asset auction
        let bid_msg = ExecuteMsg::BidForAssets {  };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder0000", &[coin(10_000_000, "not_bid_asset")]),
            bid_msg,
        )
        .unwrap_err();

        //Push env to auction end time
        let mut env = mock_env();
        //Push env to auction end time
        env.block.time = env.block.time.plus_seconds(86400 );

        // //Conclude live auction: Success
        // let conclude_msg = ExecuteMsg::ConcludeAuction { };
        // let _res = execute(
        //     deps.as_mut(),
        //     env.clone(),
        //     mock_info("bidder0000", &[]),
        //     conclude_msg,
        // ).unwrap();

        // //Bid Auction: Error - wrong asset
        // let bid_msg = ExecuteMsg::BidForAssets {  };
        // let _res = execute(
        //     deps.as_mut(),
        //     mock_env(),
        //     mock_info("bidder0000", &[coin(10_000_000, "not_auction_asset")]),
        //     bid_msg,
        // )
        // .unwrap_err();

        // //Bid Auction: Success
        // let bid_msg = ExecuteMsg::BidForAssets {  };
        // let _res = execute(
        //     deps.as_mut(),
        //     mock_env(),
        //     mock_info("bidder0000", &[coin(10_000_000, "meme")]),
        //     bid_msg,
        // )
        // .unwrap();    

        // //Bid Auction: Error - bid too low
        // let bid_msg = ExecuteMsg::BidForAssets {  };
        // let res = execute(
        //     deps.as_mut(),
        //     mock_env(),
        //     mock_info("bidder0000", &[coin(10_000_100, "meme")]),
        //     bid_msg,
        // )
        // .unwrap_err();
        // assert_eq!(&res.to_string(), "Custom Error val: Bid is lower than the minimum outbid amount");
        
        // //Query to assert current bid asset bids
        // let query_msg = QueryMsg::LiveBidAssetAuction { };
        // let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        // let resp: BidAssetAuction = from_json(&res).unwrap();
        // assert_eq!(resp, 
        //     BidAssetAuction { 
        //     auctioned_asset: Coin { denom: String::from("cdt"), amount: Uint128::new(20000000) },
        //     highest_bid: Bid { bidder: Addr::unchecked("bidder0000"), amount: 10000000u128 }, 
        // });
        
        // //Bid Auction: Outbid Success
        // let bid_msg = ExecuteMsg::BidForAssets {  };
        // let _res = execute(
        //     deps.as_mut(),
        //     mock_env(),
        //     mock_info("newbidder0000", &[coin(10_100_000, "meme")]),
        //     bid_msg,
        // )
        // .unwrap();  
        // //Query to assert new highest bid
        // let query_msg = QueryMsg::LiveBidAssetAuction { };
        // let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        // let resp: BidAssetAuction = from_json(&res).unwrap();
        // assert_eq!(resp, 
        //     BidAssetAuction { 
        //     auctioned_asset: Coin { denom: String::from("cdt"), amount: Uint128::new(20000000) },
        //     highest_bid: Bid { bidder: Addr::unchecked("newbidder0000"), amount: 10100000u128 }, 
        // });  

        
        // //Bid for NFT of 2nd Auction
        // let bid_msg = ExecuteMsg::BidForNft { };
        // let _res = execute(
        //     deps.as_mut(),
        //     mock_env(),
        //     mock_info("bidder0000", &[coin(10_000_000, "cdt")]),
        //     bid_msg,
        // )
        // .unwrap();
        
        // //Push env to auction end time
        // env.block.time = env.block.time.plus_seconds(86400 );

        // //Conclude live auction: Success
        // let conclude_msg = ExecuteMsg::ConcludeAuction { };
        // let _res = execute(
        //     deps.as_mut(),
        //     env,
        //     mock_info("bidder0000", &[]),
        //     conclude_msg,
        // ).unwrap();

        // //Check that the bid auction was concluded
        // let query_msg = QueryMsg::LiveBidAssetAuction { };
        // let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        // let resp: BidAssetAuction = from_json(&res).unwrap();
        // assert_eq!(resp, 
        //     BidAssetAuction { 
        //     auctioned_asset: Coin { denom: String::from("cdt"), amount: Uint128::new(10000000) },
        //     highest_bid: Bid { bidder: Addr::unchecked("cosmos2contract"), amount: 0 }, 
        // });  
    }

    #[test]
    fn update_config(){
        
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            sg721_code_id: None,
            minter_addr: Some(String::from("some_minter_address")),
            base_factory_address: String::from("stars1a45hcxty3spnmm2f0papl8v4dk5ew29s4syhn4efte8u5haex99qlkrtnx"),
            bid_denom: String::from("cdt"),
            incentive_denom: Some(String::from("meme")),
            first_submission: SubmissionInfo {
                submitter: Addr::unchecked(""),
                proceed_recipient: Addr::unchecked("proceed_recipient0000"),
                token_uri: String::from("ipfs://imageFolderCID/1.png"),
            },
            mint_cost: 1000u128,
        };
        //Instantiating contract
        let v_info = mock_info("sender88", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), v_info, msg).unwrap();

        //Update config: Not owner
        let update_config_msg = ExecuteMsg::UpdateConfig {
            owner: None,
            bid_denom: None,
            minimum_outbid: None,
            incentive_denom: None,
            incentive_distribution_amount: Some(0u128),
            incentive_bid_percent: None,
            mint_cost: None,
            submission_cost: None,
            submission_limit: None,
            submission_vote_period: None,
            curation_threshold: None,
            auction_period: None,
        };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder0000", &[]),
            update_config_msg,
        )
        .unwrap_err();
        
        //Update config
        let update_config_msg = ExecuteMsg::UpdateConfig {
            owner: None,
            bid_denom: Some(String::from("different")),
            minimum_outbid: Some(Decimal::zero()),
            incentive_denom: Some(String::from("different")),
            incentive_distribution_amount: Some(0u128),
            incentive_bid_percent: Some(Decimal::zero()),
            mint_cost: Some(0u128),
            submission_cost: Some(0u128),
            submission_limit: Some(0),
            submission_vote_period: Some(0),
            curation_threshold: Some(Decimal::zero()),
            auction_period: Some(0),
        };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sender88", &[]),
            update_config_msg,
        )
        .unwrap();

        //Query config
        let query_msg = QueryMsg::Config { };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        //Assert changes
        let resp: Config = from_json(&res).unwrap();
        assert_eq!(resp, Config {
            owner: Addr::unchecked("sender88"),
            bid_denom: String::from("different"),
            minimum_outbid: Decimal::zero(),
            incentive_denom: Some(String::from("different")),
            incentive_distribution_amount: 0u128,
            incentive_bid_percent: Decimal::zero(),
            current_token_id: 0u64,
            current_submission_id: 0u64,
            minter_addr: String::from("some_minter_address"),
            auction_period: 0u64,
            curation_threshold: Decimal::zero(),
            submission_cost: 0u128,
            submission_limit: 0u64,
            submission_total: 0u64,
            submission_vote_period: 0u64,
            mint_cost: 0u128,
        } );

    }
}