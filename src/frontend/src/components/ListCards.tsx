import { useState, useEffect } from 'react';
import { api, type Card } from '../api';
import { haptic, theme } from '../telegram';

interface ListCardsProps {
  onBack: () => void;
}

function ListCards({ onBack }: ListCardsProps) {
  const [cards, setCards] = useState<Card[]>([]);
  const [loading, setLoading] = useState(true);
  const [message, setMessage] = useState('');

  useEffect(() => {
    loadCards();
  }, []);

  const loadCards = async () => {
    setLoading(true);
    try {
      const data = await api.listCards();
      setCards(data);
    } catch (error) {
      console.error('Failed to load cards:', error);
      setMessage('❌ Failed to load cards');
      haptic.error();
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: number, name: string) => {
    if (!confirm(`Delete "${name}"?`)) {
      return;
    }

    haptic.medium();
    try {
      await api.deleteCard(id);
      setMessage(`✅ Deleted "${name}"`);
      haptic.success();
      // Reload cards
      loadCards();
      // Clear message after 2 seconds
      setTimeout(() => setMessage(''), 2000);
    } catch (error) {
      console.error('Failed to delete card:', error);
      setMessage('❌ Failed to delete card');
      haptic.error();
    }
  };

  const parseCategories = (jsonStr: string): string[] => {
    try {
      return JSON.parse(jsonStr);
    } catch {
      return [];
    }
  };

  return (
    <div className="page-container">
      <div className="page-header">
        <button onClick={onBack} className="back-button" style={{ color: theme.linkColor }}>
          ← Back
        </button>
        <h1 style={{ color: theme.textColor }}>My Cards</h1>
      </div>

      {loading ? (
        <div className="loading" style={{ color: theme.hintColor }}>
          Loading cards...
        </div>
      ) : cards.length === 0 ? (
        <div className="empty-state" style={{ color: theme.hintColor }}>
          <p>No cards yet</p>
          <p>Add your first card to get started!</p>
        </div>
      ) : (
        <div className="cards-list">
          {cards.map((card) => (
            <div
              key={card.id}
              className="card-item"
              style={{
                backgroundColor: theme.bgColor,
                borderColor: theme.hintColor,
              }}
            >
              <div className="card-item-header">
                <h3 style={{ color: theme.textColor }}>{card.name}</h3>
                <button
                  onClick={() => handleDelete(card.id, card.name)}
                  className="delete-button"
                  style={{ color: '#dc3545' }}
                >
                  🗑️ Delete
                </button>
              </div>

              <div className="card-details" style={{ color: theme.textColor }}>
                <div className="detail-row">
                  <span className="detail-label" style={{ color: theme.hintColor }}>
                    Miles:
                  </span>
                  <span>{card.miles_per_dollar}x per ${card.block_size}</span>
                </div>

                {card.miles_per_dollar_foreign && (
                  <div className="detail-row">
                    <span className="detail-label" style={{ color: theme.hintColor }}>
                      Foreign:
                    </span>
                    <span>{card.miles_per_dollar_foreign}x</span>
                  </div>
                )}

                <div className="detail-row">
                  <span className="detail-label" style={{ color: theme.hintColor }}>
                    Renewal:
                  </span>
                  <span>Day {card.statement_renewal_date}</span>
                </div>

                <div className="detail-row">
                  <span className="detail-label" style={{ color: theme.hintColor }}>
                    Categories:
                  </span>
                  <span className="tags">
                    {parseCategories(card.categories).join(', ')}
                  </span>
                </div>

                <div className="detail-row">
                  <span className="detail-label" style={{ color: theme.hintColor }}>
                    Payment:
                  </span>
                  <span className="tags">
                    {parseCategories(card.payment_categories).join(', ')}
                  </span>
                </div>

                {card.max_reward_limit && (
                  <div className="detail-row">
                    <span className="detail-label" style={{ color: theme.hintColor }}>
                      Max Limit:
                    </span>
                    <span>${card.max_reward_limit.toFixed(2)}</span>
                  </div>
                )}

                {card.min_spend && (
                  <div className="detail-row">
                    <span className="detail-label" style={{ color: theme.hintColor }}>
                      Min Spend:
                    </span>
                    <span>${card.min_spend.toFixed(2)}</span>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

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

export default ListCards;
