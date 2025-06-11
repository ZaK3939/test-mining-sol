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
}

export interface TestUser {
  name: string;
  keypair: Keypair;
  userStatePda: PublicKey;
  farmSpacePda: PublicKey;
  seedStoragePda: PublicKey;
  inviteCodePda: PublicKey;
  seedPackPda: PublicKey;
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

  static async setup(): Promise<TestEnvironment> {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.FarmGame as Program<FarmGame>;

    const accounts = await TestEnvironment.createTestAccounts(provider.connection);
    const pdas = TestEnvironment.findPDAs(program.programId);

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
}

// Factory function for quick test environment setup
export async function setupTestEnvironment(): Promise<TestEnvironment> {
  const testEnv = await TestEnvironment.setup();
  await testEnv.initializeSystem();
  return testEnv;
}

// Create a test user with all necessary PDAs
export async function createUser(testEnv: TestEnvironment, name: string): Promise<TestUser> {
  const keypair = Keypair.generate();
  
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
    tokenAccount: tokenAccount.address,
  };
}