import { useState } from 'react';
import { api, type CardRecommendation } from '../api';
import { haptic, theme } from '../telegram';

const CATEGORIES = ['dining', 'travel', 'groceries', 'transport', 'shopping', 'entertainment'];
const PAYMENT_METHODS = ['contactless', 'mobile contactless', 'online'];

interface BestCardProps {
  onBack: () => void;
}

function BestCard({ onBack }: BestCardProps) {
  const [category, setCategory] = useState('dining');
  const [amount, setAmount] = useState('');
  const [paymentMethod, setPaymentMethod] = useState('contactless');
  const [recommendations, setRecommendations] = useState<CardRecommendation[]>([]);
  const [loading, setLoading] = useState(false);
  const [searched, setSearched] = useState(false);

  const handleFind = async () => {
    if (!amount || parseFloat(amount) <= 0) {
      haptic.error();
      return;
    }

    setLoading(true);
    setSearched(false);
    haptic.light();

    try {
      const results = await api.getBestCard(
        category,
        parseFloat(amount),
        paymentMethod
      );
      setRecommendations(results);
      setSearched(true);
      haptic.success();
    } catch (error) {
      console.error('Failed to find best card:', error);
      haptic.error();
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="page-container">
      <div className="page-header">
        <button onClick={onBack} className="back-button" style={{ color: theme.linkColor }}>
          ← Back
        </button>
        <h1 style={{ color: theme.textColor }}>Best Card Finder</h1>
      </div>

      <div className="form">
        <div className="form-group">
          <label>Category</label>
          <select
            value={category}
            onChange={(e) => setCategory(e.target.value)}
            style={{
              backgroundColor: theme.bgColor,
              color: theme.textColor,
              borderColor: theme.hintColor,
            }}
          >
            {CATEGORIES.map(cat => (
              <option key={cat} value={cat}>{cat}</option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label>Amount ($)</label>
          <input
            type="number"
            step="0.01"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            placeholder="50.00"
            style={{
              backgroundColor: theme.bgColor,
              color: theme.textColor,
              borderColor: theme.hintColor,
            }}
          />
        </div>

        <div className="form-group">
          <label>Payment Method</label>
          <select
            value={paymentMethod}
            onChange={(e) => setPaymentMethod(e.target.value)}
            style={{
              backgroundColor: theme.bgColor,
              color: theme.textColor,
              borderColor: theme.hintColor,
            }}
          >
            {PAYMENT_METHODS.map(method => (
              <option key={method} value={method}>{method}</option>
            ))}
          </select>
        </div>

        <button
          onClick={handleFind}
          disabled={loading || !amount}
          className="submit-button"
          style={{
            backgroundColor: theme.buttonColor,
            color: theme.buttonTextColor,
          }}
        >
          {loading ? '🔍 Finding...' : '🔍 Find Best Card'}
        </button>
      </div>

      {searched && recommendations.length === 0 && (
        <div className="empty-state" style={{ color: theme.hintColor }}>
          <p>No cards found for this category</p>
        </div>
      )}

      {recommendations.length > 0 && (
        <div className="recommendations-list">
          <h3 style={{ color: theme.textColor, marginBottom: '16px' }}>
            Recommendations
          </h3>
          {recommendations.map((rec, idx) => (
            <div
              key={idx}
              className={`recommendation-card ${rec.eligible ? 'eligible' : 'not-eligible'}`}
              style={{
                backgroundColor: theme.bgColor,
                borderColor: rec.eligible ? '#28a745' : '#ffc107',
              }}
            >
              <div className="rec-header">
                <h4 style={{ color: theme.textColor }}>{rec.card_name}</h4>
                <span className={`status-badge ${rec.eligible ? 'eligible' : 'not-eligible'}`}>
                  {rec.eligible ? '✅ Eligible' : '⚠️ Not Eligible'}
                </span>
              </div>

              <div className="rec-details" style={{ color: theme.textColor }}>
                <div className="rec-main">
                  <div className="rec-miles">
                    <span className="miles-value">{rec.miles_earned.toFixed(0)}</span>
                    <span style={{ color: theme.hintColor, fontSize: '12px' }}>miles</span>
                  </div>
                  <div className="rec-rate">
                    <span style={{ color: theme.hintColor, fontSize: '12px' }}>Rate:</span>
                    <span>{rec.effective_rate.toFixed(1)}x</span>
                  </div>
                </div>

                {rec.remaining_limit !== null && (
                  <div className="rec-info" style={{ color: theme.hintColor }}>
                    Remaining limit: ${rec.remaining_limit.toFixed(2)}
                  </div>
                )}

                <div
                  className="rec-reason"
                  style={{
                    color: rec.eligible ? '#28a745' : '#ffc107',
                    marginTop: '8px',
                    fontSize: '14px',
                  }}
                >
                  {rec.reason}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default BestCard;
