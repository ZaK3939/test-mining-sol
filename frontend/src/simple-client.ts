// Simplified Anchor client - core functionality only
import { AnchorProvider, Program, BN } from '@coral-xyz/anchor';
import { Connection, PublicKey, SystemProgram } from '@solana/web3.js';
import { getAssociatedTokenAddress, createAssociatedTokenAccountInstruction } from '@solana/spl-token';
import * as CryptoJS from 'crypto-js';
import idl from './idl/facility_game.json';

export class SimpleClient {
  private program: Program;
  private provider: AnchorProvider;

  constructor(provider: AnchorProvider) {
    this.provider = provider;
    this.program = new Program(idl as any, provider);
  }

  // Calculate PDAs
  private async calculatePDAs(userPublicKey: PublicKey) {
    const [userState] = PublicKey.findProgramAddressSync(
      [Buffer.from('user'), userPublicKey.toBuffer()],
      this.program.programId
    );
    
    const [farmSpace] = PublicKey.findProgramAddressSync(
      [Buffer.from('farm_space'), userPublicKey.toBuffer()],
      this.program.programId
    );
    
    const [config] = PublicKey.findProgramAddressSync(
      [Buffer.from('config')],
      this.program.programId
    );
    
    const [rewardMint] = PublicKey.findProgramAddressSync(
      [Buffer.from('reward_mint')],
      this.program.programId
    );
    
    const [mintAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from('mint_authority')],
      this.program.programId
    );
    
    const [globalStats] = PublicKey.findProgramAddressSync(
      [Buffer.from('global_stats')],
      this.program.programId
    );

