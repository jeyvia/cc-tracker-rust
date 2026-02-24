import { useState, useEffect } from 'react';
import { api, type Card } from '../api';
import { haptic, theme } from '../telegram';

const CATEGORIES = ['dining', 'travel', 'groceries', 'transport', 'shopping', 'entertainment'];

interface AddSpendingProps {
  onBack: () => void;
}

function AddSpending({ onBack }: AddSpendingProps) {
  const [cards, setCards] = useState<Card[]>([]);
  const [formData, setFormData] = useState({
    card_id: '',
    amount: '',
    category: 'dining',
    date: new Date().toISOString().split('T')[0],
  });
  const [loading, setLoading] = useState(false);
  const [loadingCards, setLoadingCards] = useState(true);
  const [message, setMessage] = useState('');

  useEffect(() => {
    loadCards();
  }, []);

  const loadCards = async () => {
    try {
      const data = await api.listCards();
      setCards(data);
      if (data.length > 0) {
        setFormData(prev => ({ ...prev, card_id: data[0].id.toString() }));
      }
    } catch (error) {
      console.error('Failed to load cards:', error);
    } finally {
      setLoadingCards(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!formData.card_id || !formData.amount) {
      setMessage('❌ Please fill in all required fields');
      haptic.error();
      return;
    }

    setLoading(true);
    haptic.medium();

    try {
      const response = await api.addSpending({
        card_id: parseInt(formData.card_id),
        amount: parseFloat(formData.amount),
        category: formData.category,
        date: formData.date,
      });

      setMessage(`✅ ${response.message}`);
      haptic.success();

      // Reset amount
      setFormData(prev => ({ ...prev, amount: '' }));

      // Go back after 1.5 seconds
      setTimeout(() => onBack(), 1500);
    } catch (error) {
      console.error('Failed to add spending:', error);
      setMessage('❌ Failed to add spending. Please try again.');
      haptic.error();
    } finally {
      setLoading(false);
    }
  };

  if (loadingCards) {
    return (
      <div className="page-container">
        <div className="loading" style={{ color: theme.hintColor }}>
          Loading...
        </div>
      </div>
    );
  }

  if (cards.length === 0) {
    return (
      <div className="page-container">
        <div className="page-header">
          <button onClick={onBack} className="back-button" style={{ color: theme.linkColor }}>
            ← Back
          </button>
          <h1 style={{ color: theme.textColor }}>Add Spending</h1>
        </div>
        <div className="empty-state" style={{ color: theme.hintColor }}>
          <p>No cards available</p>
          <p>Please add a card first!</p>
        </div>
      </div>
    );
  }

  return (
    <div className="page-container">
      <div className="page-header">
        <button onClick={onBack} className="back-button" style={{ color: theme.linkColor }}>
          ← Back
        </button>
        <h1 style={{ color: theme.textColor }}>Add Spending</h1>
      </div>

      <form onSubmit={handleSubmit} className="form">
        <div className="form-group">
          <label>Card *</label>
          <select
            value={formData.card_id}
            onChange={(e) => setFormData({ ...formData, card_id: e.target.value })}
            style={{
              backgroundColor: theme.bgColor,
              color: theme.textColor,
              borderColor: theme.hintColor,
            }}
          >
            {cards.map((card) => (
              <option key={card.id} value={card.id}>
                {card.name}
              </option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label>Amount ($) *</label>
          <input
            type="number"
            step="0.01"
            value={formData.amount}
            onChange={(e) => setFormData({ ...formData, amount: e.target.value })}
            placeholder="50.00"
            style={{
              backgroundColor: theme.bgColor,
              color: theme.textColor,
              borderColor: theme.hintColor,
            }}
          />
        </div>

        <div className="form-group">
          <label>Category *</label>
          <select
            value={formData.category}
            onChange={(e) => setFormData({ ...formData, category: e.target.value })}
            style={{
              backgroundColor: theme.bgColor,
              color: theme.textColor,
              borderColor: theme.hintColor,
            }}
          >
            {CATEGORIES.map((cat) => (
              <option key={cat} value={cat}>
                {cat}
              </option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label>Date *</label>
          <input
            type="date"
            value={formData.date}
            onChange={(e) => setFormData({ ...formData, date: e.target.value })}
            style={{
              backgroundColor: theme.bgColor,
              color: theme.textColor,
              borderColor: theme.hintColor,
            }}
          />
        </div>

        <button
          type="submit"
          disabled={loading}
          className="submit-button"
          style={{
            backgroundColor: theme.buttonColor,
            color: theme.buttonTextColor,
          }}
        >
          {loading ? '⏳ Adding...' : '💳 Add Spending'}
        </button>
      </form>

      {message && (
        <div
          className="message"
          style={{
            padding: '12px',
            marginTop: '20px',
            borderRadius: '8px',
            backgroundColor: message.includes('✅') ? '#d4edda' : '#f8d7da',
            color: message.includes('✅') ? '#155724' : '#721c24',
          }}
        >
          {message}
        </div>
      )}
    </div>
  );
}

export default AddSpending;
