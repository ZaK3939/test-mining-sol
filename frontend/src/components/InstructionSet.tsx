import React from 'react';

const InstructionSet: React.FC = () => {
  const instructions = [
    {
      name: 'initialize_config',
      category: 'Admin',
      description: 'システム全体の設定を初期化',
      parameters: [
        'base_rate: Option<u64> - 基本報酬レート（デフォルト: 100 WEED/秒）',
        'halving_interval: Option<i64> - 半減期間隔（デフォルト: 7日）',
        'treasury: Pubkey - 手数料収集用ウォレット',
        'protocol_referral_address: Option<Pubkey> - プロトコル紹介者'
      ],
      security: ['admin署名必須', '一度のみ実行可能（PDAの重複初期化防止）']
    },
    {
      name: 'create_reward_mint',
      category: 'Admin',
      description: '$WEEDトークンのミントアカウントを作成（SPL Token 2022 + Transfer Fee Extension）',
      parameters: [],
      features: [
        'PDAによるミント権限管理（セキュリティ向上）',
        'SPL Token 2022のTransfer Fee Extension（2%手数料）',
        'Metaplexメタデータ作成（トークン情報表示用）',
        '6桁精度での発行設定',
        '自動手数料回収（treasury宛）'
      ],
      transferFee: {
        rate: '2.00% (200 basis points)',
        maxFee: '1000 WEED',
        collector: 'treasury',
        authority: 'mint_authority PDA'
      }
    },
    {
      name: 'init_user',
      category: 'User',
      description: 'ユーザーアカウントの初期化',
      parameters: [
        'referrer: Option<Pubkey> - 紹介者のpubkey（招待コード使用時のみ）'
      ],
      creates: [
        'UserState PDA: ユーザーの進行状況',
        '初期grow_power: 0（農場購入で増加）',
        '紹介関係の記録（多段階報酬システム用）'
      ]
    },
    {
      name: 'buy_farm_space',
      category: 'Farm',
      description: '農場スペースの購入（レベル1）',
      cost: '0.5 SOL',
      execution: [
        '0.5 SOL → treasuryに送金',
        'FarmSpace PDA作成（容量4、レベル1）',
        '初期Seed1を無料で付与（100 Grow Power）',
        'グローバル統計の更新'
      ],
      constraints: ['ユーザーは1つまで', 'UserState初期化済み必須']
    },
    {
      name: 'claim_reward_with_referral_rewards',
      category: 'Rewards',
      description: '統合報酬請求（メイン関数）',
      flow: [
        '1. 半減期チェック・適用',
        '2. 農場報酬計算（比例配分）',
        '3. 蓄積された紹介報酬請求',
        '4. 新規紹介報酬分配（L1: 10%, L2: 5%）',
        '5. すべてのトークンを一括ミント・配布'
      ],
      benefits: [
        '複数のトランザクションが不要',
        'ガス効率性向上',
        'ユーザー体験の向上'
      ]
    },
    {
      name: 'purchase_seed_pack',
      category: 'Seeds',
      description: 'ミステリーシードパックの購入（Switchboard VRF統合）',
      cost: '300 $WEED + VRF手数料',
      parameters: [
        'quantity: u8 - 購入数量（1-100）',
        'user_entropy_seed: u64 - ユーザー提供の乱数シード',
        'max_vrf_fee: u64 - 支払い可能な最大VRF手数料（lamports）'
      ],
      vrfIntegration: [
        '検証可能な真正乱数による最高品質の公平性',
        'コミット・リビール方式で操作不可能',
        '第三者オラクルによる透明性保証'
      ],
      costStructure: {
        weedBurn: '300 WEED × quantity',
        vrfFee: '約0.002 SOL（2,000,000 lamports）',
        breakdown: [
          '基本取引手数料: 5,000 × 15取引 = 75,000 lamports',
          'ストレージレント: 2,400 lamports',
          'オラクル処理費: 2,000,000 lamports',
          '総計: ~2,077,400 lamports'
        ]
      }
    },
    {
      name: 'create_invite_code',
      category: 'Invite',
      description: '招待コード作成',
      parameters: [
        'invite_code: [u8; 8] - 8バイトの招待コード（英数字のみ）'
      ],
      limits: [
        '運営者: 255回（事実上無制限）',
        '一般ユーザー: 5回まで'
      ],
      security: [
        'ハッシュベースでプライバシー確保',
        '英数字のみ使用可能'
      ]
    },
    {
      name: 'use_invite_code',
      category: 'Invite',
      description: '招待コード使用',
      parameters: [
        'invite_code: [u8; 8] - 8バイト招待コード',
        'inviter_pubkey: Pubkey - 招待者の公開鍵'
      ],
      effect: '招待コードでユーザー初期化と紹介関係確立'
    }
  ];

  const categoryColors = {
    'Admin': '#dc3545',
    'User': '#28a745',
    'Farm': '#fd7e14',
    'Rewards': '#17a2b8',
    'Seeds': '#6f42c1',
    'Invite': '#20c997'
  };

  return (
    <div style={{
      background: '#f8f9fa',
      border: '1px solid #dee2e6',
      borderRadius: '10px',
      padding: '20px',
      margin: '20px 0'
    }}>
      <h3 style={{ marginTop: 0, color: '#495057' }}>🔧 Rust 命令セット</h3>
      <p style={{ fontSize: '14px', color: '#6c757d', marginBottom: '20px' }}>
        Solanaプログラムで利用可能な命令とその詳細
      </p>
      
      <div style={{ maxHeight: '400px', overflowY: 'auto' }}>
        {instructions.map((instruction, index) => (
          <div
            key={index}
            style={{
              border: '1px solid #e9ecef',
              borderRadius: '8px',
              marginBottom: '15px',
              padding: '15px',
              background: 'white'
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', marginBottom: '10px' }}>
              <span
                style={{
                  background: categoryColors[instruction.category as keyof typeof categoryColors],
                  color: 'white',
                  padding: '2px 8px',
                  borderRadius: '4px',
                  fontSize: '12px',
                  marginRight: '10px'
                }}
              >
                {instruction.category}
              </span>
              <code style={{
                background: '#f8f9fa',
                padding: '4px 8px',
                borderRadius: '4px',
                fontFamily: 'Courier New, monospace',
                fontSize: '14px',
                fontWeight: 'bold'
              }}>
                {instruction.name}
              </code>
            </div>
            
            <p style={{ fontSize: '14px', margin: '8px 0', color: '#495057' }}>
              {instruction.description}
            </p>

            {instruction.parameters && instruction.parameters.length > 0 && (
              <div style={{ marginTop: '10px' }}>
                <strong style={{ fontSize: '13px', color: '#6c757d' }}>Parameters:</strong>
                <ul style={{ fontSize: '12px', margin: '5px 0', paddingLeft: '20px' }}>
                  {instruction.parameters.map((param, i) => (
                    <li key={i} style={{ marginBottom: '2px' }}>{param}</li>
                  ))}
                </ul>
              </div>
            )}

            {instruction.features && (
              <div style={{ marginTop: '10px' }}>
                <strong style={{ fontSize: '13px', color: '#6c757d' }}>Features:</strong>
                <ul style={{ fontSize: '12px', margin: '5px 0', paddingLeft: '20px' }}>
                  {instruction.features.map((feature, i) => (
                    <li key={i} style={{ marginBottom: '2px' }}>{feature}</li>
                  ))}
                </ul>
              </div>
            )}

            {instruction.security && (
              <div style={{ marginTop: '10px' }}>
                <strong style={{ fontSize: '13px', color: '#dc3545' }}>Security:</strong>
                <ul style={{ fontSize: '12px', margin: '5px 0', paddingLeft: '20px' }}>
                  {instruction.security.map((sec, i) => (
                    <li key={i} style={{ marginBottom: '2px' }}>{sec}</li>
                  ))}
                </ul>
              </div>
            )}

            {instruction.cost && (
              <div style={{ marginTop: '10px' }}>
                <strong style={{ fontSize: '13px', color: '#fd7e14' }}>Cost: </strong>
                <span style={{ fontSize: '12px' }}>{instruction.cost}</span>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

export default InstructionSet;