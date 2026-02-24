import { useState } from 'react';
import { api } from '../api';
import { haptic, theme } from '../telegram';

const CATEGORIES = ['dining', 'travel', 'groceries', 'transport', 'shopping', 'entertainment'];
const PAYMENT_METHODS = ['contactless', 'mobile contactless', 'online'];

interface AddCardProps {
  onBack: () => void;
}

function AddCard({ onBack }: AddCardProps) {
  const [formData, setFormData] = useState({
    name: '',
    miles_per_dollar: '',
    block_size: '1',
    renewal_date: '1',
    categories: [] as string[],
    payment_categories: [] as string[],
    max_reward_limit: '',
    min_spend: '',
  });

  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState('');

  const handleCategoryToggle = (category: string) => {
    haptic.light();
    setFormData(prev => ({
      ...prev,
      categories: prev.categories.includes(category)
        ? prev.categories.filter(c => c !== category)
        : [...prev.categories, category]
    }));
  };

  const handlePaymentToggle = (method: string) => {
    haptic.light();
    setFormData(prev => ({
      ...prev,
      payment_categories: prev.payment_categories.includes(method)
        ? prev.payment_categories.filter(m => m !== method)
        : [...prev.payment_categories, method]
    }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!formData.name || !formData.miles_per_dollar) {
      setMessage('❌ Please fill in card name and miles per dollar');
      haptic.error();
      return;
    }

    setLoading(true);
    haptic.medium();

    try {
      const response = await api.addCard({
        name: formData.name,
        categories: formData.categories.length > 0 ? formData.categories : undefined,
        payment_categories: formData.payment_categories.length > 0 ? formData.payment_categories : undefined,
        miles_per_dollar: parseFloat(formData.miles_per_dollar),
        block_size: parseFloat(formData.block_size),
        renewal_date: parseInt(formData.renewal_date),
        max_reward_limit: formData.max_reward_limit ? parseFloat(formData.max_reward_limit) : undefined,
        min_spend: formData.min_spend ? parseFloat(formData.min_spend) : undefined,
      });

      setMessage(`✅ ${response.message}`);
      haptic.success();

      // Reset form
      setFormData({
        name: '',
        miles_per_dollar: '',
        block_size: '1',
        renewal_date: '1',
        categories: [],
        payment_categories: [],
        max_reward_limit: '',
        min_spend: '',
      });

      // Go back after 1.5 seconds
      setTimeout(() => onBack(), 1500);
    } catch (error) {
      console.error('Failed to add card:', error);
      setMessage('❌ Failed to add card. Please try again.');
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
        <h1 style={{ color: theme.textColor }}>Add Credit Card</h1>
      </div>

      <form onSubmit={handleSubmit} className="form">
        <div className="form-group">
          <label>Card Name *</label>
          <input
            type="text"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            placeholder="Chase Sapphire Preferred"
            style={{
              backgroundColor: theme.bgColor,
              color: theme.textColor,
              borderColor: theme.hintColor
            }}
          />
        </div>

        <div className="form-group">
          <label>Miles per Dollar *</label>
          <input
            type="number"
            step="0.1"
            value={formData.miles_per_dollar}
            onChange={(e) => setFormData({ ...formData, miles_per_dollar: e.target.value })}
            placeholder="2.0"
            style={{
              backgroundColor: theme.bgColor,
              color: theme.textColor,
              borderColor: theme.hintColor
            }}
          />
        </div>

        <div className="form-group">
          <label>Block Size (dollars)</label>
          <input
            type="number"
            step="0.1"
            value={formData.block_size}
            onChange={(e) => setFormData({ ...formData, block_size: e.target.value })}
            placeholder="1.0"
            style={{
              backgroundColor: theme.bgColor,
              color: theme.textColor,
              borderColor: theme.hintColor
            }}
          />
        </div>

        <div className="form-group">
          <label>Statement Renewal Day (1-31)</label>
          <input
            type="number"
            min="1"
            max="31"
            value={formData.renewal_date}
            onChange={(e) => setFormData({ ...formData, renewal_date: e.target.value })}
            style={{
              backgroundColor: theme.bgColor,
              color: theme.textColor,
              borderColor: theme.hintColor
            }}
          />
        </div>

        <div className="form-group">
          <label>Categories (leave empty for all)</label>
          <div className="checkbox-group">
            {CATEGORIES.map(cat => (
              <label key={cat} className="checkbox-label">
                <input
                  type="checkbox"
                  checked={formData.categories.includes(cat)}
                  onChange={() => handleCategoryToggle(cat)}
                />
                <span style={{ color: theme.textColor }}>{cat}</span>
              </label>
            ))}
          </div>
        </div>

        <div className="form-group">
          <label>Payment Methods (leave empty for all)</label>
          <div className="checkbox-group">
            {PAYMENT_METHODS.map(method => (
              <label key={method} className="checkbox-label">
                <input
                  type="checkbox"
                  checked={formData.payment_categories.includes(method)}
                  onChange={() => handlePaymentToggle(method)}
                />
                <span style={{ color: theme.textColor }}>{method}</span>
              </label>
            ))}
          </div>
        </div>

        <div className="form-group">
          <label>Max Reward Limit (optional)</label>
          <input
            type="number"
            step="0.01"
            value={formData.max_reward_limit}
            onChange={(e) => setFormData({ ...formData, max_reward_limit: e.target.value })}
            placeholder="1000.00"
            style={{
              backgroundColor: theme.bgColor,
              color: theme.textColor,
              borderColor: theme.hintColor
            }}
          />
        </div>

        <div className="form-group">
          <label>Minimum Spend (optional)</label>
          <input
            type="number"
            step="0.01"
            value={formData.min_spend}
            onChange={(e) => setFormData({ ...formData, min_spend: e.target.value })}
            placeholder="100.00"
            style={{
              backgroundColor: theme.bgColor,
              color: theme.textColor,
              borderColor: theme.hintColor
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
          {loading ? '⏳ Adding...' : '✅ Add Card'}
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

      <div className="info" style={{ color: theme.hintColor, marginTop: '30px', fontSize: '12px' }}>
        <p>* Required fields</p>
        <p>If categories/payment methods are empty, card works for all.</p>
      </div>
    </div>
  );
}

export default AddCard;
