export const calculateShares = (amount: number) => {
  const base = 1_000_000.0;
  const exponent = 1.1;
  return Math.round((amount / base) ** exponent * base);
};
