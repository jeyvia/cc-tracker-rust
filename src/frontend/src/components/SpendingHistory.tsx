import { useState, useEffect } from 'react';
import { api, type Spending, type Card } from '../api';
import { haptic, theme } from '../telegram';

interface SpendingHistoryProps {
  onBack: () => void;
}

function SpendingHistory({ onBack }: SpendingHistoryProps) {
  const [spending, setSpending] = useState<Spending[]>([]);
  const [cards, setCards] = useState<Card[]>([]);
  const [selectedCardId, setSelectedCardId] = useState<number | undefined>(undefined);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, [selectedCardId]);

  const loadData = async () => {
    setLoading(true);
    try {
      const [spendingData, cardsData] = await Promise.all([
        api.listSpending(selectedCardId),
        api.listCards(),
      ]);
      setSpending(spendingData);
      setCards(cardsData);
    } catch (error) {
      console.error('Failed to load data:', error);
      haptic.error();
    } finally {
      setLoading(false);
    }
  };

  const getCardName = (cardId: number): string => {
    const card = cards.find(c => c.id === cardId);
    return card ? card.name : `Card #${cardId}`;
  };

  const totalMiles = spending.reduce((sum, s) => sum + s.miles_earned, 0);
  const totalSpent = spending.reduce((sum, s) => sum + s.amount, 0);

  return (
    <div className="page-container">
      <div className="page-header">
        <button onClick={onBack} className="back-button" style={{ color: theme.linkColor }}>
          ← Back
        </button>
        <h1 style={{ color: theme.textColor }}>Spending History</h1>
      </div>

      <div className="form-group">
        <label>Filter by Card</label>
        <select
          value={selectedCardId || ''}
          onChange={(e) => {
            haptic.light();
            setSelectedCardId(e.target.value ? parseInt(e.target.value) : undefined);
          }}
          style={{
            backgroundColor: theme.bgColor,
            color: theme.textColor,
            borderColor: theme.hintColor,
          }}
        >
          <option value="">All Cards</option>
          {cards.map((card) => (
            <option key={card.id} value={card.id}>
              {card.name}
            </option>
          ))}
        </select>
      </div>

      {loading ? (
        <div className="loading" style={{ color: theme.hintColor }}>
          Loading...
        </div>
      ) : spending.length === 0 ? (
        <div className="empty-state" style={{ color: theme.hintColor }}>
          <p>No spending records yet</p>
          <p>Start tracking your purchases!</p>
        </div>
      ) : (
        <>
          <div className="summary-cards">
            <div className="summary-card" style={{ borderColor: theme.hintColor }}>
              <div className="summary-value" style={{ color: theme.textColor }}>
                {totalMiles.toFixed(0)}
              </div>
              <div className="summary-label" style={{ color: theme.hintColor }}>
                Total Miles
              </div>
            </div>
            <div className="summary-card" style={{ borderColor: theme.hintColor }}>
              <div className="summary-value" style={{ color: theme.textColor }}>
                ${totalSpent.toFixed(2)}
              </div>
              <div className="summary-label" style={{ color: theme.hintColor }}>
                Total Spent
              </div>
            </div>
          </div>

          <div className="spending-list">
            {spending.map((item) => (
              <div
                key={item.id}
                className="spending-item"
                style={{
                  backgroundColor: theme.bgColor,
                  borderColor: theme.hintColor,
                }}
              >
                <div className="spending-header">
                  <div>
                    <div className="spending-card-name" style={{ color: theme.textColor }}>
                      {getCardName(item.card_id)}
                    </div>
                    <div className="spending-category" style={{ color: theme.hintColor }}>
                      {item.category}
                    </div>
                  </div>
                  <div className="spending-date" style={{ color: theme.hintColor }}>
                    {item.date}
                  </div>
                </div>

                <div className="spending-details">
                  <div className="spending-amount" style={{ color: theme.textColor }}>
                    ${item.amount.toFixed(2)}
                  </div>
                  <div className="spending-miles" style={{ color: '#28a745' }}>
                    +{item.miles_earned.toFixed(0)} miles
                  </div>
                </div>
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
}

export default SpendingHistory;
