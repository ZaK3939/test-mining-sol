// エラーハンドリング共通ユーティリティ
// アプリケーション全体で一貫したエラー処理を提供

import { logger } from '../logger';
import { ERROR_MESSAGES } from './constants';

/**
 * エラーを安全な文字列に変換
 */
export function formatError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

/**
 * エラーの種類を判定し、適切なメッセージを返す
 */
export function getErrorMessage(error: unknown): string {
  const errorMessage = formatError(error);

  // 既知のエラーパターンをマッピング
  if (
    errorMessage.includes('Attempt to debit an account but found no record of a prior credit') ||
    errorMessage.includes('insufficient funds') ||
    errorMessage.includes('Transaction simulation failed: Attempt to debit an account')
  ) {
    return ERROR_MESSAGES.INSUFFICIENT_FUNDS;
  }

  if (errorMessage.includes('not connected')) {
    return ERROR_MESSAGES.WALLET_NOT_CONNECTED;
  }

  if (errorMessage.includes('Transaction simulation failed')) {
    return `トランザクションシミュレーションに失敗しました。ウォレットの残高を確認してください。`;
  }

  // その他のエラーはそのまま返す
  return errorMessage;
}

/**
 * 共通のエラーハンドリングデコレータ
 * 非同期関数用
 */
export function withErrorHandling<T extends (...args: unknown[]) => Promise<unknown>>(
  fn: T,
  options: {
    showLoading?: (message: string) => void;
    hideLoading?: () => void;
    showError?: (message: string) => void;
    logPrefix?: string;
  } = {}
): T {
  return (async (...args: Parameters<T>): Promise<ReturnType<T>> => {
    try {
      return await fn(...args) as ReturnType<T>;
    } catch (error) {
      const errorMessage = getErrorMessage(error);

      // ログ出力
      if (options.logPrefix) {
        logger.error(`${options.logPrefix}: ${errorMessage}`);
      } else {
        logger.error(errorMessage);
      }

      // UI エラー表示
      if (options.showError) {
        options.showError(errorMessage);
      }

      // エラーを再スロー（必要に応じて）
      throw error;
    } finally {
      // ローディング状態を終了
      if (options.hideLoading) {
        options.hideLoading();
      }
    }
  }) as T;
}

/**
 * ウォレット接続チェック付きの操作ラッパー
 */
export async function requireWalletConnection<T>(
  walletState: { connected: boolean },
  operation: () => Promise<T>,
  onError?: (message: string) => void
): Promise<T> {
  if (!walletState.connected) {
    const message = ERROR_MESSAGES.WALLET_NOT_CONNECTED;
    logger.error(message);
    if (onError) {
      onError(message);
    }
    throw new Error(message);
  }

  return await operation();
}

/**
 * トランザクション実行用の共通ラッパー
 */
export async function executeTransaction<T>(
  operation: () => Promise<T>,
  options: {
    operationName: string;
    showLoading: (message: string) => void;
    hideLoading: () => void;
    showSuccess: (message: string) => void;
    showError: (message: string) => void;
    successMessage: string;
    onSuccess?: () => Promise<void>;
  }
): Promise<T> {
  try {
    options.showLoading(`${options.operationName}中...`);

    const result = await operation();

    // 成功後の処理
    if (options.onSuccess) {
      await options.onSuccess();
    }

    options.showSuccess(options.successMessage);
    return result;
  } catch (error) {
    const errorMessage = getErrorMessage(error);
    logger.error(`${options.operationName}エラー: ${errorMessage}`);
    options.showError(`${options.operationName}に失敗しました: ${errorMessage}`);
    throw error;
  } finally {
    options.hideLoading();
  }
}

/**
 * 特殊な戻り値を持つ操作用のラッパー
 * (例: 'already_initialized', 'already_owned' など)
 */
export async function executeTransactionWithSpecialReturns<T>(
  operation: () => Promise<T>,
  options: {
    operationName: string;
    showLoading: (message: string) => void;
    hideLoading: () => void;
    showSuccess: (message: string) => void;
    showError: (message: string) => void;
    successMessage: string;
    specialReturns?: Record<string, string>; // 特殊戻り値 -> メッセージのマッピング
    onSuccess?: () => Promise<void>;
  }
): Promise<T> {
  try {
    options.showLoading(`${options.operationName}中...`);

    const result = await operation();

    // 特殊戻り値の処理
    if (options.specialReturns && typeof result === 'string' && result in options.specialReturns) {
      options.showSuccess(options.specialReturns[result]);
    } else {
      options.showSuccess(options.successMessage);
    }

    // 成功後の処理
    if (options.onSuccess) {
      await options.onSuccess();
    }

    return result;
  } catch (error) {
    const errorMessage = getErrorMessage(error);
    logger.error(`${options.operationName}エラー: ${errorMessage}`);
    options.showError(`${options.operationName}に失敗しました: ${errorMessage}`);
    throw error;
  } finally {
    options.hideLoading();
  }
}
