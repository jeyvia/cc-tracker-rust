import { useState } from 'react';
import { theme } from './telegram';
import './App.css';

import Home from './components/Home';
import AddCard from './components/AddCard';
import ListCards from './components/ListCards';
import BestCard from './components/BestCard';
import AddSpending from './components/AddSpending';
import SpendingHistory from './components/SpendingHistory';

type Page = 'home' | 'add-card' | 'list-cards' | 'best-card' | 'add-spending' | 'spending-history';

function App() {
  const [currentPage, setCurrentPage] = useState<Page>('home');

  const renderPage = () => {
    switch (currentPage) {
      case 'home':
        return <Home onNavigate={setCurrentPage} />;
      case 'add-card':
        return <AddCard onBack={() => setCurrentPage('home')} />;
      case 'list-cards':
        return <ListCards onBack={() => setCurrentPage('home')} />;
      case 'best-card':
        return <BestCard onBack={() => setCurrentPage('home')} />;
      case 'add-spending':
        return <AddSpending onBack={() => setCurrentPage('home')} />;
      case 'spending-history':
        return <SpendingHistory onBack={() => setCurrentPage('home')} />;
      default:
        return <Home onNavigate={setCurrentPage} />;
    }
  };

  return (
    <div className="app" style={{
      backgroundColor: theme.bgColor,
      color: theme.textColor,
      minHeight: '100vh',
    }}>
      {renderPage()}
    </div>
  );
}

export default App;
