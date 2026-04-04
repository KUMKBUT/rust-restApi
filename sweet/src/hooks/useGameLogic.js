import { useState, useEffect, useRef } from 'react';

const API_URL = 'http://localhost:3000/api';

// Размер одной ячейки + gap (80px + 5px gap = 85px)
// Должно совпадать с CSS .grid: 80px ячейки, gap 5px
const CELL_SIZE = 85;
const ROWS = 5;
const COLS = 6;

export const useGameLogic = () => {
  const [grid, setGrid] = useState([]);
  const gridRef = useRef([]);

  const [isSpinning, setIsSpinning] = useState(false);
  const [balance, setBalance] = useState(10000);
  const [bet, setBet] = useState(100);
  const [displayWin, setDisplayWin] = useState(0);
  const [freeSpins, setFreeSpins] = useState(0);
  const [isBonusGame, setIsBonusGame] = useState(false);
  const [explodingSymbols, setExplodingSymbols] = useState([]);
  const [pulsingScatters, setPulsingScatters] = useState([]);
  const [explodingBombs, setExplodingBombs] = useState([]);
  const [currentMultiplier, setCurrentMultiplier] = useState(0);

  useEffect(() => {
    gridRef.current = grid;
  }, [grid]);

  const changeBet = (amount) => {
    if (isSpinning || isBonusGame) return;
    setBet(prev => Math.min(Math.max(prev + amount, 10), 100000));
  };

  const delay = (ms) => new Promise(res => setTimeout(res, ms));

  const updateGridWithPhysics = (newGridData, oldGridData) => {
    return newGridData.map((row, rIdx) =>
      row.map((cell, cIdx) => {
        let oldRowIdx = -1;
        if (oldGridData && oldGridData[cIdx]) {
          for (let r = 0; r < ROWS; r++) {
            if (oldGridData[r][cIdx]?.uid === cell.uid) {
              oldRowIdx = r;
              break;
            }
          }
        }
        let dropPx = oldRowIdx !== -1 
          ? -( (rIdx - oldRowIdx) * CELL_SIZE ) 
          : -( (ROWS + (ROWS - rIdx)) * CELL_SIZE );
        
        return { ...cell, dropPx: dropPx || 0, col: cIdx };
      })
    );
  };

  const spin = async (isBuy = false) => {
    if (isSpinning) return;
    try {
      setIsSpinning(true);
      setExplodingSymbols([]);
      setExplodingBombs([]);
      setDisplayWin(0);

      const response = await fetch(`${API_URL}/spin`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ bet, is_buy_bonus: isBuy })
      });
      const data = await response.json();
      if (isBuy) setBalance(prev => prev - (bet * 100));

      // 1. ПЕРВОЕ ПАДЕНИЕ
      const initialWithPhysics = updateGridWithPhysics(data.initial_grid, gridRef.current);
      setGrid(initialWithPhysics);
      await delay(1200); 

      let currentGridForPhysics = data.initial_grid;

      // 2. ЦИКЛ КАСКАДОВ
      for (let i = 0; i < data.cascades.length; i++) {
        const step = data.cascades[i];

        // ВЫЧИСЛЯЕМ UIDs ДЛЯ ВЗРЫВА (т.к. бэкенд шлет только IDs типов)
        const uidsToExplode = [];
        if (step.winning_ids && step.winning_ids.length > 0) {
          currentGridForPhysics.forEach(row => {
            row.forEach(cell => {
              if (cell && step.winning_ids.includes(cell.id)) {
                uidsToExplode.push(cell.uid);
              }
            });
          });
        }

        if (uidsToExplode.length > 0) {
          setExplodingSymbols(uidsToExplode);
          await delay(550); // Время анимации взрыва
        }

        const nextGridWithPhysics = updateGridWithPhysics(step.grid, currentGridForPhysics);
        setExplodingSymbols([]); 
        setGrid(nextGridWithPhysics);
        currentGridForPhysics = step.grid;
        
        await delay(900); // Время падения новых символов
      }

      setBalance(prev => prev + parseFloat(data.total_win));
      setDisplayWin(parseFloat(data.total_win));
      if (data.total_multiplier > 0) setCurrentMultiplier(data.total_multiplier);
      if (data.free_spins_won > 0) {
        setIsBonusGame(true);
        setFreeSpins(prev => prev + data.free_spins_won);
      }
    } catch (error) { console.error(error); } finally { setIsSpinning(false); }
  };

  useEffect(() => {
    if (isBonusGame && freeSpins > 0 && !isSpinning) {
      const timer = setTimeout(() => {
        setFreeSpins(prev => prev - 1);
        spin(false);
      }, 2000);
      return () => clearTimeout(timer);
    } else if (isBonusGame && freeSpins === 0 && !isSpinning) {
      setIsBonusGame(false);
      setCurrentMultiplier(0);
    }
  }, [isBonusGame, freeSpins, isSpinning]);

  // Начальный плейсхолдер
  useEffect(() => {
    const validIds = [1, 2, 3, 5, 6, 7, 8, 9];
    const placeholder = Array.from({ length: ROWS }, (_, rowIdx) =>
      Array.from({ length: COLS }, (_, colIdx) => ({
        id: validIds[Math.floor(Math.random() * validIds.length)],
        uid: Math.random().toString(36).substring(7),
        // ИСПРАВЛЕНИЕ: Выстраиваем их в колонну над сеткой
        // Самая нижняя строка (rowIdx 4) должна быть сразу над сеткой, 
        // а самая верхняя (rowIdx 0) — выше всех.
        dropPx: -((ROWS + (ROWS - rowIdx)) * CELL_SIZE), 
        col: colIdx,
      }))
    );
    setGrid(placeholder);
  }, []);

  return {
    grid,
    spin: () => spin(false),
    buyBonus: () => spin(true),
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
    currentMultiplier,
  };
};