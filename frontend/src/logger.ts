// ログ管理システム

import type { LogEntry } from './types';

class Logger {
  private logs: LogEntry[] = [];
  private logElement: HTMLElement | null = null;

  constructor() {
    // DOM読み込み後にログ要素を取得 (テスト環境では実行しない)
    if (typeof document !== 'undefined') {
      if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
          this.logElement = document.getElementById('logs');
        });
      } else {
        this.logElement = document.getElementById('logs');
      }
    }
  }

  private addLog(level: LogEntry['level'], message: string) {
    const entry: LogEntry = {
      timestamp: new Date(),
      level,
      message,
    };

    this.logs.push(entry);

    // コンソールにも出力
    switch (level) {
      case 'error':
        console.error(message);
        break;
      case 'warn':
        console.warn(message);
        break;
      case 'success':
        console.log(`✅ ${message}`);
        break;
      default:
        console.log(message);
    }

    // DOM要素に表示
    this.updateLogDisplay();
  }

  private updateLogDisplay() {
    if (!this.logElement) return;

    const formattedLogs = this.logs
      .slice(-50) // 最新50件のみ表示
      .map((log) => {
        const time = log.timestamp.toLocaleTimeString();
        const icon = this.getLogIcon(log.level);
        return `[${time}] ${icon} ${log.message}`;
      })
      .join('\n');

    this.logElement.textContent = formattedLogs;
    this.logElement.scrollTop = this.logElement.scrollHeight;
  }

  private getLogIcon(level: LogEntry['level']): string {
    switch (level) {
      case 'error':
        return '❌';
      case 'warn':
        return '⚠️';
      case 'success':
        return '✅';
      default:
        return '📋';
    }
  }

  info(message: string) {
    this.addLog('info', message);
  }

  warn(message: string) {
    this.addLog('warn', message);
  }

  error(message: string) {
    this.addLog('error', message);
  }

  success(message: string) {
    this.addLog('success', message);
  }

  clear() {
    this.logs = [];
    if (this.logElement) {
      this.logElement.textContent = 'ログをクリアしました...\n';
    }
  }

  getLogs(): LogEntry[] {
    return [...this.logs];
  }
}

export const logger = new Logger();
