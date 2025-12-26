import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MeridianProtocol } from "../target/types/meridian_protocol";
import { Connection, Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { program, SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { ASSOCIATED_TOKEN_PROGRAM_ID, createMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { createSignerFromKeypair, generateSigner, KeypairSigner, signerIdentity } from "@metaplex-foundation/umi";
import { fromWeb3JsKeypair, fromWeb3JsPublicKey, toWeb3JsPublicKey } from "@metaplex-foundation/umi-web3js-adapters";
import { createV1, mplCore } from "@metaplex-foundation/mpl-core";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { BN } from "bn.js";

describe("meridian_protocol", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  let connection = provider.connection;

  const program = anchor.workspace.meridianProtocol as   Program<MeridianProtocol>;

  const payer = provider.wallet;

  //Day 1, Airdrop sol to all of them..
  let authority: Keypair;
  //admins(2 of them, diff keypairs)
  let admin_one: Keypair;
  let admin_two: Keypair;
  //lender..
  let lender: Keypair;
  //borrower..
  let borrower: Keypair;
  //liquidator..
  let liquidator: Keypair;

  //PDA's (with seeds for all of them)
  //lending pool
  let lending_pool_pda: PublicKey;
  //seize vault
  let lending_pool_seize_vault_PDA: PublicKey;
  //verification vault
  let lending_pool_verification_vault: PublicKey;
  //admin reg
  let admin_registry: PublicKey;
  //mock oracle
  let mock_oracle: PublicKey;

  //Day 2
 
  //RWA for the NFT(setup)...

  //MINTS -> USDC and LP Tokens
  let mint_usdc: PublicKey;
  let mint_lp: PublicKey;

  //ATA's
  //USDC ATA's (Lending Pool, Lender,Borrower,Liquidator)..
  
  let lending_pool_usdc_ata: PublicKey;
  let lender_usdc_ata: PublicKey;
  let borrower_usdc_ata: PublicKey;
  let liquidator_usdc_ata: PublicKey;

  //LP ATA (Lending Pool,Lender)
  let lending_pool_lp_ata: PublicKey;
  let lender_lp_ata: PublicKey;

  let lending_pool_bump: number;
  let lending_pool_seize_vault_bump: number;
  let lending_pool_verification_vault_bump: number;
  let admin_registry_bump: number;
  let mock_oracle_bump: number;
  
   //ASSET (GOLD RWA)
   let asset: KeypairSigner;
   let assetAddress: PublicKey;
   let umi: any;


  before( "Setting up accounts", async() =>{

    authority =  await generateKeypair("Authority" ,authority);
    admin_one = await generateKeypair("Admin One", admin_one);
    admin_two = await generateKeypair("Admin Two",admin_two);
    lender = await generateKeypair("Lender",lender);
    borrower = await generateKeypair("Borrower",borrower);
    liquidator =  await generateKeypair("Liquidator", liquidator);
    

    //Airdropping 
    await airdrop(provider,authority.publicKey, 100,connection);
    await airdrop(provider,admin_one.publicKey,100,connection);
    await airdrop(provider,admin_two.publicKey,100,connection);
    await airdrop(provider,lender.publicKey,100,connection);
    await airdrop(provider,borrower.publicKey,100,connection);
    await airdrop(provider,liquidator.publicKey,100,connection);

    umi = createUmi(connection);
    const umiKeypair = fromWeb3JsKeypair(payer.payer);
    const umiSigner = createSignerFromKeypair(umi,umiKeypair);
    umi.use(signerIdentity(umiSigner));
    umi.use(mplCore());

    asset  = generateSigner(umi);

    await createV1(
      umi,
      {
        asset,
        name: "GOLD RWA",
        uri: " ",
        owner: fromWeb3JsPublicKey(borrower.publicKey)
      }
    );

    assetAddress = toWeb3JsPublicKey(asset.publicKey);
        console.log("Asset created at: ", assetAddress.toBase58());

    //Creating pda's

    //Lending Pool PDA
    [lending_pool_pda,lending_pool_bump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("meridian_pool"),
        authority.publicKey.toBuffer()
      ],
      program.programId
    );

    console.log("Lending Pool PDA: ",lending_pool_pda.toBase58());
    console.log("Lending Pool Bump: ",lending_pool_bump);

    //SEIZE VAULT PDA
    [lending_pool_seize_vault_PDA,lending_pool_seize_vault_bump]  = PublicKey.findProgramAddressSync(
      [
        Buffer.from("meridian_seize_vault"),
        lending_pool_pda.toBuffer()
      ],
      program.programId,
    );

    console.log("Lending Pool Seize Vault PDA: ",lending_pool_seize_vault_PDA.toBase58());
    console.log("Lending Pool Seize Vault Bump: ",lending_pool_seize_vault_bump);

    //Verification Vault PDA
    [lending_pool_verification_vault,lending_pool_verification_vault_bump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("meridian_verification_vault"),
        lending_pool_pda.toBuffer()
      ],
      program.programId
    );

    console.log("Lending Pool Verification Vault PDA:",lending_pool_verification_vault.toBase58());
    console.log("Lending Pool Verification Vault Bump: ",lending_pool_verification_vault_bump);


    //Admin Registry
    [admin_registry,admin_registry_bump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("meridian_pool_admin_registry"),
        lending_pool_pda.toBuffer(),
      ],
      program.programId
    );

    console.log("Lending Pool Admin Registry: ", admin_registry.toBase58());
    console.log("Lending Pool Admin Registry Bump: ", admin_registry_bump);

    //Mock Oracle
    [mock_oracle,mock_oracle_bump]= PublicKey.findProgramAddressSync(
      [
        Buffer.from("meridian_mock_oracle"),
        lending_pool_pda.toBuffer(),
      ],
      program.programId
    );

    console.log("Lending Pool Mock Oracle: ",mock_oracle.toBase58());
    console.log("Mock Oracle Bump: ", mock_oracle_bump);


    //Creating mints
    mint_usdc = await createMint(
      connection,
      payer.payer,
      authority.publicKey,
      null,
      6
    );

    console.log(`Mint USDC created at : ${mint_usdc.toBase58()}`);


    mint_lp = await createMint(
      connection,
      payer.payer,
      authority.publicKey,
      null,
      6
    );

    console.log(`Mint LP created at : ${mint_lp.toBase58()}`);

    //Creating ATA's
    lender_usdc_ata = await createAta("USDC","Lender",lender_usdc_ata,connection, lender,mint_usdc,lender.publicKey);
    lender_lp_ata = await createAta("LP","Lender",lender_lp_ata,connection,lender,mint_lp,lender.publicKey);
    
    //e The owners for both LPool ATA's are Authority
    lending_pool_usdc_ata = await createAta("USDC","Lending Pool",lending_pool_usdc_ata,connection, authority,mint_usdc,authority.publicKey);
    lending_pool_lp_ata = await createAta("LP","Lending Pool",lending_pool_lp_ata,connection,authority,mint_lp,authority.publicKey);

    borrower_usdc_ata = await createAta("USDC","Borrower",borrower_usdc_ata,connection, borrower,mint_usdc,borrower.publicKey);
    liquidator_usdc_ata = await createAta("USDC","Liquidator",liquidator_usdc_ata,connection,liquidator,mint_usdc,liquidator.publicKey);
    
    //Minting USDC and LP's to the necessary ATA's
    await mintTokens("Lending Pool ATA" ,"USDC",connection,authority,mint_usdc,authority,10,lending_pool_usdc_ata);
    await mintTokens("Lender USDC ATA", "USDC",connection,authority,mint_usdc,authority,10,lender_usdc_ata);
    
    await mintTokens("Lending POOL LP ATA", "LP",connection,authority,mint_lp, authority,10,lending_pool_lp_ata);
  })  
  
  it("Initialize the Pool",async() => {
    let ltv = 7500; //LTV = 75%
    let u1_bps = 0; 
    let u2_bps = 2500; 
    let u3_bps = 5000;
    let u4_bps = 7500;
    let u5_bps = 9000;

    let apr_1 = 500;
    let apr_2 = 800;
    let apr_3 = 1200;
    let apr_4 = 1800;
    let apr_5 = 2000;

    let liquidation_threshold_bps =10000;
    let liquidation_penalty_bps = 1000; //10% of total debt
    let liquidator_reward_bps = 2000; //20% of total penalty

    let early_withdrawal_fee_bps = 100;
    let origination_fee_bps = 100;
    
    let withdrawal_epoch = new BN(7*86400);

    const tx = await program.methods.initialize(
       ltv,
       u1_bps,
       u2_bps,
       u3_bps,
       u4_bps,
       u5_bps,
       apr_1,
       apr_2,
       apr_3,
       apr_4,
       apr_5,
       early_withdrawal_fee_bps,
       origination_fee_bps,
       withdrawal_epoch,
       liquidation_threshold_bps,
       liquidation_penalty_bps,
       liquidator_reward_bps
    ).accountsPartial({
      authority: authority.publicKey,
      mint: mint_usdc,
      mintLp: mint_lp,
      lendingPool: lending_pool_pda,
      adminRegistry: admin_registry,
      mockOracle: mock_oracle,
      lendingPoolUsdcAta: lending_pool_usdc_ata,
      lendingPoolLpAta: lending_pool_lp_ata,
      protocolSeizeVault: lending_pool_seize_vault_PDA,
      protocolVerificationVault: lending_pool_verification_vault,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).signers([authority]).rpc();

    console.log("Pool Initialized Succesfully...",tx);

    const poolState = await program.account.lendingPool.fetch(lending_pool_pda);
    log_state("Pool Authority: ", poolState.owner);
    const pool_usdc_balance = await connection.getTokenAccountBalance(lender_usdc_ata);
    log_state("Pool USDC Balance After Initialization: ", pool_usdc_balance.value.amount);

    const pool_lp_balance = await connection.getTokenAccountBalance(lender_lp_ata);
    log_state("Pool LP Balance After Initialization: ", pool_lp_balance.value.amount);
  })


  it("Add Admin Test: ", async() => {
    const tx = await program.methods.addAdmin(admin_one.publicKey).accountsPartial({
      authority: authority.publicKey,
      adminRegistry: admin_registry,
      lendingPool: lending_pool_pda,
      systemProgram: SystemProgram.programId,
    }).signers([authority]).rpc();

    console.log("Admin added succesfully : ", tx);

    const adminregstate = await program.account.adminRegistry.fetch(admin_registry);
    log_state("Admin registry admins : ", adminregstate.admins)
  })


  //Lock Pool Test 
  it("Lock Pool", async() => {
    const tx = await program.methods.lock().accountsPartial({
      authority: authority.publicKey,
      lendingPool: lending_pool_pda,
      systemProgram: SystemProgram.programId
    }).signers([authority]).rpc();

    console.log("Pool Locked Succesfully: ",tx);
    const PoolState = await program.account.lendingPool.fetch(lending_pool_pda);
    log_state(`Pool Is Locked : `,  PoolState.isLocked);
  })

   it("UnLock Pool", async() => {
    const tx = await program.methods.unlockPool().accountsPartial({
      authority: authority.publicKey,
      lendingPool: lending_pool_pda,
      systemProgram: SystemProgram.programId
    }).signers([authority]).rpc();

    console.log("Pool Unlocked Succesfully: ",tx);
    const PoolState = await program.account.lendingPool.fetch(lending_pool_pda);
    log_state(`Pool Is Locked : `,  PoolState.isLocked);
  })
  
  it("Lending", async() => {
    const lender_usdc_balance_before  = await connection.getTokenAccountBalance(lender_usdc_ata);
    const lender_lp_balace_before = await connection.getTokenAccountBalance(lender_lp_ata);
        
   log_state("Lender USDC Balance Before Deposit: ", lender_usdc_balance_before.value.amount);
   log_state("Lender LP State Before Balance: ", lender_lp_balace_before.value.amount);
    const tx = await program.methods.deposit(new BN(5)).accountsPartial({
      authority: authority.publicKey,
      lender: lender.publicKey,
      mint: mint_usdc,
      mintLp: mint_lp,
      lendingPool: lending_pool_pda,
      lenderUsdcAta: lender_usdc_ata,
      lenderLpAta: lender_lp_ata,
      lendingPoolLpAta: lending_pool_lp_ata,
      lendingPoolUsdcAta: lending_pool_usdc_ata,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).signers([authority,lender]).rpc();

   console.log("Lender Deposit transaction was succesful");
   const lender_usdc_balance_after  = await connection.getTokenAccountBalance(lender_usdc_ata);
   const lender_lp_balace_after = await connection.getTokenAccountBalance(lender_lp_ata);
    
   log_state("Lender USDC Balance After Deposit: ", lender_usdc_balance_after.value.amount);
   log_state("Lender LP State After Balance: ", lender_lp_balace_after.value.amount);
  })

 it("Update Oracle Values", async() => {
  let price = new BN(2000 * 10**8); //$2000 * 10**8 per troy ounce which is scaled further..
  let exponent = -8; 
   const tx = await program.methods.updateOracleValues(price,exponent).accountsPartial({
    ownerOracle: admin_one.publicKey,
    lendingPool: lending_pool_pda,
    adminRegistry: admin_registry,
    systemProgram: SystemProgram.programId,
   }).signers([admin_one]).rpc();

   let oracle_state = await program.account.mockOracleState.fetch(mock_oracle);
   log_state("Gold  Price Per Troy Ounce..: ",oracle_state.price);
   log_state("Gold Exponent..: ", oracle_state.exponent);
 })
});

function log_state(str: String, state: any) { 
  console.log(`${str} : ${state}`)
}

async function generateKeypair(name: String,keypair: Keypair) {
  keypair = Keypair.generate();
  console.log(`Keypair generated for the account: ${name} with the publickey: ${keypair.publicKey}`);
  return keypair
}


async function airdrop(provider: anchor.Provider,key: PublicKey, amount: number,connection: Connection) {
  const tx_signature = await connection.requestAirdrop(
    key,
    amount*anchor.web3.LAMPORTS_PER_SOL,
  );

  const tx_confirmed = await connection.confirmTransaction({
    signature: tx_signature,
    blockhash: (await provider.connection.getLatestBlockhash()).blockhash,
    lastValidBlockHeight: (await provider.connection.getLatestBlockhash()).lastValidBlockHeight
  });

  console.log(`Airdrop confirmed to ${key}. Transaction Confirmation: ${tx_signature}.`)
  console.log(`Confirmation status: ${tx_confirmed.value.err ? 'Failed': 'Confirmed'}`)
}

async function createAta(mint_name: String,name: String,key: PublicKey,connection: Connection,payer: anchor.web3.Signer,mint: PublicKey,authority: PublicKey) {
  key = (await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mint,
    authority
  )).address

  console.log(`${mint_name} ATA Created for ${name} with the address: ${key.toBase58()}`)

  return key 
}

//minting tokens
async function mintTokens(recipient_name: String,mint_name: String,connection: Connection,payer: anchor.web3.Signer,mint: PublicKey,authority: Keypair,amount: number,destination: PublicKey) {
  const mintTx = await mintTo(
    connection,
    payer,
    mint,
    destination,
    authority,
    amount*10**6
  );

  console.log(`${amount}  ${mint_name} is minted to ${recipient_name}: ${mintTx}`);
  return mintTx

}


