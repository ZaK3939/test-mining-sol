import React from 'react';

const EconomicsFormulas: React.FC = () => {
  const formulas = [
    {
      category: '基本報酬計算',
      name: 'calculate_base_reward',
      formula: '(elapsed_time × grow_power × base_rate) / 1000',
      description: 'ユーザーの基本報酬を計算',
      parameters: [
        'elapsed_time: u64 - 経過時間（秒）',
        'grow_power: u64 - ユーザーのGrow Power',
        'base_rate: u64 - 基本報酬レート（デフォルト: 100 WEED/秒）'
      ],
      example: {
        inputs: 'elapsed_time=3600, grow_power=100, base_rate=10',
        calculation: '(3600 × 100 × 10) / 1000 = 3600 WEED',
        note: '1時間で3600 WEED獲得（100 Grow Power、レート10の場合）'
      }
    },
    {
      category: 'ユーザーシェア計算',
      name: 'calculate_user_share_reward',
      formula: '(user_grow_power / total_grow_power) × base_rate × elapsed_time',
      description: 'グローバルプール報酬のユーザーシェアを計算',
      parameters: [
        'user_grow_power: u64 - ユーザーのGrow Power',
        'total_grow_power: u64 - 全体のGrow Power',
        'base_rate: u64 - 基本報酬レート',
        'elapsed_time: u64 - 経過時間'
      ],
      example: {
        inputs: 'user=100GP, total=1000GP, rate=10, time=3600s',
        calculation: '(100/1000) × 10 × 3600 = 3600 WEED',
        note: '全体の10%のシェアを持つユーザーが1時間で獲得'
      }
    },
    {
      category: '紹介報酬計算',
      name: 'calculate_referral_rewards',
      formula: 'Level1: base_reward × 10%, Level2: base_reward × 5%',
      description: 'レベル1とレベル2の紹介報酬を計算',
      parameters: [
        'base_reward: u64 - 基本報酬額',
        'LEVEL1_REFERRAL_PERCENTAGE: 10%',
        'LEVEL2_REFERRAL_PERCENTAGE: 5%'
      ],
      example: {
        inputs: 'base_reward=1000 WEED',
        calculation: 'L1: 1000 × 10% = 100 WEED, L2: 1000 × 5% = 50 WEED',
        note: '多段階報酬システムによる自動分配'
      }
    },
    {
      category: '半減期メカニズム',
      name: 'check_and_apply_halving',
      formula: 'new_rate = current_rate / 2 (when time >= next_halving_time)',
      description: '半減期チェックと新レート適用',
      parameters: [
        'current_time: i64 - 現在時刻',
        'next_halving_time: i64 - 次の半減期時刻',
        'current_rate: u64 - 現在の報酬レート',
        'halving_interval: i64 - 半減期間隔（デフォルト: 7日）'
      ],
      example: {
        inputs: 'current_rate=100, interval=7日',
        calculation: '7日後: 100 → 50, 14日後: 50 → 25',
        note: '定期的な報酬レート半減によるインフレ抑制'
      }
    },
    {
      category: '取引手数料計算',
      name: 'calculate_trading_fee',
      formula: 'fee = amount × 2%, transfer_amount = amount - fee',
      description: 'SPL Token 2022 Transfer Fee Extension',
      parameters: [
        'amount: u64 - 転送金額',
        'TRADING_FEE_PERCENTAGE: 2%',
        'max_fee: 1000 WEED'
      ],
      example: {
        inputs: 'amount=1000 WEED',
        calculation: 'fee: 1000 × 2% = 20 WEED, transfer: 980 WEED',
        note: '自動手数料回収、treasury宛に送金'
      }
    },
    {
      category: 'シードパック期待値',
      name: 'calculate_seed_pack_expected_value',
      formula: 'Σ(grow_power[i] × probability[i])',
      description: 'シードパックの期待Grow Power値',
      probabilities: [
        'Seed1 (100GP): 42.23%',
        'Seed2 (180GP): 24.44%',
        'Seed3 (420GP): 13.33%',
        'Seed4 (720GP): 8.33%',
        'Seed5 (1000GP): 5.56%',
        'Seed6 (5000GP): 3.33%',
        'Seed7 (15000GP): 1.33%',
        'Seed8 (30000GP): 0.89%',
        'Seed9 (60000GP): 0.56%'
      ],
      example: {
        calculation: '100×0.4223 + 180×0.2444 + ... ≈ 853 GP',
        note: 'Switchboard VRFによる暗号学的証明済み確率'
      }
    },
    {
      category: 'アップグレード効率',
      name: 'calculate_upgrade_efficiency',
      formula: 'cost_per_slot = upgrade_cost / capacity_increase',
      description: 'アップグレードの容量あたりコスト効率',
      farmLevels: [
        'Level 1→2: 容量 4→8 (+4), コスト: 1000 WEED',
        'Level 2→3: 容量 8→12 (+4), コスト: 2000 WEED',
        'Level 3→4: 容量 12→16 (+4), コスト: 3000 WEED',
        'Level 4→5: 容量 16→20 (+4), コスト: 4000 WEED'
      ],
      example: {
        calculation: 'L1→L2: 1000/4 = 250 WEED/slot',
        note: '自動アップグレード: 30, 100, 300, 500パック購入時'
      }
    },
    {
      category: 'ROI計算',
      name: 'calculate_seed_roi',
      formula: '((grow_power - expected_cost) / expected_cost) × 100',
      description: '各シードタイプの投資収益率',
      parameters: [
        'grow_power: 各シードのGrow Power値',
        'expected_cost: pack_cost / probability',
        'pack_cost: 300 WEED'
      ],
      example: {
        inputs: 'Seed1: GP=100, prob=42.23%',
        calculation: 'cost: 300/0.4223≈711, ROI: ((100-711)/711)×100≈-86%',
        note: '高レアリティシードほど正のROIの可能性'
      }
    }
  ];

  return (
    <div style={{
      background: '#f8f9fa',
      border: '1px solid #dee2e6',
      borderRadius: '10px',
      padding: '20px',
      margin: '20px 0'
    }}>
      <h3 style={{ marginTop: 0, color: '#495057' }}>📊 経済計算式</h3>
      <p style={{ fontSize: '14px', color: '#6c757d', marginBottom: '20px' }}>
        ゲーム経済の核となる計算式とその詳細（economics.rs）
      </p>
      
      <div style={{ maxHeight: '400px', overflowY: 'auto' }}>
        {formulas.map((formula, index) => (
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
            <div style={{ marginBottom: '10px' }}>
              <span
                style={{
                  background: '#17a2b8',
                  color: 'white',
                  padding: '2px 8px',
                  borderRadius: '4px',
                  fontSize: '12px',
                  marginRight: '10px'
                }}
              >
                {formula.category}
              </span>
              <code style={{
                background: '#f8f9fa',
                padding: '4px 8px',
                borderRadius: '4px',
                fontFamily: 'Courier New, monospace',
                fontSize: '13px',
                fontWeight: 'bold'
              }}>
                {formula.name}
              </code>
            </div>
            
            <div style={{
              background: '#fff3cd',
              border: '1px solid #ffeaa7',
              borderRadius: '6px',
              padding: '10px',
              margin: '10px 0',
              fontFamily: 'Courier New, monospace',
              fontSize: '14px',
              fontWeight: 'bold',
              color: '#856404'
            }}>
              {formula.formula}
            </div>
            
            <p style={{ fontSize: '14px', margin: '8px 0', color: '#495057' }}>
              {formula.description}
            </p>

            {formula.parameters && (
              <div style={{ marginTop: '10px' }}>
                <strong style={{ fontSize: '13px', color: '#6c757d' }}>Parameters:</strong>
                <ul style={{ fontSize: '12px', margin: '5px 0', paddingLeft: '20px' }}>
                  {formula.parameters.map((param, i) => (
                    <li key={i} style={{ marginBottom: '2px' }}>{param}</li>
                  ))}
                </ul>
              </div>
            )}

            {formula.probabilities && (
              <div style={{ marginTop: '10px' }}>
                <strong style={{ fontSize: '13px', color: '#6c757d' }}>Seed Probabilities:</strong>
                <div style={{ 
                  display: 'grid', 
                  gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))', 
                  gap: '5px',
                  fontSize: '11px',
                  margin: '5px 0'
                }}>
                  {formula.probabilities.map((prob, i) => (
                    <div key={i} style={{ 
                      background: '#f8f9fa',
                      padding: '3px 6px',
                      borderRadius: '3px'
                    }}>
                      {prob}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {formula.farmLevels && (
              <div style={{ marginTop: '10px' }}>
                <strong style={{ fontSize: '13px', color: '#6c757d' }}>Farm Levels:</strong>
                <ul style={{ fontSize: '12px', margin: '5px 0', paddingLeft: '20px' }}>
                  {formula.farmLevels.map((level, i) => (
                    <li key={i} style={{ marginBottom: '2px' }}>{level}</li>
                  ))}
                </ul>
              </div>
            )}

            {formula.example && (
              <div style={{
                background: '#d4edda',
                border: '1px solid #c3e6cb',
                borderRadius: '6px',
                padding: '10px',
                marginTop: '10px'
              }}>
                <strong style={{ fontSize: '13px', color: '#155724' }}>Example:</strong>
                <div style={{ fontSize: '12px', marginTop: '5px' }}>
                  <div><strong>Input:</strong> {formula.example.inputs}</div>
                  <div><strong>Calculation:</strong> {formula.example.calculation}</div>
                  {formula.example.note && (
                    <div><strong>Note:</strong> {formula.example.note}</div>
                  )}
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

export default EconomicsFormulas;