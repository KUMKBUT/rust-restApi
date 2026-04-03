import React from 'react';
import { useGameLogic } from './hooks/useGameLogic';
import Symbol from './components/GameGrid/Symbol';
import './index.css';

const SYMBOL_MAP = {
  1:  { name: 'banana',     img: '/symbols/banana.png' },
  2:  { name: 'grape',      img: '/symbols/grape.png' },
  3:  { name: 'watermelon', img: '/symbols/watermelon.png' },
  5:  { name: 'apple',      img: '/symbols/apple.png' },
  6:  { name: 'blue_gem',   img: '/symbols/blue.png' },
  7:  { name: 'green_gem',  img: '/symbols/green.png' },
  8:  { name: 'purple_gem', img: '/symbols/purple.png' },
  9:  { name: 'heart',      img: '/symbols/heart.png' },
  10: { name: 'scatter',    img: '/symbols/scatter.png' },
  11: { name: 'bomb',       img: '/symbols/bomb.png' },
};

function App() {
  const {
    grid, spin, isSpinning, balance, bet, changeBet, buyBonus,
    displayWin, freeSpins, isBonusGame, 
    explodingSymbols, pulsingScatters, explodingBombs, currentMultiplier
  } = useGameLogic();

  return (
    <div className={`game-container ${isBonusGame ? 'bonus-mode' : ''}`}>
      
      {/* 1. ВЕРХНЯЯ ЧАСТЬ: Статус фриспинов */}
      <div className="top-status-bar">
        {isBonusGame ? (
          <div className="bonus-badge">
            FREE SPINS: <span>{freeSpins}</span>
          </div>
        ) : (
          <div className="logo-text">SWEET BONANZA</div>
        )}
      </div>

      {/* 2. ИГРОВОЕ ПОЛЕ */}
      <div className="grid-wrapper">
        <div className="grid">
          {grid.map((row, rowIndex) => (
            <React.Fragment key={rowIndex}>
              {row.map((cell, colIndex) => {
                // Добавляем проверку на существование cell
                if (!cell) return null; 

                // Безопасно получаем данные из мапы
                const symbolData = SYMBOL_MAP[cell.id] || SYMBOL_MAP[1];

                const isExploding = Boolean(explodingSymbols.includes(cell.id));
                const isPulsing = Boolean(pulsingScatters.includes(cell.uid));
                const isBombExploding = Boolean(explodingBombs.includes(cell.uid));

                return (
                  <Symbol
                    key={cell.uid || `${rowIndex}-${colIndex}`}
                    data={{
                      ...cell,
                      img: symbolData.img,
                      name: symbolData.name
                    }}
                    isExploding={isExploding}
                    isPulsing={isPulsing}
                    isBombExploding={isBombExploding}
                  />
                );
              })}
            </React.Fragment>
          ))}
        </div>
      </div>

      {/* 3. НОВАЯ ПАНЕЛЬ МЕЖДУ СЕТКОЙ И КНОПКАМИ */}
      <div className="info-display-layer">
        {/* Блок Множителя (только в бонусе и когда есть X) */}
        {isBonusGame && (
          <div className={`multiplier-box ${currentMultiplier > 0 ? 'active' : ''}`}>
            <span className="mult-label">TOTAL MULTIPLIER</span>
            <span className="mult-value">x{currentMultiplier || 0}</span>
          </div>
        )}

        {/* Блок Выигрыша */}
        <div className={`win-box ${displayWin > 0 ? 'active' : ''}`}>
          <span className="win-label">LAST WIN</span>
          <span className="win-amount">${displayWin.toLocaleString()}</span>
        </div>
      </div>

      {/* 4. ПАНЕЛЬ УПРАВЛЕНИЯ */}
      <div className="controls-container">
        {!isBonusGame && (
          <button className="buy-bonus-btn" onClick={buyBonus} disabled={isSpinning || balance < bet * 100}>
            BUY BONUS <br/><span className="price">${bet * 100}</span>
          </button>
        )}

        <div className="ui-main-panel">
          <div className="bet-selector">
            <button onClick={() => changeBet(-10)} disabled={isSpinning || isBonusGame}>-</button>
            <div className="bet-info">
              <span className="label">BET</span>
              <span className="value">${bet}</span>
            </div>
            <button onClick={() => changeBet(10)} disabled={isSpinning || isBonusGame}>+</button>
          </div>

          <div className="balance-info">
            <span className="label">BALANCE</span>
            <span className="value"> ${balance.toLocaleString()}</span>
          </div>
        </div>

        <button
          className={`spin-btn-large ${isSpinning ? 'spinning' : ''}`}
          onClick={() => spin()}
          disabled={isSpinning && freeSpins === 0}
        >
          {isBonusGame ? (
            <div className="spin-label">BONUS <span>{freeSpins}</span></div>
          ) : (
            isSpinning ? '...' : 'SPIN'
          )}
        </button>
      </div>
    </div>
  );
}

export default App;