    return { userState, farmSpace, config, rewardMint, mintAuthority, globalStats };
  }

  // === ADMIN FUNCTIONS ===
  
  async initializeConfig(): Promise<string> {
    const userPublicKey = this.provider.wallet.publicKey;
    const pdas = await this.calculatePDAs(userPublicKey);
    
    const tx = await this.program.methods
      .initializeConfig(new BN(100), new BN(604800), userPublicKey, null)
      .accounts({
        config: pdas.config,
        admin: userPublicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
      
    return tx;
  }

  async createRewardMint(): Promise<string> {
    const userPublicKey = this.provider.wallet.publicKey;
    const pdas = await this.calculatePDAs(userPublicKey);
    
    const tx = await this.program.methods
      .createRewardMint()
      .accountsPartial({
        rewardMint: pdas.rewardMint,
        mintAuthority: pdas.mintAuthority,
        transferFeeConfigAuthority: pdas.mintAuthority,
        withdrawWithheldAuthority: userPublicKey,
        admin: userPublicKey,
        tokenProgram: new PublicKey('TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb'),
        systemProgram: SystemProgram.programId,
        rent: new PublicKey('SysvarRent111111111111111111111111111111111'),
      })
      .rpc();
      
    return tx;
  }

  async initializeGlobalStats(): Promise<string> {
    const userPublicKey = this.provider.wallet.publicKey;
    const pdas = await this.calculatePDAs(userPublicKey);
    
    const tx = await this.program.methods
      .initializeGlobalStats()
      .accounts({
        globalStats: pdas.globalStats,
        admin: userPublicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
      
    return tx;
  }

  // === USER FUNCTIONS ===
  
  async initUser(inviteCode?: string, inviterPublicKey?: PublicKey): Promise<string> {
    const userPublicKey = this.provider.wallet.publicKey;
    const pdas = await this.calculatePDAs(userPublicKey);
    
    // If invite code is provided, use the invite system
    if (inviteCode && inviterPublicKey) {
      return this.useInviteCode(inviteCode, inviterPublicKey);
    }
    
    // Normal user initialization without referrer
    const tx = await this.program.methods
      .initUser(null)
      .accounts({
        userState: pdas.userState,
        user: userPublicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
      
    return tx;
  }

  async useInviteCode(inviteCode: string, inviterPublicKey: PublicKey): Promise<string> {
    const userPublicKey = this.provider.wallet.publicKey;
    const pdas = await this.calculatePDAs(userPublicKey);
    
    // Convert string to 8-byte array
    const codeBytes = new Array(8).fill(0);
    for (let i = 0; i < Math.min(inviteCode.length, 8); i++) {
      codeBytes[i] = inviteCode.charCodeAt(i);
    }
    
    // Calculate SHA256 hash exactly like createInviteCode
    const salt = new Uint8Array([
      0x46, 0x41, 0x43, 0x49, 0x4c, 0x49, 0x54, 0x59,  // "FACILITY"
      0x47, 0x41, 0x4d, 0x45, 0x32, 0x30, 0x32, 0x34,  // "GAME2024"
    ]);
    
    // Prepare data for hashing: code + salt + pubkey (inviter's pubkey for verification)
    const hashData = new Uint8Array([...codeBytes, ...salt, ...inviterPublicKey.toBytes()]);
    
    // Use crypto-js to compute SHA256 hash (same as Rust hash function)
    const hashHex = CryptoJS.SHA256(CryptoJS.lib.WordArray.create(hashData)).toString();
    const hash = new Uint8Array(Buffer.from(hashHex, 'hex'));
    
    // Calculate invite account PDA
    const [inviteAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('invite_hash'), inviterPublicKey.toBuffer(), Buffer.from(hash)],
      this.program.programId
    );
    
    const tx = await this.program.methods
      .useInviteCode(codeBytes, inviterPublicKey)
      .accounts({
        inviteAccount: inviteAccount,
        userState: pdas.userState,
        config: pdas.config,
        invitee: userPublicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
      
    return tx;
  }

  async buyFarmSpace(): Promise<string> {
    const userPublicKey = this.provider.wallet.publicKey;
    const pdas = await this.calculatePDAs(userPublicKey);
    
    // Get config for treasury
    const config = await (this.program.account as any).config.fetch(pdas.config);
    
    // Initial seed PDA
    const [initialSeed] = PublicKey.findProgramAddressSync(
      [Buffer.from('seed'), userPublicKey.toBuffer(), new BN(0).toArrayLike(Buffer, 'le', 8)],
      this.program.programId
    );
    
    const tx = await this.program.methods
      .buyFarmSpace()
      .accounts({
        userState: pdas.userState,
        farmSpace: pdas.farmSpace,
        initialSeed: initialSeed,
        config: pdas.config,
        globalStats: pdas.globalStats,
        treasury: config.treasury,
        user: userPublicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
      
    return tx;
  }

  async claimRewards(): Promise<string> {
    const userPublicKey = this.provider.wallet.publicKey;
    const pdas = await this.calculatePDAs(userPublicKey);
    
    const TOKEN_2022_PROGRAM_ID = new PublicKey('TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb');
    
    // Get or create user token account for Token 2022
    const userTokenAccount = await getAssociatedTokenAddress(
      pdas.rewardMint, 
      userPublicKey,
      false,
      TOKEN_2022_PROGRAM_ID
    );
    
    // Check if token account exists
    const tokenAccountInfo = await this.provider.connection.getAccountInfo(userTokenAccount);
    
    // Build transaction accounts with all required fields
    // For users without referrers, we need to provide the accounts but they'll be ignored
    const accounts = {
      userState: pdas.userState,
      config: pdas.config,
      globalStats: pdas.globalStats,
      rewardMint: pdas.rewardMint,
      mintAuthority: pdas.mintAuthority,
      userTokenAccount: userTokenAccount,
      user: userPublicKey,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      // Optional referrer accounts - pass user's own account as placeholder
      level1ReferrerState: pdas.userState, // Will be ignored in Rust
      level1Referrer: userPublicKey,       // Will be ignored in Rust
      level2ReferrerState: pdas.userState, // Will be ignored in Rust  
      level2Referrer: userPublicKey,       // Will be ignored in Rust
    };
    
    let txBuilder = this.program.methods.claimRewardWithReferralRewards().accounts(accounts);
    
    // Add ATA creation instruction if needed
    if (!tokenAccountInfo) {
      const createATAInstruction = createAssociatedTokenAccountInstruction(
        userPublicKey, // payer
        userTokenAccount, // associated token account
        userPublicKey, // owner
        pdas.rewardMint, // mint
        TOKEN_2022_PROGRAM_ID
      );
      txBuilder = txBuilder.preInstructions([createATAInstruction]);
    }
    
    const tx = await txBuilder.rpc();
    return tx;
  }

  async getTokenBalance(userPublicKey: PublicKey): Promise<number> {
    try {
      const pdas = await this.calculatePDAs(userPublicKey);
      const TOKEN_2022_PROGRAM_ID = new PublicKey('TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb');
      const userTokenAccount = await getAssociatedTokenAddress(
        pdas.rewardMint, 
        userPublicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );
      
      const tokenAccountInfo = await this.provider.connection.getTokenAccountBalance(userTokenAccount);
      const amount = new BN(tokenAccountInfo.value.amount);
      const decimals = tokenAccountInfo.value.decimals;
      
      return amount.toNumber() / Math.pow(10, decimals);
    } catch {
      return 0;
    }
  }

  // === ADDITIONAL FUNCTIONS ===

  async createInviteCode(inviteCode: string): Promise<string> {
    const userPublicKey = this.provider.wallet.publicKey;
    const pdas = await this.calculatePDAs(userPublicKey);
    
    // Convert string to 8-byte array
    const codeBytes = new Array(8).fill(0);
    for (let i = 0; i < Math.min(inviteCode.length, 8); i++) {
      codeBytes[i] = inviteCode.charCodeAt(i);
    }
    
    // Calculate SHA256 hash exactly like Rust implementation
    const salt = new Uint8Array([
      0x46, 0x41, 0x43, 0x49, 0x4c, 0x49, 0x54, 0x59,  // "FACILITY"
      0x47, 0x41, 0x4d, 0x45, 0x32, 0x30, 0x32, 0x34,  // "GAME2024"
    ]);
    
    // Prepare data for hashing: code + salt + pubkey
    const hashData = new Uint8Array([...codeBytes, ...salt, ...userPublicKey.toBytes()]);
    
    // Use crypto-js to compute SHA256 hash (same as Rust hash function)
    const hashHex = CryptoJS.SHA256(CryptoJS.lib.WordArray.create(hashData)).toString();
    const hash = new Uint8Array(Buffer.from(hashHex, 'hex'));
    
    // Calculate invite account PDA with correct seeds
    const [inviteAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('invite_hash'), userPublicKey.toBuffer(), Buffer.from(hash)],
      this.program.programId
    );
    
    const tx = await this.program.methods
      .createInviteCode(codeBytes)
      .accounts({
        inviteAccount: inviteAccount,
        config: pdas.config,
        inviter: userPublicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
      
    return tx;
  }

  async purchaseSeedPack(quantity: number = 1): Promise<string> {
    const userPublicKey = this.provider.wallet.publicKey;
    const pdas = await this.calculatePDAs(userPublicKey);
    
    const TOKEN_2022_PROGRAM_ID = new PublicKey('TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb');
    
    // Get user token account
    const userTokenAccount = await getAssociatedTokenAddress(
      pdas.rewardMint, 
      userPublicKey,
      false,
      TOKEN_2022_PROGRAM_ID
    );
    
    // Calculate seed storage PDA
    const [seedStorage] = PublicKey.findProgramAddressSync(
      [Buffer.from('seed_storage'), userPublicKey.toBuffer()],
      this.program.programId
    );
    
    // Calculate probability table PDA
    const [probabilityTable] = PublicKey.findProgramAddressSync(
      [Buffer.from('probability_table')],
      this.program.programId
    );
    
    const tx = await this.program.methods
      .purchaseSeedPack(quantity, new BN(Math.floor(Math.random() * 1000000)), new BN(5000000))
      .accounts({
        userState: pdas.userState,
        seedStorage: seedStorage,
        userTokenAccount: userTokenAccount,
        rewardMint: pdas.rewardMint,
        config: pdas.config,
        probabilityTable: probabilityTable,
        user: userPublicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();
      
    return tx;
  }

  async viewPendingReferralRewards(): Promise<string> {
    const userPublicKey = this.provider.wallet.publicKey;
    const pdas = await this.calculatePDAs(userPublicKey);
    
    const tx = await this.program.methods
      .viewPendingReferralRewards()
      .accounts({
        userState: pdas.userState,
        user: userPublicKey,
      })
      .rpc();
      
    return tx;
  }

  async getUserState(userPublicKey: PublicKey): Promise<any> {
    try {
      const pdas = await this.calculatePDAs(userPublicKey);
      const userState = await (this.program.account as any).userState.fetch(pdas.userState);
      return userState;
    } catch {
      return null;
    }
  }

  async getFarmSpace(userPublicKey: PublicKey): Promise<any> {
    try {
      const pdas = await this.calculatePDAs(userPublicKey);
      const farmSpace = await (this.program.account as any).farmSpace.fetch(pdas.farmSpace);
      return farmSpace;
    } catch {
      return null;
    }
  }

  async getGlobalStats(): Promise<any> {
    try {
      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);
      const globalStats = await (this.program.account as any).globalStats.fetch(pdas.globalStats);
      return globalStats;
    } catch {
      return null;
    }
  }
}