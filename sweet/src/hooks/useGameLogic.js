import { useState, useCallback, useEffect } from 'react';

// Убедись, что порт совпадает с тем, что указан в твоем .env на Rust
const API_URL = 'http://localhost:3000/api';

export const useGameLogic = () => {
  // Базовые состояния игры
  const [grid, setGrid] = useState([]);
  const [isSpinning, setIsSpinning] = useState(false);
  const [balance, setBalance] = useState(10000);
  const [bet, setBet] = useState(100);
  const [displayWin, setDisplayWin] = useState(0);
  const [freeSpins, setFreeSpins] = useState(0);
  const [isBonusGame, setIsBonusGame] = useState(false);
  
  // Состояния для анимаций (используются в App.jsx и Symbol.jsx)
  const [explodingSymbols, setExplodingSymbols] = useState([]);
  const [pulsingScatters, setPulsingScatters] = useState([]);
  const [explodingBombs, setExplodingBombs] = useState([]);
  const [currentMultiplier, setCurrentMultiplier] = useState(0);

  // Изменение ставки
  const changeBet = (amount) => {
    if (isSpinning || isBonusGame) return;
    setBet(prev => Math.min(Math.max(prev + amount, 10), 100000));
  };

  // Основная функция спина (и для обычного хода, и для покупки бонуса)
  const spin = async (isBuy = false) => {
    if (isSpinning) return;

    // Проверка баланса перед запросом (локальная)
    if (isBuy && balance < bet * 100) {
      alert("Недостаточно средств для покупки бонуса!");
      return;
    }

    setIsSpinning(true);
    setExplodingSymbols([]);
    setExplodingBombs([]);
    setDisplayWin(0);

    try {
      const endpoint = isBuy ? '/buy-bonus' : '/spin';
      const response = await fetch(`${API_URL}${endpoint}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ bet })
      });

      if (!response.ok) throw new Error('Ошибка сервера');

      const data = await response.json();

      // 1. Установка начальной сетки от бэкенда
      setGrid(data.initial_grid);

      // Снимаем ставку (если это покупка — снимаем х100)
      setBalance(prev => prev - (isBuy ? bet * 100 : bet));

      // 2. Анимация скаттеров (если их 4 и более)
      const scatters = data.initial_grid.flat().filter(c => c.id === 10);
      if (scatters.length >= 4) {
        setPulsingScatters(scatters.map(s => s.uid));
        await new Promise(res => setTimeout(res, 1500));
        setPulsingScatters([]);
      }

      // 3. Проигрывание каскадов (лавины)
      for (const step of data.cascades) {
        // Выделяем символы, которые взорвутся на этом шаге
        setExplodingSymbols(step.winning_ids);
        
        // Проверяем наличие бомб для анимации explodingBombs
        const bombsInStep = step.grid.flat().filter(c => c.id === 11).map(b => b.uid);
        setExplodingBombs(bombsInStep);

        // Ждем анимацию взрыва
        await new Promise(res => setTimeout(res, 1000));
        
        // Обновляем сетку на состояние после падения новых символов
        setGrid(step.grid);
        setExplodingSymbols([]);
        setExplodingBombs([]);
        
        // Небольшая пауза перед следующим каскадом
        await new Promise(res => setTimeout(res, 500));
      }

      // 4. Финализация результатов раунда
      const totalWinValue = parseFloat(data.total_win);
      if (totalWinValue > 0) {
        setDisplayWin(totalWinValue);
        setBalance(prev => prev + totalWinValue);
        // Множитель обновляем в конце всех каскадов
        setCurrentMultiplier(data.total_multiplier);
      } else {
        setCurrentMultiplier(0);
      }

      // Если выиграны фриспины
      if (data.free_spins_won > 0) {
        setIsBonusGame(true);
        setFreeSpins(prev => prev + data.free_spins_won);
      }

    } catch (error) {
      console.error("Ошибка при выполнении запроса:", error);
      alert("Не удалось связаться с сервером Rust");
    } finally {
      setIsSpinning(false);
    }
  };

  // Автоматический запуск фриспинов
  useEffect(() => {
    if (isBonusGame && freeSpins > 0 && !isSpinning) {
      const timer = setTimeout(() => {
        setFreeSpins(prev => prev - 1);
        spin(false);
      }, 2000);
      return () => clearTimeout(timer);
    } else if (isBonusGame && freeSpins === 0 && !isSpinning) {
      // Завершение бонусной игры
      setIsBonusGame(false);
      setCurrentMultiplier(0);
    }
  }, [isBonusGame, freeSpins, isSpinning]);

  useEffect(() => {
  const validIds = [1, 2, 3, 5, 6, 7, 8, 9]; // ID из твоего SYMBOL_MAP
  const initialPlaceholder = Array.from({ length: 5 }, () =>
    Array.from({ length: 6 }, () => {
      const randomId = validIds[Math.floor(Math.random() * validIds.length)];
      return {
        id: randomId,
        uid: Math.random().toString(36).substring(7)
      };
    })
  );
  setGrid(initialPlaceholder);
}, []);

  // Функции-обертки для кнопок
  const handleSpin = () => spin(false);
  const buyBonus = () => spin(true);

  return {
    grid,
    spin: handleSpin,
    buyBonus,
    isSpinning,
    balance,
    bet,
    changeBet,
    displayWin,
    freeSpins,
    isBonusGame,
    explodingSymbols,
    pulsingScatters,
    explodingBombs,
    currentMultiplier
  };
};