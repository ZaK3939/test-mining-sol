import React, { useState } from 'react';

const InviteCodeSection: React.FC = () => {
  const [inviteCode, setInviteCode] = useState('');
  const [inviterAddress, setInviterAddress] = useState('');

  const handleCreateCode = () => {
    if (inviteCode.length !== 8) {
      alert('招待コードは8文字で入力してください');
      return;
    }
    console.log(`Creating invite code: ${inviteCode}`);
    // 実際の作成ロジックはGameControlsで実装
  };

  const handleUseCode = () => {
    if (inviteCode.length !== 8 || !inviterAddress) {
      alert('招待コードと招待者アドレスを正しく入力してください');
      return;
    }
    console.log(`Using invite code: ${inviteCode} from ${inviterAddress}`);
    // 実際の使用ロジックはGameControlsで実装
  };

  return (
    <div style={{
      background: '#f8f9fa',
      border: '1px solid #dee2e6',
      borderRadius: '10px',
      padding: '20px',
      margin: '20px 0'
    }}>
      <h3 style={{ marginTop: 0, color: '#495057' }}>🎫 招待コード処理</h3>
      <p style={{ fontSize: '14px', color: '#6c757d', marginBottom: '20px' }}>
        ハッシュベース招待システムの仕様と実装
      </p>

      {/* 招待システム概要 */}
      <div style={{
        background: '#d1ecf1',
        border: '1px solid #bee5eb',
        borderRadius: '8px',
        padding: '15px',
        marginBottom: '20px'
      }}>
        <h4 style={{ marginTop: 0, color: '#0c5460' }}>📋 システム仕様</h4>
        
        <div style={{ marginBottom: '15px' }}>
          <strong style={{ color: '#0c5460' }}>招待コード形式:</strong>
          <ul style={{ fontSize: '13px', marginTop: '5px', paddingLeft: '20px' }}>
            <li>8バイト固定長（8文字）</li>
            <li>英数字のみ使用可能</li>
            <li>ハッシュベースでプライバシー確保</li>
            <li>重複防止機能内蔵</li>
          </ul>
        </div>

        <div style={{ marginBottom: '15px' }}>
          <strong style={{ color: '#0c5460' }}>作成制限:</strong>
          <ul style={{ fontSize: '13px', marginTop: '5px', paddingLeft: '20px' }}>
            <li>運営者: 255回（事実上無制限）</li>
            <li>一般ユーザー: 5回まで</li>
            <li>一度に1つのアクティブコード</li>
          </ul>
        </div>

        <div>
          <strong style={{ color: '#0c5460' }}>紹介報酬システム:</strong>
          <ul style={{ fontSize: '13px', marginTop: '5px', paddingLeft: '20px' }}>
            <li>Level 1 (直接): 基本報酬の10%</li>
            <li>Level 2 (間接): 基本報酬の5%</li>
            <li>自動分配・蓄積システム</li>
            <li>統合請求機能</li>
          </ul>
        </div>
      </div>

      {/* Rust命令の詳細 */}
      <div style={{ marginBottom: '20px' }}>
        <h4 style={{ color: '#495057', marginBottom: '15px' }}>🔧 Rust実装詳細</h4>
        
        <div style={{ marginBottom: '15px' }}>
          <div style={{
            background: '#f8f9fa',
            border: '1px solid #e9ecef',
            borderRadius: '6px',
            padding: '10px'
          }}>
            <code style={{ fontWeight: 'bold' }}>create_invite_code</code>
            <div style={{ fontSize: '13px', marginTop: '8px' }}>
              <strong>Parameters:</strong>
              <ul style={{ marginTop: '5px', paddingLeft: '20px' }}>
                <li><code>invite_code: [u8; 8]</code> - 8バイト招待コード</li>
              </ul>
              <strong>Security:</strong>
              <ul style={{ marginTop: '5px', paddingLeft: '20px' }}>
                <li>ユーザー署名必須</li>
                <li>作成制限チェック</li>
                <li>重複防止</li>
                <li>英数字バリデーション</li>
              </ul>
            </div>
          </div>
        </div>

        <div style={{ marginBottom: '15px' }}>
          <div style={{
            background: '#f8f9fa',
            border: '1px solid #e9ecef',
            borderRadius: '6px',
            padding: '10px'
          }}>
            <code style={{ fontWeight: 'bold' }}>use_invite_code</code>
            <div style={{ fontSize: '13px', marginTop: '8px' }}>
              <strong>Parameters:</strong>
              <ul style={{ marginTop: '5px', paddingLeft: '20px' }}>
                <li><code>invite_code: [u8; 8]</code> - 使用する招待コード</li>
                <li><code>inviter_pubkey: Pubkey</code> - 招待者の公開鍵</li>
              </ul>
              <strong>Process:</strong>
              <ul style={{ marginTop: '5px', paddingLeft: '20px' }}>
                <li>招待コード検証</li>
                <li>ユーザー初期化</li>
                <li>紹介関係確立</li>
                <li>報酬システム登録</li>
              </ul>
            </div>
          </div>
        </div>
      </div>

      {/* PDA構造 */}
      <div style={{ marginBottom: '20px' }}>
        <h4 style={{ color: '#495057', marginBottom: '15px' }}>🏗️ PDA構造</h4>
        
        <div style={{
          background: '#fff3cd',
          border: '1px solid #ffeaa7',
          borderRadius: '6px',
          padding: '10px',
          fontFamily: 'Courier New, monospace',
          fontSize: '13px'
        }}>
          <div><strong>InviteCode PDA:</strong></div>
          <div>Seeds: ["invite_code", creator_pubkey, invite_code_hash]</div>
          <br />
          <div><strong>UserState PDA:</strong></div>
          <div>Seeds: ["user", user_pubkey]</div>
          <div>Includes: referrer, referral_level, invite_count</div>
        </div>
      </div>

      {/* テスト用UI（デモ） */}
      <div style={{
        background: '#e2e3e5',
        border: '1px solid #d6d8db',
        borderRadius: '8px',
        padding: '15px'
      }}>
        <h4 style={{ marginTop: 0, color: '#383d41' }}>🧪 テスト機能（デモ用）</h4>
        
        <div style={{ marginBottom: '15px' }}>
          <label style={{ display: 'block', marginBottom: '5px', fontSize: '13px', fontWeight: 'bold' }}>
            招待コード（8文字）:
          </label>
          <input
            type="text"
            value={inviteCode}
            onChange={(e) => setInviteCode(e.target.value.slice(0, 8))}
            placeholder="例: GAME2024"
            style={{
              width: '200px',
              padding: '8px',
              borderRadius: '4px',
              border: '1px solid #ced4da',
              fontSize: '14px',
              fontFamily: 'Courier New, monospace'
            }}
            maxLength={8}
          />
          <div style={{ fontSize: '11px', color: '#6c757d', marginTop: '3px' }}>
            {inviteCode.length}/8 文字 （英数字のみ）
          </div>
        </div>

        <div style={{ marginBottom: '15px' }}>
          <label style={{ display: 'block', marginBottom: '5px', fontSize: '13px', fontWeight: 'bold' }}>
            招待者アドレス（使用時のみ）:
          </label>
          <input
            type="text"
            value={inviterAddress}
            onChange={(e) => setInviterAddress(e.target.value)}
            placeholder="招待者のPublicKey"
            style={{
              width: '100%',
              padding: '8px',
              borderRadius: '4px',
              border: '1px solid #ced4da',
              fontSize: '12px',
              fontFamily: 'Courier New, monospace'
            }}
          />
        </div>

        <div style={{ display: 'flex', gap: '10px' }}>
          <button
            onClick={handleCreateCode}
            style={{
              background: '#28a745',
              color: 'white',
              border: 'none',
              padding: '8px 16px',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '13px'
            }}
            disabled={inviteCode.length !== 8}
          >
            📝 招待コード作成
          </button>
          
          <button
            onClick={handleUseCode}
            style={{
              background: '#17a2b8',
              color: 'white',
              border: 'none',
              padding: '8px 16px',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '13px'
            }}
            disabled={inviteCode.length !== 8 || !inviterAddress}
          >
            🎫 招待コード使用
          </button>
        </div>
        
        <div style={{ fontSize: '11px', color: '#6c757d', marginTop: '10px' }}>
          ※ 実際の機能はウォレット接続後のゲームコントロールから利用可能
        </div>
      </div>
    </div>
  );
};

export default InviteCodeSection;