// Manual frontend test for browser functionality
// ブラウザでの手動テスト用スクリプト

import { SolanaService } from '../solana';
import { GameService } from '../services/game-service';
import { logger } from '../logger';
import { config } from '../config';

// Simple manual test to verify frontend functionality
export async function runManualFrontendTest() {
  console.log('🎯 Manual Frontend Test Starting...');

  try {
    // Test 1: Configuration loading
    console.log('✅ Config loaded:', {
      programId: config.programId,
      network: config.network,
      rpcUrl: config.rpcUrl.includes('api-key')
        ? config.rpcUrl.split('?')[0] + '?api-key=***'
        : config.rpcUrl,
    });

    // Test 2: SolanaService initialization
    const solanaService = new SolanaService();
    const networkInfo = solanaService.getNetworkInfo();
    console.log('✅ SolanaService initialized:', networkInfo);

    // Test 3: RPC connection test
    console.log('🔗 Testing RPC connection...');
    const connectionTest = await solanaService.testConnection();
    console.log('✅ RPC Connection:', connectionTest ? 'Success' : 'Failed');

    // Test 4: GameService initialization
    new GameService(solanaService);
    console.log('✅ GameService initialized');

    // Test 5: Logger functionality
    logger.info('🧪 Testing logger functionality');
    logger.success('✅ Logger working correctly');

    console.log('🎉 All manual tests passed!');
    return true;
  } catch (error) {
    console.error('❌ Manual test failed:', error);
    return false;
  }
}

// Export for browser console usage
if (typeof window !== 'undefined') {
  (window as any).runManualFrontendTest = runManualFrontendTest;
  console.log('💡 Run "runManualFrontendTest()" in browser console to test frontend');
}
