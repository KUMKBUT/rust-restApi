export const SYMBOLS = {
  SCATTER:    { id: 10, value: 100, weight: 2,  img: '/src/assets/symbols/scatter.png' },
  HEART:      { id: 9,  value: 50,  weight: 4,  img: '/src/assets/symbols/heart.png' },
  PURPLE_GEM: { id: 8,  value: 25,  weight: 6,  img: '/src/assets/symbols/purple.png' },
  GREEN_GEM:  { id: 7,  value: 15,  weight: 8,  img: '/src/assets/symbols/green.png' },
  BLUE_GEM:   { id: 6,  value: 12,  weight: 10, img: '/src/assets/symbols/blue.png' },
  APPLE:      { id: 5,  value: 10,  weight: 12, img: '/src/assets/symbols/apple.png' },
  WATERMELON: { id: 3,  value: 5,   weight: 16, img: '/src/assets/symbols/watermelon.png' },
  BOMB:       { id: 11, value: 0,   weight: 3,  img: '/src/assets/symbols/bomb.png' }, // Исключим из обычного пула
  GRAPE:      { id: 2,  value: 4,   weight: 18, img: '/src/assets/symbols/grape.png' },
  BANANA:     { id: 1,  value: 2,   weight: 20, img: '/src/assets/symbols/banana.png' },
};

export const GRID_COLS = 6;
export const GRID_ROWS = 5;
export const WIN_THRESHOLD = 8;
export const SCATTER_ID = 10;
export const BOMB_ID = 11;

let _standardPool = null;

export const getRandomMultiplier = () => {
  const r = Math.random() * 100;
  if (r < 60) return Math.floor(Math.random() * 4) + 2;
  if (r < 90) return [10, 15, 20, 25][Math.floor(Math.random() * 4)];
  return Math.random() > 0.5 ? 50 : 100;
};

const getStandardPool = () => {
  if (!_standardPool) {
    _standardPool = [];
    Object.values(SYMBOLS).forEach(symbol => {
      if (symbol.id !== BOMB_ID) {
        for (let i = 0; i < symbol.weight; i++) {
          _standardPool.push(symbol);
        }
      }
    });
  }
  return _standardPool;
};

export const getRandomSymbol = (isBonus = false) => {
  if (isBonus && Math.random() < 0.02) {
    return { 
      id: BOMB_ID, 
      value: 0, 
      img: '/src/assets/symbols/bomb.png',
      uid: Math.random().toString(36).substring(2, 9),
      multiplier: getRandomMultiplier() 
    };
  }

  const pool = getStandardPool();
  const randomIndex = Math.floor(Math.random() * pool.length);
  return { 
    ...pool[randomIndex], 
    uid: Math.random().toString(36).substring(2, 9) 
  };
};

export const calculateWins = (grid) => {
  const flatGrid = grid.flat();
  const counts = {};
  
  flatGrid.forEach(cell => {
    if (!cell || cell.id === BOMB_ID) return;
    counts[cell.id] = (counts[cell.id] || 0) + 1;
  });

  const winningIds = Object.keys(counts)
    .filter(id => counts[id] >= WIN_THRESHOLD)
    .map(Number);

  return winningIds;
};