import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { FarmGame } from "../../target/types/farm_game";
import { 
  createMint, 
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAccount
} from "@solana/spl-token";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, Connection } from "@solana/web3.js";

export interface TestAccounts {
  admin: Keypair;
  treasury: Keypair;
  protocolReferralAddress: Keypair;
  users: Keypair[];
}

export interface TestPDAs {
  configPda: PublicKey;
  globalStatsPda: PublicKey;
  rewardMintPda: PublicKey;
  mintAuthorityPda: PublicKey;
  feePoolPda: PublicKey;
}

export interface UserTestData {
  keypair: Keypair;
  userStatePda: PublicKey;
  farmSpacePda: PublicKey;
  tokenAccount: any;
  seedStoragePda: PublicKey;
}

export class TestEnvironment {
  constructor(
    public program: Program<FarmGame>,
    public provider: anchor.AnchorProvider,
    public accounts: TestAccounts,
    public pdas: TestPDAs
  ) {}

  static async setup(): Promise<TestEnvironment> {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.FarmGame as Program<FarmGame>;

    // Initialize test accounts
    const accounts = await TestEnvironment.createTestAccounts(provider.connection);
    
    // Find PDAs
    const pdas = TestEnvironment.findPDAs(program.programId);

    return new TestEnvironment(program, provider, accounts, pdas);
  }

  private static async createTestAccounts(connection: Connection): Promise<TestAccounts> {
    const admin = Keypair.generate();
    const treasury = Keypair.generate();
    const protocolReferralAddress = Keypair.generate();
    
    // Create multiple test users for different scenarios
    const users: Keypair[] = [];
    for (let i = 0; i < 5; i++) {
      users.push(Keypair.generate());
    }

    // Airdrop SOL to test accounts
    const allAccounts = [admin, treasury, protocolReferralAddress, ...users];
    await Promise.all(
      allAccounts.map(account => 
        connection.requestAirdrop(account.publicKey, 2 * LAMPORTS_PER_SOL)
      )
    );

    // Wait for airdrops to confirm
    await new Promise(resolve => setTimeout(resolve, 2000));

    return { admin, treasury, protocolReferralAddress, users };
  }

  private static findPDAs(programId: PublicKey): TestPDAs {
    const [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      programId
    );

    const [globalStatsPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("global_stats")],
      programId
    );

