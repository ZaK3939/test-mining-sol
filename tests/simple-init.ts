// Simple initialization test
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { FarmGame } from "../target/types/farm_game";

describe("Simple System Init", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.FarmGame as Program<FarmGame>;

  it("Initialize config if needed", async () => {
    const [configPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    console.log("Config PDA:", configPDA.toString());

    try {
      // Check if config exists
      const config = await program.account.config.fetchNullable(configPDA);
      if (config) {
        console.log("✅ Config already exists");
        return;
      }

      console.log("📦 Creating config...");
      await program.methods
        .initializeConfig()
        .accounts({
          config: configPDA,
          admin: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("✅ Config created successfully");
    } catch (error) {
      console.log("Config creation result:", error);
    }
  });

  it("Create reward mint if needed", async () => {
    const [configPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    const [rewardMintPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("reward_mint")],
      program.programId
    );

    const [mintAuthorityPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint_authority")],
      program.programId
    );

    try {
      console.log("🪙 Creating reward mint...");
      await program.methods
        .createRewardMint()
        .accounts({
          config: configPDA,
          rewardMint: rewardMintPDA,
          mintAuthority: mintAuthorityPDA,
          admin: provider.wallet.publicKey,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc();

      console.log("✅ Reward mint created successfully");
    } catch (error) {
      console.log("Reward mint creation result:", error);
    }
  });
});