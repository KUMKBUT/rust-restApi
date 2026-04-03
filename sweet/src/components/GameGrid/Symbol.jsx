import React from 'react';

const Symbol = ({ data, isExploding, isPulsing, isBombExploding }) => {
  if (!data) return <div className="symbol-wrapper"></div>;

  // Приоритет анимаций
  let animationClass = 'fall';
  if (isBombExploding) animationClass = 'bomb-explode';
  else if (isExploding) animationClass = 'explode';
  else if (isPulsing) animationClass = 'scatter-pulse';

  return (
    <div className="symbol-wrapper">
      <img 
        key={data.uid}
        src={data.img} 
        className={`symbol ${animationClass}`}
        alt="symbol"
      />
      
      {data.id === 11 && data.multiplier && (
        <div className={`bomb-multiplier ${isBombExploding ? 'hide-text' : ''}`}>
          x{data.multiplier}
        </div>
      )}
    </div>
  );
};

export default Symbol;