    const [rewardMintPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("reward_mint")],
      programId
    );

    const [mintAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint_authority")],
      programId
    );

    const [feePoolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("fee_pool")],
      programId
    );

    return {
      configPda,
      globalStatsPda,
      rewardMintPda,
      mintAuthorityPda,
      feePoolPda,
    };
  }

  async initializeSystem(): Promise<void> {
    // Initialize system configuration
    await this.program.methods
      .initializeConfig(
        new anchor.BN(100), // base_rate
        new anchor.BN(6 * 24 * 60 * 60), // halving_interval (6 days)
        this.accounts.treasury.publicKey,
        this.accounts.protocolReferralAddress.publicKey
      )
      .accounts({
        config: this.pdas.configPda,
        admin: this.accounts.admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([this.accounts.admin])
      .rpc();

    // Initialize global stats
    await this.program.methods
      .initializeGlobalStats()
      .accounts({
        globalStats: this.pdas.globalStatsPda,
        admin: this.accounts.admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([this.accounts.admin])
      .rpc();

    // Initialize fee pool
    await this.program.methods
      .initializeFeePool(this.accounts.treasury.publicKey)
      .accounts({
        feePool: this.pdas.feePoolPda,
        admin: this.accounts.admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([this.accounts.admin])
      .rpc();

    // Create reward mint
    await this.program.methods
      .createRewardMint()
      .accounts({
        rewardMint: this.pdas.rewardMintPda,
        mintAuthority: this.pdas.mintAuthorityPda,
        metadataAccount: Keypair.generate().publicKey,
        admin: this.accounts.admin.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenMetadataProgram: new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
      })
      .signers([this.accounts.admin])
      .rpc();
  }

  async createUserTestData(userIndex: number, referrer?: PublicKey): Promise<UserTestData> {
    const keypair = this.accounts.users[userIndex];
    
    const [userStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("user"), keypair.publicKey.toBuffer()],
      this.program.programId
    );

    const [farmSpacePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("farm_space"), keypair.publicKey.toBuffer()],
      this.program.programId
    );

    const [seedStoragePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("seed_storage"), keypair.publicKey.toBuffer()],
      this.program.programId
    );

    // Initialize user
    await this.program.methods
      .initUser(referrer || null)
      .accounts({
        userState: userStatePda,
        user: keypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([keypair])
      .rpc();

    // Create token account
    const tokenAccount = await getOrCreateAssociatedTokenAccount(
      this.provider.connection,
      keypair,
      this.pdas.rewardMintPda,
      keypair.publicKey
    );

    return {
      keypair,
      userStatePda,
      farmSpacePda,
      tokenAccount,
      seedStoragePda,
    };
  }

  async buyFarmSpace(userData: UserTestData): Promise<void> {
    await this.program.methods
      .buyFarmSpace()
      .accounts({
        userState: userData.userStatePda,
        farmSpace: userData.farmSpacePda,
        globalStats: this.pdas.globalStatsPda,
        config: this.pdas.configPda,
        user: userData.keypair.publicKey,
        treasury: this.accounts.treasury.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([userData.keypair])
      .rpc();
  }

  async claimRewards(userData: UserTestData): Promise<anchor.BN> {
    const initialBalance = await this.getTokenBalance(userData.tokenAccount.address);
    
    await this.program.methods
      .claimReward()
      .accounts({
        userState: userData.userStatePda,
        config: this.pdas.configPda,
        globalStats: this.pdas.globalStatsPda,
        rewardMint: this.pdas.rewardMintPda,
        mintAuthority: this.pdas.mintAuthorityPda,
        userTokenAccount: userData.tokenAccount.address,
        user: userData.keypair.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([userData.keypair])
      .rpc();

    const finalBalance = await this.getTokenBalance(userData.tokenAccount.address);
    return new anchor.BN(finalBalance).sub(new anchor.BN(initialBalance));
  }

  async distributeReferralRewards(
    claimantData: UserTestData, 
    level1Data: UserTestData, 
    baseReward: anchor.BN
  ): Promise<void> {
    await this.program.methods
      .distributeReferralOnClaim(baseReward)
      .accounts({
        claimantState: claimantData.userStatePda,
        level1ReferrerState: level1Data.userStatePda,
        level1TokenAccount: level1Data.tokenAccount.address,
        rewardMint: this.pdas.rewardMintPda,
        mintAuthority: this.pdas.mintAuthorityPda,
        config: this.pdas.configPda,
        claimant: claimantData.keypair.publicKey,
        level1Referrer: level1Data.keypair.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([claimantData.keypair])
      .rpc();
  }

  async getTokenBalance(tokenAccountAddress: PublicKey): Promise<string> {
    const balance = await this.provider.connection.getTokenAccountBalance(tokenAccountAddress);
    return balance.value.amount;
  }

  async waitForRewards(milliseconds: number = 2000): Promise<void> {
    await new Promise(resolve => setTimeout(resolve, milliseconds));
  }

  // Utility methods for test assertions
  async assertTokenBalance(
    tokenAccountAddress: PublicKey, 
    expectedAmount: anchor.BN, 
    tolerance: anchor.BN = new anchor.BN(0)
  ): Promise<void> {
    const actualBalance = new anchor.BN(await this.getTokenBalance(tokenAccountAddress));
    const diff = actualBalance.sub(expectedAmount).abs();
    
    if (diff.gt(tolerance)) {
      throw new Error(
        `Token balance assertion failed. Expected: ${expectedAmount.toString()}, ` +
        `Actual: ${actualBalance.toString()}, Difference: ${diff.toString()}`
      );
    }
  }

  async getUserState(userStatePda: PublicKey): Promise<any> {
    return await this.program.account.userState.fetch(userStatePda);
  }

  async getFarmSpace(farmSpacePda: PublicKey): Promise<any> {
    return await this.program.account.farmSpace.fetch(farmSpacePda);
  }

  async getGlobalStats(): Promise<any> {
    return await this.program.account.globalStats.fetch(this.pdas.globalStatsPda);
  }

  async getConfig(): Promise<any> {
    return await this.program.account.config.fetch(this.pdas.configPda);
  }
}

// Common test constants
export const TEST_CONSTANTS = {
  BASE_RATE: 100,
  HALVING_INTERVAL: 6 * 24 * 60 * 60, // 6 days
  FARM_SPACE_COST: 0.5 * LAMPORTS_PER_SOL,
  LEVEL1_REFERRAL_PERCENTAGE: 10,
  LEVEL2_REFERRAL_PERCENTAGE: 5,
  SEED_PACK_COST: 300 * 1_000_000, // 300 WEED with 6 decimals
  FARM_SPACE_LEVELS: {
    1: { capacity: 4, upgradeCore: 0 },
    2: { capacity: 8, upgradeCost: 3500 * 1_000_000 },
    3: { capacity: 12, upgradeCost: 18000 * 1_000_000 },
    4: { capacity: 16, upgradeCost: 20000 * 1_000_000 },
    5: { capacity: 20, upgradeCost: 25000 * 1_000_000 },
  },
  SEED_GROW_POWERS: {
    Seed1: 100,
    Seed2: 180,
    Seed3: 420,
    Seed4: 720,
    Seed5: 1000,
    Seed6: 5000,
    Seed7: 15000,
    Seed8: 30000,
    Seed9: 60000,
  }
};