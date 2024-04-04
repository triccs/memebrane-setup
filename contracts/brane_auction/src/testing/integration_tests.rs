#[cfg(test)]
#[allow(unused_variables)]
mod tests {

    use cosmwasm_std::{ to_json_binary,
        coin, Addr, Binary, Empty, Response, StdResult, Uint128, Decimal,
    };
    use cw721::TokensResponse;
    use cw_multi_test::{App, AppBuilder, BankKeeper, Contract, ContractWrapper, Executor};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use crate::state::SubmissionInfo;
    use crate::testing::helpers::AuctionContract;
    use crate::msgs::InstantiateMsg;

    const USER: &str = "user";
    const ADMIN: &str = "admin";

    //Auction Contract
    pub fn auction_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(
            crate::contracts::execute,
            crate::contracts::instantiate,
            crate::contracts::query,
        ).with_reply(crate::contracts::reply);
        Box::new(contract)
    }

    
    //Mock sg721 Contract
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum sg721_MockExecuteMsg {
        TransferNft {
            recipient: String,
            token_id: String,
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct sg721_MockInstantiateMsg {}

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum sg721_MockQueryMsg {
        Tokens {
            owner: String,
            start_after: Option<String>,
            limit: Option<u32>,
        },
        AllTokens {
            start_after: Option<String>,
            limit: Option<u32>,
        }
    }
    
    pub fn sg721_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            |deps, _, info, msg: sg721_MockExecuteMsg| -> StdResult<Response> {
                Ok(Response::default())
            },
            |_, _, _, _: sg721_MockInstantiateMsg| -> StdResult<Response> {
                Ok(Response::default())
            },
            |_, _, msg: sg721_MockQueryMsg| -> StdResult<Binary> {
                match msg {
                    sg721_MockQueryMsg::Tokens { owner, start_after, limit } => {
                        if owner == "three_votes" {
                            Ok(to_json_binary(&TokensResponse {
                                tokens: vec![                                    
                                    String::from("1"),
                                    String::from("2"),
                                    String::from("3")
                                ],
                            })?)
                        } else if owner == "not_a_holder" {
                            Ok(to_json_binary(&TokensResponse {
                                tokens: vec![ ],
                            })?)

                        } else {
                            Ok(to_json_binary(&TokensResponse {
                                tokens: vec![
                                    String::from("1")
                                    ],
                            })?)
                        }
                    },
                    sg721_MockQueryMsg::AllTokens { start_after, limit } => {
                        Ok(to_json_binary(&TokensResponse {
                            tokens: vec![
                                String::from("1"),
                                String::from("2"),
                                String::from("3"),
                                String::from("1"),
                                String::from("2"),
                                String::from("3"),
                                String::from("1"),
                                String::from("2"),
                                String::from("3"),
                                String::from("1"),
                                String::from("2"),
                                String::from("3"),
                                String::from("1"),
                                String::from("2"),
                                String::from("3"),
                                String::from("1"),
                                String::from("2"),
                                String::from("3"),
                                String::from("1"),
                                String::from("2"),
                                String::from("3"),
                            ],
                        })?)
                    }
                }
            },
        );
        Box::new(contract)
    }

    //Mock NFT Mint Contract
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum Mint_MockExecuteMsg {
        Mint {
            token_uri: Option<String>,
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct Mint_MockInstantiateMsg {}

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum Mint_MockQueryMsg {
    }
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct Mock_Response { }


    pub fn mint_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            |deps, _, info, msg: Mint_MockExecuteMsg| -> StdResult<Response> {
                Ok(Response::default().add_attribute("token_id", "1"))
            },
            |_, _, _, _: Mint_MockInstantiateMsg| -> StdResult<Response> {
                Ok(Response::default())
            },
            |_, _, msg: Mint_MockQueryMsg| -> StdResult<Binary> {
                Ok(to_json_binary(&Mock_Response {})?)
            },
        );
        Box::new(contract)
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            let bank = BankKeeper::new();

            bank.init_balance(
                storage,
                &Addr::unchecked("contract2"),
                vec![coin(10_000_000, "ustars")],
            )
            .unwrap(); //contract1 = Auction contract
            bank.init_balance(
                storage,
                &Addr::unchecked("asset_bidder"),
                vec![coin(20_000_000, "mbrn")],
            )
            .unwrap();
            bank.init_balance(
                storage,
                &Addr::unchecked("nft_bidder"),
                vec![coin(30_000_000, "cdt")],
            )
            .unwrap();
            bank.init_balance(
                storage,
                &Addr::unchecked("not_a_holder"),
                vec![coin(10_000_000, "cdt")],
            )
            .unwrap();
            bank.init_balance(
                storage,
                &Addr::unchecked(USER),
                vec![coin(100_000_000_000_000, "cdt"), coin(30_000_000_000_000, "mbrn")],
            )
            .unwrap();

            router.bank = bank;
        })
    }

    fn proper_instantiate() -> (App, AuctionContract) {
        let mut app = mock_app();

        //Instaniate Mint
        let proxy_id = app.store_code(mint_contract());

        let mint_contract_addr = app
            .instantiate_contract(
                proxy_id,
                Addr::unchecked(ADMIN),
                &Mint_MockInstantiateMsg {},
                &[],
                "test",
                None,
            )
            .unwrap();

        //Instaniate sg721
        let proxy_id = app.store_code(sg721_contract());

        let sg721_contract_addr = app
            .instantiate_contract(
                proxy_id,
                Addr::unchecked(ADMIN),
                &Mint_MockInstantiateMsg {},
                &[],
                "test",
                None,
            )
            .unwrap();

        //Instantiate Auction contract
        let auction_id = app.store_code(auction_contract());

        let msg = InstantiateMsg {
            sg721_code_id: None,
            sg721_addr: Some(sg721_contract_addr.to_string()),
            minter_addr: Some(mint_contract_addr.to_string()),
            base_factory_address: String::from("stars1a45hcxty3spnmm2f0papl8v4dk5ew29s4syhn4efte8u5haex99qlkrtnx"),
            bid_denom: String::from("cdt"),
            incentive_denom: Some(String::from("mbrn")),
            first_submission: SubmissionInfo {
                submitter: Addr::unchecked(""),
                proceed_recipient: Addr::unchecked("proceed_recipient0000"),
                token_uri: String::from("ipfs://imageFolderCID/1.png"),
            },
            mint_cost: 1000u64,
        };

        let auction_contract_addr = app
            .instantiate_contract(
                auction_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let auction_contract = AuctionContract(auction_contract_addr);

        (app, auction_contract)
    }

    mod auction {

        use crate::{msgs::{ExecuteMsg, QueryMsg}, state::{Bid, BidAssetAuction}};

        use super::*;

        #[test]
        fn mock_usage() {
            let (mut app, auction_contract) = proper_instantiate();

            //Submit NFT: Error without submission funds
            let submit_msg = ExecuteMsg::SubmitNft {
                proceed_recipient: String::from("proceed_recipient0000"),
                token_uri: String::from("ipfs://imageFolderCID/submission1.png"),
            };
            let cosmos_msg = auction_contract.call(submit_msg, vec![]).unwrap();
            app.execute(Addr::unchecked("not_a_holder"), cosmos_msg).unwrap_err();

            //Submit NFT: Success
            let submit_msg = ExecuteMsg::SubmitNft {
                proceed_recipient: String::from("proceed_recipient0000"),
                token_uri: String::from("ipfs://imageFolderCID/submission1.png"),
            };
            let cosmos_msg = auction_contract.call(submit_msg, vec![coin(10_000_000, "cdt")]).unwrap();
            app.execute(Addr::unchecked("not_a_holder"), cosmos_msg).unwrap();

            //Curate with 1 vote
            let curate_msg = ExecuteMsg::VoteToCurate {
                submission_ids: vec![0],
            };
            let cosmos_msg = auction_contract.call(curate_msg, vec![]).unwrap();
            app.execute(Addr::unchecked("one_vote"), cosmos_msg).unwrap();

            //Curate with 3 votes to pass 11% threshold 
            let curate_msg = ExecuteMsg::VoteToCurate {
                submission_ids: vec![0],
            };
            let cosmos_msg = auction_contract.call(curate_msg, vec![]).unwrap();
            app.execute(Addr::unchecked("three_votes"), cosmos_msg).unwrap();

            //Bid for NFT current live auction
            let bid_msg = ExecuteMsg::BidForNft { };
            let cosmos_msg = auction_contract.call(bid_msg, vec![coin(1_000_000, "cdt")]).unwrap();
            app.execute(Addr::unchecked("nft_bidder"), cosmos_msg).unwrap();
            
            //Bid for NFT current live auction: Error bid too low
            let bid_msg = ExecuteMsg::BidForNft { };
            let cosmos_msg = auction_contract.call(bid_msg, vec![coin(1_000_100, "cdt")]).unwrap();
            app.execute(Addr::unchecked("nft_bidder"), cosmos_msg).unwrap_err();

            //Bid for NFT current live auction: Outbid returns funds
            let bid_msg = ExecuteMsg::BidForNft { };
            let cosmos_msg = auction_contract.call(bid_msg, vec![coin(10_000_000, "cdt")]).unwrap();
            app.execute(Addr::unchecked("nft_bidder"), cosmos_msg).unwrap();


            //Skip a day
            let mut block_info = app.block_info();
            block_info.time = block_info.time.plus_seconds(86400);
            app.set_block(block_info);            

            //Conclude Auction
            let conclude_msg = ExecuteMsg::ConcludeAuction { };
            let cosmos_msg = auction_contract.call(conclude_msg, vec![]).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
            //Bid for Asset Auction: Success
            let bid_msg = ExecuteMsg::BidForAssets { };
            let cosmos_msg = auction_contract.call(bid_msg, vec![coin(1_000_000, "mbrn")]).unwrap();
            app.execute(Addr::unchecked("asset_bidder"), cosmos_msg).unwrap();
                        
            //Bid for NFT current live auction: outbid too low
            let bid_msg = ExecuteMsg::BidForAssets { };
            let cosmos_msg = auction_contract.call(bid_msg, vec![coin(1_000_100, "mbrn")]).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap_err();

            //Bid for Asset Auction: Outbid returns assets
            let bid_msg = ExecuteMsg::BidForAssets { };
            let cosmos_msg = auction_contract.call(bid_msg, vec![coin(10_000_000, "mbrn")]).unwrap();
            app.execute(Addr::unchecked("asset_bidder"), cosmos_msg).unwrap();




            //Bid for NFT current live auction
            let bid_msg = ExecuteMsg::BidForNft { };
            let cosmos_msg = auction_contract.call(bid_msg, vec![coin(10_000_000, "cdt")]).unwrap();
            app.execute(Addr::unchecked("nft_bidder"), cosmos_msg).unwrap();

            //Skip a day
            let mut block_info = app.block_info();
            block_info.time = block_info.time.plus_seconds(86400);
            app.set_block(block_info);
            

            //Query Bid Asset Auction for highest bid
            let query_msg = QueryMsg::LiveBidAssetAuction { };
            let res: BidAssetAuction = app
                .wrap()
                .query_wasm_smart(auction_contract.addr(), &query_msg.clone())
                .unwrap();
            assert_eq!(res.highest_bid, Bid {
                bidder: Addr::unchecked("asset_bidder"),
                amount: 10000000u128
            });

            //Conclude Auction
            let conclude_msg = ExecuteMsg::ConcludeAuction { };
            let cosmos_msg = auction_contract.call(conclude_msg, vec![]).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
                       
            //Check to see that the asset_bidder got the NFT's bid & the submission cost from the non_holder
            assert_eq!(
                app.wrap().query_all_balances(Addr::unchecked("asset_bidder")).unwrap(),
                vec![
                    coin(11_000_000, "cdt"), //new from winning the auction bid - 1 from the first bid and 10 from the submission cost
                    coin(10000000, "mbrn") //old
                    ]
            );
            //Check that the curators have been rewarded with MBRN that was sent in the first bid
            //Both curators get the same amount
            assert_eq!(
                app.wrap().query_all_balances(Addr::unchecked("three_votes")).unwrap(),
                vec![
                    coin(2_500_000, "mbrn")
                    ]
            );
            assert_eq!(
                app.wrap().query_all_balances(Addr::unchecked("one_vote")).unwrap(),
                vec![
                    coin(2_500_000, "mbrn")
                    ]
            );
            //Check that the artist got the auction proceeds
            assert_eq!(
                app.wrap().query_all_balances(Addr::unchecked("proceed_recipient0000")).unwrap(),
                vec![
                    coin(18_000_000, "cdt") // 9 from each auction
                    ]
            );
            //Curators & bid assets only get proceeds from 1 auction bc 
            //..the 1st auction wasn't curated 
            //..the 2nd bid auction just started

            //Update incentive_distribution_amount to 0
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
            let cosmos_msg = auction_contract.call(update_config_msg, vec![]).unwrap();
            app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();
            //This will burn the next bid auction

            //Bid for NFT current live auction: wrong asset
            let bid_msg = ExecuteMsg::BidForAssets { };
            let cosmos_msg = auction_contract.call(bid_msg, vec![coin(10_000_000, "cdt")]).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap_err();

            //Bid for Asset Auction: Success
            let bid_msg = ExecuteMsg::BidForAssets { };
            let cosmos_msg = auction_contract.call(bid_msg, vec![coin(10_000_000, "mbrn")]).unwrap();
            app.execute(Addr::unchecked("asset_bidder"), cosmos_msg).unwrap();
            
            //Submit NFT
            let submit_msg = ExecuteMsg::SubmitNft {
                proceed_recipient: String::from("proceed_recipient0000"),
                token_uri: String::from("ipfs://imageFolderCID/submission1.png"),
            };
            let cosmos_msg = auction_contract.call(submit_msg, vec![]).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();

            //Curate with 3 votes to pass 11% threshold 
            let curate_msg = ExecuteMsg::VoteToCurate {
                submission_ids: vec![1],
            };
            let cosmos_msg = auction_contract.call(curate_msg, vec![]).unwrap();
            app.execute(Addr::unchecked("three_votes"), cosmos_msg).unwrap();

            //Bid for NFT current live auction
            let bid_msg = ExecuteMsg::BidForNft { };
            let cosmos_msg = auction_contract.call(bid_msg, vec![coin(10_000_000, "cdt")]).unwrap();
            app.execute(Addr::unchecked("nft_bidder"), cosmos_msg).unwrap();

            //Skip a day
            let mut block_info = app.block_info();
            block_info.time = block_info.time.plus_seconds(86400);
            app.set_block(block_info);            

            //Conclude Auction
            let conclude_msg = ExecuteMsg::ConcludeAuction { };
            let cosmos_msg = auction_contract.call(conclude_msg, vec![]).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();

            //Check to see that the burn address got the last bid asset bid
            assert_eq!(
                app.wrap().query_all_balances(Addr::unchecked("stars1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8lhzvv")).unwrap(),
                vec![
                    coin(10_000_000, "mbrn")
                    ]
            );
            //Query Bid Asset Auction for default bid
            let query_msg = QueryMsg::LiveBidAssetAuction { };
            let res: BidAssetAuction = app
                .wrap()
                .query_wasm_smart(auction_contract.addr(), &query_msg.clone())
                .unwrap();
            assert_eq!(res.highest_bid, Bid {
                bidder: Addr::unchecked("contract2"),
                amount: 0
            });
            
        }

    }
}
