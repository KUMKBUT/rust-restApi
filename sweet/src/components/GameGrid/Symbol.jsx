const Symbol = ({ data, isExploding, isPulsing, isBombExploding }) => {
  if (!data) return <div className="symbol-wrapper"></div>;

  let animationClass = 'fall';
  let wrapperClass = 'symbol-wrapper';

  // ВАЖНО: Если идет взрыв, добавляем класс 'exploding' обертке
  if (isBombExploding || isExploding) {
    wrapperClass += ' exploding'; // Это активирует ::after вспышку в CSS
  }

  if (isExploding) {
    animationClass = 'explode';
  } else if (isBombExploding) {
    animationClass = 'bomb-explode'; // Класс для обычной анимации уничтожения
  } else if (isPulsing) {
    animationClass = 'scatter-pulse';
  } else if (data.dropPx !== undefined && data.dropPx !== 0) {
    animationClass = 'fall';
  }

  const style = {
    '--drop-px': `${data.dropPx ?? 0}px`,
    '--col': data.col ?? 0,
  };

  return (
    <div className={wrapperClass} style={style}>
      <img
        // Добавление суффикса к ключу заставляет React пересоздать DOM-узел,
        // что гарантирует мгновенный старт анимации .explode
        key={data.uid + (isExploding ? '-boom' : '')} 
        src={data.img}
        className={`symbol ${animationClass}`}
        alt={data.name}
      />
      {data.id === 11 && data.multiplier && !isBombExploding && !isExploding && (
        <div className="bomb-multiplier">x{data.multiplier}</div>
      )}
    </div>
  );
};

export default Symbol;