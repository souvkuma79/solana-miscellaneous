//This routine creates the new wallet account with seed phrase
pub fn CreateSPLTokenAccount(payer_keypair: Keypair, spl_token_mint_token_b58: String, 
    spl_token_mint_account_b58: String, cluster_url: String)  -> Result<String, Box<dyn std::error::Error>> {
    let payer_pubkey = payer_keypair.pubkey();
    let spl_token_mint_token_pubkey: solana_program::pubkey::Pubkey = Pubkey::from_str(&spl_token_mint_token_b58).unwrap();
    let spl_token_mint_account_pubkey: solana_program::pubkey::Pubkey = Pubkey::from_str(&spl_token_mint_account_b58).unwrap();

    //Create a new Keypair for the new account
    let wallet_keypair = Keypair::new();
    let wallet_pubkey = wallet_keypair.pubkey();

    //Create RPC client to be used to talk to Solana cluster
    let rpc = RpcClient::new_with_commitment(cluster_url, CommitmentConfig::confirmed());

    //Number of bytes to allocate for the new account data
    let space = spl_token::state::Account::get_packed_len();

    //Calculate min rent according to expected account data size
    let rent = rpc.get_minimum_balance_for_rent_exemption(space)?;

    //Build instruction for create account 
    let create_token_acc_ix = solana_program::system_instruction::create_account_with_seed(
        &payer_pubkey,
        &wallet_pubkey,
        &payer_pubkey,
        &"test",
        rent,
        space as u64,
        &spl_token::ID,
    );
	
    //Build instruction for initialize account
    let init_token_acc_ix: Instruction  = spl_token::instruction::initialize_account(
        &spl_token::ID,        
        &wallet_pubkey,
        &spl_token_mint_token_pubkey,
        &payer_pubkey   
    ).unwrap();

    //Build instruction for create associated token account
    let create_asso_acc_ix =  
    spl_associated_token_account::create_associated_token_account(&payer_pubkey, &wallet_pubkey, &spl_token_mint_token_pubkey);

    //Build instruction for Transport lamport
    let transfer_tokens_ix: Instruction = spl_token::instruction::transfer(
    &spl_token::ID,
    &spl_token_mint_account_pubkey,
    &wallet_pubkey,
    &payer_pubkey,
    &[&payer_pubkey],
    0
    ).unwrap();

    //List all the Instructions
    let ixs = vec![create_token_acc_ix, init_token_acc_ix, create_asso_acc_ix, transfer_tokens_ix];

    //Get the recent block
    let latest_blockhash = rpc.get_latest_blockhash()?;

    //Send transaction to the cluster and wait for confirmation
    let signers = [&payer_keypair, &wallet_keypair];
    let txn = Transaction::new_signed_with_payer(
        &ixs,
        Some(&payer_pubkey),
        &signers,
        latest_blockhash,
    ); 

    //Send transaction to the cluster and wait for confirmation
    let tx_hash = rpc.send_and_confirm_transaction_with_spinner(&txn)?;
    println!("Transaction hash: {:?}",tx_hash);

    //Get balance
    let payer_account_balance = rpc.get_balance(&payer_pubkey).unwrap();
	println!("Balance fetch from payer account is successfull, balance: {}", payer_account_balance);

    let new_account_balance = rpc.get_token_account_balance(&wallet_pubkey).unwrap();
    println!("Balance fetch from newly created token associated account is successfull, balance: {:?}", new_account_balance);

    Ok(wallet_pubkey.to_string())
}