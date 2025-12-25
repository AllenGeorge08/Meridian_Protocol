import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MeridianProtocol } from "../target/types/meridian_protocol";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { program } from "@coral-xyz/anchor/dist/cjs/native/system";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";

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
  
  before( "Setting up accounts", async() =>{

    authority =  await generateKeypair("Authority" ,authority);
    admin_one = await generateKeypair("Admin One", admin_one);
    admin_two = await generateKeypair("Admin Two",admin_two);
    lender = await generateKeypair("Lender",lender);
    borrower = await generateKeypair("Borrower",borrower);
    liquidator =  await generateKeypair("Liquidator", liquidator);
    

    //Airdropping 
    await airdrop(authority.publicKey, 100,connection);
    await airdrop(admin_one.publicKey,100,connection);
    await airdrop(admin_two.publicKey,100,connection);
    await airdrop(lender.publicKey,100,connection);
    await airdrop(borrower.publicKey,100,connection);
    await airdrop(liquidator.publicKey,100,connection);


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
    
    lending_pool_usdc_ata = await createAta("USDC","Lending Pool",lending_pool_usdc_ata,connection, authority,mint_usdc,authority.publicKey);
    lending_pool_lp_ata = await createAta("LP","Lending Pool",lending_pool_lp_ata,connection,authority,mint_lp,authority.publicKey);

    borrower_usdc_ata = await createAta("USDC","Borrower",borrower_usdc_ata,connection, borrower,mint_usdc,borrower.publicKey);
    liquidator_usdc_ata = await createAta("USDC","Liquidator",liquidator_usdc_ata,connection,liquidator,mint_usdc,liquidator.publicKey);
    

    //Minting USDC and LP's to the necessary ATA's
    await mintTokens("Lending Pool ATA" ,"USDC",connection,authority,mint_usdc,authority,10,lending_pool_usdc_ata);
    await mintTokens("Lender USDC ATA", "USDC",connection,authority,mint_usdc,authority,10,lender_usdc_ata);
  
    
    await mintTokens("Lending POOL LP ATA", "LP",connection,authority,mint_lp, authority,10,lender_lp_ata);
  })  
  

  // it("Is initialized!", async () => {
  //   // Add your test here.
  //   const tx = await program.methods.initialize().rpc();
  //   console.log("Your transaction signature", tx);
  // });

  it("Before test runs", async() => {
    console.log("Test works...")
  })
});

async function generateKeypair(name: String,keypair: Keypair) {
  keypair = Keypair.generate();
  console.log(`Keypair generated for the account: ${name} with the publickey: ${keypair.publicKey}`);
  return keypair
}


async function airdrop(key: PublicKey, amount: number,connection: Connection) {
  const tx_signature = await connection.requestAirdrop(
    key,
    amount*anchor.web3.LAMPORTS_PER_SOL,
  );

  console.log(`Airdrop confirmed to ${key}. Transaction Signature: ${tx_signature}`)
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
    authority.publicKey,
    amount*10**6
  );

  console.log(`${amount}  ${mint_name} is minted to ${recipient_name}: ${mintTx}`);
  return mintTx

}

//Creating and minting the rwa token..
async function minting_rwa_token(){

}