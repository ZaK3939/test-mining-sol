import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { FarmGame } from "../../target/types/farm_game";
import { 
  createMint, 
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
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
  treasuryPda: PublicKey;
  probabilityTablePda: PublicKey;
}

export interface TestUser {
  name: string;
  keypair: Keypair;
  userStatePda: PublicKey;
  farmSpacePda: PublicKey;
  seedStoragePda: PublicKey;
  inviteCodePda: PublicKey;
  seedPackPda: PublicKey;
  initialSeedPda: PublicKey;
  tokenAccount: PublicKey;
}

export class TestEnvironment {
  constructor(
    public program: Program<FarmGame>,
    public provider: anchor.AnchorProvider,
    public connection: Connection,
    public accounts: TestAccounts,
    public pdas: TestPDAs
  ) {}

  get adminKeypair(): Keypair {
    return this.accounts.admin;
  }

  static async setup(): Promise<TestEnvironment> {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.FarmGame as Program<FarmGame>;

    const accounts = await TestEnvironment.createTestAccounts(provider.connection);
    const pdas = TestEnvironment.findPDAs(program.programId, accounts);

    return new TestEnvironment(program, provider, provider.connection, accounts, pdas);
  }

  private static async createTestAccounts(connection: Connection): Promise<TestAccounts> {
    const admin = Keypair.generate();
    const treasury = Keypair.generate();
    const protocolReferralAddress = Keypair.generate();
    
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

  private static findPDAs(programId: PublicKey, accounts: TestAccounts): TestPDAs {
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

    const [probabilityTablePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("probability_table")],
      programId
    );

    return {
      configPda,
      globalStatsPda,
      rewardMintPda,
      mintAuthorityPda,
      feePoolPda,
      treasuryPda: accounts.treasury.publicKey,
      probabilityTablePda,
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
      .accountsPartial({
        config: this.pdas.configPda,
        admin: this.accounts.admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([this.accounts.admin])
      .rpc();

    // Initialize global stats
    await this.program.methods
      .initializeGlobalStats()
      .accountsPartial({
        globalStats: this.pdas.globalStatsPda,
        admin: this.accounts.admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([this.accounts.admin])
      .rpc();

    // Initialize fee pool
    await this.program.methods
      .initializeFeePool(this.accounts.treasury.publicKey)
      .accountsPartial({
        feePool: this.pdas.feePoolPda,
        admin: this.accounts.admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([this.accounts.admin])
      .rpc();

    // Create reward mint
    await this.program.methods
      .createRewardMint()
      .accountsPartial({
        rewardMint: this.pdas.rewardMintPda,
        mintAuthority: this.pdas.mintAuthorityPda,
        admin: this.accounts.admin.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([this.accounts.admin])
      .rpc();

    // Initialize probability table for VRF seed generation
    try {
      await this.program.methods
        .initializeProbabilityTable()
        .accountsPartial({
          probabilityTable: this.pdas.probabilityTablePda,
          admin: this.accounts.admin.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([this.accounts.admin])
        .rpc();
      
      console.log("✅ Probability table initialized successfully");
    } catch (error: any) {
      // Check specific error types
      if (error.message && error.message.includes("already in use")) {
        console.log("⚠️ Probability table already exists, continuing with existing table");
      } else if (error.message && error.message.includes("InvalidInstruction")) {
        console.log("⚠️ Probability table instruction not available, using fallback VRF mode");
      } else {
        console.log(`⚠️ Probability table initialization failed: ${error.message || error}`);
        console.log("   Continuing with basic VRF functionality...");
      }
    }
  }
}

// Factory function for quick test environment setup
export async function setupTestEnvironment(): Promise<TestEnvironment> {
  const testEnv = await TestEnvironment.setup();
  await testEnv.initializeSystem();
  return testEnv;
}

// Create a test user with all necessary PDAs
export async function createUser(testEnv: TestEnvironment, name: string, existingKeypair?: Keypair): Promise<TestUser> {
  const keypair = existingKeypair || Keypair.generate();
  
  // Airdrop SOL for transactions
  await testEnv.connection.confirmTransaction(
    await testEnv.connection.requestAirdrop(keypair.publicKey, 2 * LAMPORTS_PER_SOL),
    "confirmed"
  );

  // Generate PDAs
  const [userStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("user"), keypair.publicKey.toBuffer()],
    testEnv.program.programId
  );

  const [farmSpacePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("farm_space"), keypair.publicKey.toBuffer()],
    testEnv.program.programId
  );

  const [seedStoragePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("seed_storage"), keypair.publicKey.toBuffer()],
    testEnv.program.programId
  );

  const [inviteCodePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("invite_code"), keypair.publicKey.toBuffer()],
    testEnv.program.programId
  );

  const [seedPackPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("seed_pack"), keypair.publicKey.toBuffer()],
    testEnv.program.programId
  );

  const [initialSeedPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("seed"), keypair.publicKey.toBuffer(), Buffer.from([0, 0, 0, 0, 0, 0, 0, 0])], // seed_id 0
    testEnv.program.programId
  );

  // Create token account
  const tokenAccount = await getOrCreateAssociatedTokenAccount(
    testEnv.connection,
    keypair,
    testEnv.pdas.rewardMintPda,
    keypair.publicKey
  );

  return {
    name,
    keypair,
    userStatePda,
    farmSpacePda,
    seedStoragePda,
    inviteCodePda,
    seedPackPda,
    initialSeedPda,
    tokenAccount: tokenAccount.address,
  };
}