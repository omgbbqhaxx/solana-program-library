use clap::ArgMatches;
use solana_clap_utils::{
    input_parsers::pubkey_of_signer,
    keypair::{pubkey_from_path, signer_from_path},
};
use solana_client::{blockhash_query::BlockhashQuery, rpc_client::RpcClient};
use solana_remote_wallet::remote_wallet::RemoteWalletManager;
use solana_sdk::{pubkey::Pubkey, signature::Signer};
use spl_associated_token_account::*;
use std::{process::exit, sync::Arc};

pub(crate) struct Config<'a> {
    pub(crate) rpc_client: RpcClient,
    pub(crate) verbose: bool,
    pub(crate) fee_payer: Pubkey,
    pub(crate) default_keypair_path: String,
    pub(crate) nonce_account: Option<Pubkey>,
    pub(crate) nonce_authority: Option<Pubkey>,
    pub(crate) blockhash_query: BlockhashQuery,
    pub(crate) sign_only: bool,
    pub(crate) multisigner_pubkeys: Vec<&'a Pubkey>,
}

impl<'a> Config<'a> {
    // Check if an explicit token account address was provided, otherwise
    // return the associated token address for the default address.
    pub(crate) fn associated_token_address_or_override(
        &self,
        arg_matches: &ArgMatches,
        override_name: &str,
        wallet_manager: &mut Option<Arc<RemoteWalletManager>>,
    ) -> Pubkey {
        if let Some(address) = pubkey_of_signer(arg_matches, override_name, wallet_manager).unwrap()
        {
            return address;
        }

        let token = pubkey_of_signer(arg_matches, "token", wallet_manager)
            .unwrap()
            .unwrap();
        let owner = self
            .default_signer(arg_matches, wallet_manager)
            .unwrap_or_else(|e| {
                eprintln!("error: {}", e);
                exit(1);
            })
            .pubkey();
        get_associated_token_address(&owner, &token)
    }

    // Checks if an explicit address was provided, otherwise return the default address.
    pub(crate) fn pubkey_or_default(
        &self,
        arg_matches: &ArgMatches,
        address_name: &str,
        wallet_manager: &mut Option<Arc<RemoteWalletManager>>,
    ) -> Pubkey {
        if address_name != "owner" {
            if let Some(address) =
                pubkey_of_signer(arg_matches, address_name, wallet_manager).unwrap()
            {
                return address;
            }
        }

        return self
            .default_address(arg_matches, wallet_manager)
            .unwrap_or_else(|e| {
                eprintln!("error: {}", e);
                exit(1);
            });
    }

    // Checks if an explicit signer was provided, otherwise return the default signer.
    pub(crate) fn signer_or_default(
        &self,
        arg_matches: &ArgMatches,
        authority_name: &str,
        wallet_manager: &mut Option<Arc<RemoteWalletManager>>,
    ) -> (Box<dyn Signer>, Pubkey) {
        let mut load_authority = move || {
            if authority_name != "owner" {
                if let Some(keypair_path) = arg_matches.value_of(authority_name) {
                    return signer_from_path(
                        &arg_matches,
                        keypair_path,
                        authority_name,
                        wallet_manager,
                    );
                }
            }

            self.default_signer(arg_matches, wallet_manager)
        };

        let authority = load_authority().unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(1);
        });

        let authority_address = authority.pubkey();
        (authority, authority_address)
    }

    fn default_address(
        &self,
        matches: &ArgMatches,
        wallet_manager: &mut Option<Arc<RemoteWalletManager>>,
    ) -> Result<Pubkey, Box<dyn std::error::Error>> {
        // for backwards compatibility, check owner before cli config default
        if let Some(address) = pubkey_of_signer(matches, "owner", wallet_manager).unwrap() {
            return Ok(address);
        }

        let path = &self.default_keypair_path;
        pubkey_from_path(matches, path, "default", wallet_manager)
    }

    fn default_signer(
        &self,
        matches: &ArgMatches,
        wallet_manager: &mut Option<Arc<RemoteWalletManager>>,
    ) -> Result<Box<dyn Signer>, Box<dyn std::error::Error>> {
        // for backwards compatibility, check owner before cli config default
        if let Some(owner_path) = matches.value_of("owner") {
            return signer_from_path(&matches, owner_path, "owner", wallet_manager);
        }

        let path = &self.default_keypair_path;
        signer_from_path(matches, path, "default", wallet_manager)
    }
}
