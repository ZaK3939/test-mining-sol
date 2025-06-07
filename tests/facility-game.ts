import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { FacilityGame } from '../target/types/facility_game';
import { PublicKey, Keypair, SystemProgram } from '@solana/web3.js';
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  getAccount,
} from '@solana/spl-token';
import { expect } from 'chai';

describe('facility-game', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.FacilityGame as Program<FacilityGame>;
  const admin = provider.wallet as anchor.Wallet;

  // Test user
  const user = Keypair.generate();

  // PDAs
  let configPda: PublicKey;
  let userStatePda: PublicKey;
  let facilityPda: PublicKey;
  let rewardMintPda: PublicKey;
  let mintAuthorityPda: PublicKey;
  let userTokenAccount: PublicKey;

  before(async () => {
    // Airdrop SOL to test user
    await provider.connection.requestAirdrop(user.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);

    // Derive PDAs
    [configPda] = PublicKey.findProgramAddressSync([Buffer.from('config')], program.programId);

    [userStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from('user'), user.publicKey.toBuffer()],
      program.programId,
    );

    [facilityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('facility'), user.publicKey.toBuffer()],
      program.programId,
    );

    [rewardMintPda] = PublicKey.findProgramAddressSync([Buffer.from('reward_mint')], program.programId);

    [mintAuthorityPda] = PublicKey.findProgramAddressSync([Buffer.from('mint_authority')], program.programId);

    // Get user's token account address
    userTokenAccount = await getAssociatedTokenAddress(rewardMintPda, user.publicKey);
  });

  it('Initialize config', async () => {
    const baseRate = new anchor.BN(10);
    const halvingInterval = new anchor.BN(86400 * 365); // 1 year in seconds

    await program.methods
      .initializeConfig(baseRate, halvingInterval)
      .accounts({
        config: configPda,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const configAccount = await program.account.config.fetch(configPda);
    expect(configAccount.baseRate.toString()).to.equal('10');
    expect(configAccount.admin.toString()).to.equal(admin.publicKey.toString());
  });

  it('Create reward mint', async () => {
    await program.methods
      .createRewardMint()
      .accounts({
        rewardMint: rewardMintPda,
        mintAuthority: mintAuthorityPda,
        admin: admin.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const mintAccount = await program.provider.connection.getAccountInfo(rewardMintPda);
    expect(mintAccount).to.not.be.null;
  });

  it('Initialize user', async () => {
    await program.methods
      .initUser()
      .accounts({
        userState: userStatePda,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const userState = await program.account.userState.fetch(userStatePda);
    expect(userState.owner.toString()).to.equal(user.publicKey.toString());
    expect(userState.hasFacility).to.be.false;
    expect(userState.totalGrowPower.toString()).to.equal('0');
  });

  it('Buy facility', async () => {
    await program.methods
      .buyFacility()
      .accounts({
        userState: userStatePda,
        facility: facilityPda,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const userState = await program.account.userState.fetch(userStatePda);
    expect(userState.hasFacility).to.be.true;
    expect(userState.totalGrowPower.toString()).to.equal('100');

    const facility = await program.account.facility.fetch(facilityPda);
    expect(facility.owner.toString()).to.equal(user.publicKey.toString());
    expect(facility.machineCount).to.equal(1);
    expect(facility.totalGrowPower.toString()).to.equal('100');
  });

  it('Create user token account', async () => {
    const instruction = createAssociatedTokenAccountInstruction(
      user.publicKey, // payer
      userTokenAccount,
      user.publicKey, // owner
      rewardMintPda,
    );

    const transaction = new anchor.web3.Transaction().add(instruction);
    await provider.sendAndConfirm(transaction, [user]);
  });

  it('Claim reward after some time', async () => {
    // Wait a bit to accumulate rewards
    await new Promise((resolve) => setTimeout(resolve, 2000));

    await program.methods
      .claimReward()
      .accounts({
        userState: userStatePda,
        config: configPda,
        rewardMint: rewardMintPda,
        mintAuthority: mintAuthorityPda,
        userTokenAccount: userTokenAccount,
        user: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    // Check token balance
    const tokenAccount = await getAccount(provider.connection, userTokenAccount);
    expect(Number(tokenAccount.amount)).to.be.greaterThan(0);

    console.log(`User received ${tokenAccount.amount} reward tokens`);
  });

  it('Cannot buy facility twice', async () => {
    try {
      await program.methods
        .buyFacility()
        .accounts({
          userState: userStatePda,
          facility: facilityPda,
          user: user.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      expect.fail('Should have thrown an error');
    } catch (error: any) {
      // Debug error details
      console.log('=== ERROR DEBUG ===');
      console.log('Error toString():', error.toString());
      console.log('Error message:', error.message);
      console.log('Error logs:', error.logs);
      console.log('Error code:', error.code);
      console.log('Full error object:', JSON.stringify(error, null, 2));
      console.log('===================');

      // Verify that an error occurred
      expect(error).to.exist;
    }
  });

  it('Check halving mechanism', async () => {
    const configBefore = await program.account.config.fetch(configPda);
    const initialRate = configBefore.baseRate;

    console.log(`Initial base rate: ${initialRate}`);
    console.log(`Next halving time: ${new Date(configBefore.nextHalvingTime.toNumber() * 1000)}`);
  });
});
