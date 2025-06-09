// PDA (Program Derived Address) 計算ヘルパー
// 重複したPDA計算ロジックを統一管理

import { PublicKey } from '@solana/web3.js';
import { Buffer } from 'buffer';

export interface PDAs {
  userState: PublicKey;
  facility: PublicKey;
  config: PublicKey;
  rewardMint: PublicKey;
  mintAuthority: PublicKey;
  treasury: PublicKey;
}

/**
 * PDA計算ヘルパークラス
 * Solana プログラムで使用される決定論的アドレスを統一管理
 */
export class PDAHelper {
  /**
   * ユーザーに関連するすべてのPDAを計算
   * @param userPublicKey ユーザーのウォレットアドレス
   * @param programId Solanaプログラムのアドレス
   * @returns 計算されたPDAオブジェクト
   */
  static async calculatePDAs(userPublicKey: PublicKey, programId: PublicKey): Promise<PDAs> {
    const [userState] = await PublicKey.findProgramAddress(
      [Buffer.from('user'), userPublicKey.toBuffer()],
      programId
    );

    const [facility] = await PublicKey.findProgramAddress(
      [Buffer.from('facility'), userPublicKey.toBuffer()],
      programId
    );

    const [config] = await PublicKey.findProgramAddress([Buffer.from('config')], programId);

    const [rewardMint] = await PublicKey.findProgramAddress(
      [Buffer.from('reward_mint')],
      programId
    );

    const [mintAuthority] = await PublicKey.findProgramAddress(
      [Buffer.from('mint_authority')],
      programId
    );

    const [treasury] = await PublicKey.findProgramAddress(
      [Buffer.from('treasury')],
      programId
    );

    return {
      userState,
      facility,
      config,
      rewardMint,
      mintAuthority,
      treasury,
    };
  }

  /**
   * 個別のPDA計算メソッド（必要に応じて使用）
   */
  static async getUserStatePDA(
    userPublicKey: PublicKey,
    programId: PublicKey
  ): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress(
      [Buffer.from('user'), userPublicKey.toBuffer()],
      programId
    );
  }

  static async getFacilityPDA(
    userPublicKey: PublicKey,
    programId: PublicKey
  ): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress(
      [Buffer.from('facility'), userPublicKey.toBuffer()],
      programId
    );
  }

  static async getConfigPDA(programId: PublicKey): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress([Buffer.from('config')], programId);
  }

  static async getRewardMintPDA(programId: PublicKey): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress([Buffer.from('reward_mint')], programId);
  }

  static async getMintAuthorityPDA(programId: PublicKey): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress([Buffer.from('mint_authority')], programId);
  }
}
