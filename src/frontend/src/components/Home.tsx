import { haptic, theme } from '../telegram';

type Page = 'home' | 'add-card' | 'list-cards' | 'best-card' | 'add-spending' | 'spending-history';

interface HomeProps {
  onNavigate: (page: Page) => void;
}



function Home({ onNavigate }: HomeProps) {
  const features: Array<{ id: Page; icon: string; title: string; desc: string }> = [
    { id: 'add-card', icon: '➕', title: 'Add Card', desc: 'Add a new credit card' },
    { id: 'list-cards', icon: '📇', title: 'My Cards', desc: 'View and manage your cards' },
    { id: 'best-card', icon: '🏆', title: 'Best Card', desc: 'Find the best card for a purchase' },
    { id: 'add-spending', icon: '💳', title: 'Add Spending', desc: 'Record a transaction' },
    { id: 'spending-history', icon: '📊', title: 'History', desc: 'View spending history' },
  ];

  const handleClick = (id: Page) => {
    haptic.light();
    onNavigate(id);
  };

  return (
    <div className="home-container">
      <div className="home-header">
        <h1 style={{ color: theme.textColor }}>💳 CC Miles Tracker</h1>
        <p style={{ color: theme.hintColor }}>
          Maximize your credit card rewards
        </p>
      </div>

      <div className="feature-grid">
        {features.map((feature) => (
          <button
            key={feature.id}
            className="feature-card"
            onClick={() => handleClick(feature.id)}
            style={{
              backgroundColor: theme.bgColor,
              borderColor: theme.hintColor,
            }}
          >
            <div className="feature-icon">{feature.icon}</div>
            <div className="feature-title" style={{ color: theme.textColor }}>
              {feature.title}
            </div>
            <div className="feature-desc" style={{ color: theme.hintColor }}>
              {feature.desc}
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}

export default Home;
