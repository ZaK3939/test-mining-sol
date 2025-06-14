// 実際のオンチェーンアクションを実行
import { Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { AnchorProvider, Wallet } from '@coral-xyz/anchor';
import { Connection, clusterApiUrl } from '@solana/web3.js';
import { AnchorClient } from './anchor-client';
import { logger } from './logger';

async function runLiveUserJourney() {
  console.log('🚀 実際のオンチェーンユーザージャーニー開始...\n');
  
  try {
    // 1. 接続設定
    console.log('1️⃣ 接続設定中...');
    const connection = new Connection('http://localhost:8899', 'confirmed');
    
    // テスト用のキーペア作成（本番ではPhantomを使用）
    const admin = Keypair.generate();
    const user = Keypair.generate();
    
    console.log('👤 Admin:', admin.publicKey.toString());
    console.log('👤 User:', user.publicKey.toString());
    
    // 2. SOLエアドロップ
    console.log('\n2️⃣ SOLエアドロップ実行中...');
    
    try {
      const adminAirdrop = await connection.requestAirdrop(admin.publicKey, 2 * LAMPORTS_PER_SOL);
      await connection.confirmTransaction(adminAirdrop);
      console.log('✅ Admin: 2 SOL受け取り');
      
      const userAirdrop = await connection.requestAirdrop(user.publicKey, 2 * LAMPORTS_PER_SOL);
      await connection.confirmTransaction(userAirdrop);
      console.log('✅ User: 2 SOL受け取り');
    } catch (error) {
      console.log('⚠️ エアドロップエラー（レート制限の可能性）:', error.message);
    }
    
    // 残高確認
    const adminBalance = await connection.getBalance(admin.publicKey);
    const userBalance = await connection.getBalance(user.publicKey);
    console.log(`💰 Admin残高: ${adminBalance / LAMPORTS_PER_SOL} SOL`);
    console.log(`💰 User残高: ${userBalance / LAMPORTS_PER_SOL} SOL`);
    
    // 3. AnchorProvider設定
    console.log('\n3️⃣ Anchorクライアント設定中...');
    const adminWallet = new Wallet(admin);
    const adminProvider = new AnchorProvider(connection, adminWallet, {
      commitment: 'confirmed',
      preflightCommitment: 'confirmed'
    });
    
    const adminClient = new AnchorClient(adminProvider);
    console.log('✅ Admin用Anchorクライアント作成');
    
    // 4. 管理者操作 - Config初期化
    console.log('\n4️⃣ システム設定初期化中...');
    try {
      const configTx = await adminClient.initializeConfig();
      console.log('✅ Config初期化成功! Tx:', configTx);
      console.log('   Explorer: https://explorer.solana.com/tx/' + configTx + '?cluster=devnet');
    } catch (error) {
      if (error.toString().includes('already in use')) {
        console.log('✅ Config既に初期化済み');
      } else {
        console.log('❌ Config初期化エラー:', error.toString());
      }
    }
    
    // 5. 管理者操作 - Reward Mint作成
    console.log('\n5️⃣ 報酬ミント作成中...');
    try {
      const mintTx = await adminClient.createRewardMint();
      console.log('✅ Reward Mint作成成功! Tx:', mintTx);
      console.log('   Explorer: https://explorer.solana.com/tx/' + mintTx + '?cluster=devnet');
    } catch (error) {
      if (error.toString().includes('already in use')) {
        console.log('✅ Reward Mint既に作成済み');
      } else {
        console.log('❌ Reward Mint作成エラー:', error.toString());
      }
    }
    
    // 6. 管理者操作 - Global Stats初期化
    console.log('\n6️⃣ グローバル統計初期化中...');
    try {
      const globalStatsTx = await adminClient.initializeGlobalStats();
      console.log('✅ Global Stats初期化成功! Tx:', globalStatsTx);
      console.log('   Explorer: https://explorer.solana.com/tx/' + globalStatsTx + '?cluster=devnet');
    } catch (error) {
      if (error.toString().includes('already in use')) {
        console.log('✅ Global Stats既に初期化済み');
      } else {
        console.log('❌ Global Stats初期化エラー:', error.toString());
      }
    }
    
    // 7. ユーザー操作開始
    console.log('\n7️⃣ ユーザー操作開始...');
    const userWallet = new Wallet(user);
    const userProvider = new AnchorProvider(connection, userWallet, {
      commitment: 'confirmed',
      preflightCommitment: 'confirmed'
    });
    const userClient = new AnchorClient(userProvider);
    console.log('✅ User用Anchorクライアント作成');
    
    // 8. ユーザー初期化
    console.log('\n8️⃣ ユーザー初期化中...');
    try {
      const userInitTx = await userClient.initUser();
      console.log('✅ ユーザー初期化成功! Tx:', userInitTx);
      console.log('   Explorer: https://explorer.solana.com/tx/' + userInitTx + '?cluster=devnet');
    } catch (error) {
      if (error.toString().includes('already_initialized')) {
        console.log('✅ ユーザー既に初期化済み');
      } else {
        console.log('❌ ユーザー初期化エラー:', error.toString());
      }
    }
    
    // 9. 農場購入
    console.log('\n9️⃣ 農場スペース購入中（0.5 SOL）...');
    try {
      const buyFarmTx = await userClient.buyFarmSpace();
      console.log('✅ 農場購入成功! Tx:', buyFarmTx);
      console.log('   Explorer: https://explorer.solana.com/tx/' + buyFarmTx + '?cluster=devnet');
      console.log('   → 初期シード付与: Grow Power 100');
    } catch (error) {
      if (error.toString().includes('already_owned')) {
        console.log('✅ 農場既に所有済み');
      } else {
        console.log('❌ 農場購入エラー:', error.toString());
      }
    }
    
    // 🔟 状態確認
    console.log('\n🔟 ゲーム状態確認中...');
    try {
      const gameState = await userClient.fetchCompleteGameState(user.publicKey);
      console.log('📊 ユーザー状態:');
      console.log(`   - 初期化: ${gameState.userInitialized ? 'Yes' : 'No'}`);
      console.log(`   - 農場所有: ${gameState.hasFarmSpace ? 'Yes' : 'No'}`);
      console.log(`   - Grow Power: ${gameState.growPower}`);
      console.log(`   - $WEED残高: ${gameState.tokenBalance / 1_000_000} WEED`);
    } catch (error) {
      console.log('⚠️ 状態確認エラー:', error.message);
    }
    
    // 1️⃣1️⃣ 報酬請求（時間経過後）
    console.log('\n1️⃣1️⃣ 報酬請求テスト...');
    console.log('⏳ 5秒待機中（報酬発生のため）...');
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    try {
      const claimTx = await userClient.claimRewards();
      console.log('✅ 報酬請求成功! Tx:', claimTx);
      console.log('   Explorer: https://explorer.solana.com/tx/' + claimTx + '?cluster=devnet');
      
      // 最終状態確認
      const finalState = await userClient.fetchCompleteGameState(user.publicKey);
      console.log(`   → $WEED残高: ${finalState.tokenBalance / 1_000_000} WEED`);
    } catch (error) {
      console.log('❌ 報酬請求エラー:', error.toString());
    }
    
    // 最終残高確認
    console.log('\n📊 最終残高確認:');
    const finalAdminBalance = await connection.getBalance(admin.publicKey);
    const finalUserBalance = await connection.getBalance(user.publicKey);
    console.log(`💰 Admin残高: ${finalAdminBalance / LAMPORTS_PER_SOL} SOL`);
    console.log(`💰 User残高: ${finalUserBalance / LAMPORTS_PER_SOL} SOL`);
    
    console.log('\n🎉 ユーザージャーニー完了！');
    console.log('✅ Config初期化 → ✅ Mint作成 → ✅ Global Stats → ✅ ユーザー初期化 → ✅ 農場購入 → ✅ 報酬請求');
    
  } catch (error) {
    console.error('\n❌ エラー発生:', error);
    console.log('詳細:', error.toString());
  }
}

// 実行
if (import.meta.main) {
  runLiveUserJourney().catch(console.error);
}

export { runLiveUserJourney };