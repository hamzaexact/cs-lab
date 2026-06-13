export function precisionForSymbol(symbol: string): number {
  if (symbol.length === 6 && symbol.endsWith('USD')) return 5;
  if (symbol === 'USDJPY' || symbol === 'EURJPY' || symbol === 'GBPJPY' || symbol === 'AUDJPY' || symbol === 'NZDJPY' || symbol === 'CADJPY' || symbol === 'CHFJPY') {
    return 3;
  }
  if (symbol === 'NQ' || symbol === 'ES') return 2;
  return 5;
}

export function formatPrice(value: number, symbol: string): string {
  const precision = precisionForSymbol(symbol);
  return value.toLocaleString('en-US', {
    minimumFractionDigits: precision,
    maximumFractionDigits: precision,
  });
}

// All chart times use New York timezone
export function formatChartTickMark(
  time: number,
  tickType: number,
  timeframe?: string,
): string {
  const date = new Date(time * 1000);

  // Format based on timeframe for intraday charts
  if (timeframe === '15M' || timeframe === '1H' || timeframe === '4H') {
    const hour = date.toLocaleString('en-US', { hour: '2-digit', hour12: false, timeZone: 'America/New_York' });
    const minute = date.toLocaleString('en-US', { minute: '2-digit', timeZone: 'America/New_York' });

    // At midnight (00:00), show day name to mark day transition
    if (hour === '00') {
      const weekday = date.toLocaleString('en-US', { weekday: 'short', timeZone: 'America/New_York' });
      return `${weekday} ${hour}:${minute}`;
    }

    // Otherwise just show time
    return `${hour}:${minute}`;
  }

  if (timeframe === '1W') {
    const weekday = date.toLocaleString('en-US', { weekday: 'short', timeZone: 'America/New_York' });
    const day = date.toLocaleString('en-US', { day: 'numeric', timeZone: 'America/New_York' });
    const month = date.toLocaleString('en-US', { month: 'short', timeZone: 'America/New_York' });
    return `${weekday} ${day} ${month}`;
  }

  if (timeframe === '1D') {
    // Show short date format: DD/MM
    const day = date.toLocaleString('en-US', { day: '2-digit', timeZone: 'America/New_York' });
    const month = date.toLocaleString('en-US', { month: '2-digit', timeZone: 'America/New_York' });
    return `${day}/${month}`;
  }

  // For 1D or when timeframe not specified, use tickType
  const weekday = date.toLocaleString('en-US', { weekday: 'short', timeZone: 'America/New_York' });
  const day = date.toLocaleString('en-US', { day: 'numeric', timeZone: 'America/New_York' });
  const month = date.toLocaleString('en-US', { month: 'short', timeZone: 'America/New_York' });
  const year = date.toLocaleString('en-US', { year: 'numeric', timeZone: 'America/New_York' });

  if (tickType === 0) return year;
  if (tickType === 1) return `${month} ${year}`;
  if (tickType === 2) return `${weekday} ${day} ${month}`;

  const hour = date.toLocaleString('en-US', { hour: '2-digit', hour12: false, timeZone: 'America/New_York' });
  const minute = date.toLocaleString('en-US', { minute: '2-digit', timeZone: 'America/New_York' });
  return `${weekday} ${hour}:${minute}`;
}

export function formatChartAnchorTime(time: number): string {
  const date = new Date(time * 1000);
  const weekday = date.toLocaleString('en-US', { weekday: 'short', timeZone: 'America/New_York' });
  const day = date.toLocaleString('en-US', { day: 'numeric', timeZone: 'America/New_York' });
  const month = date.toLocaleString('en-US', { month: 'short', timeZone: 'America/New_York' });
  const year = date.toLocaleString('en-US', { year: 'numeric', timeZone: 'America/New_York' });
  const hour = date.toLocaleString('en-US', { hour: '2-digit', hour12: false, timeZone: 'America/New_York' });
  const minute = date.toLocaleString('en-US', { minute: '2-digit', timeZone: 'America/New_York' });
  return `${weekday} ${day} ${month} ${year} ${hour}:${minute}`;
}
