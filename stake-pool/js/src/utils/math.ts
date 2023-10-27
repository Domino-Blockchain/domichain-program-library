import BN from 'bn.js';
import { SATOMIS_PER_SOL } from '@solana/web3.js';

export function solToSatomis(amount: number): number {
  if (isNaN(amount)) return Number(0);
  return Number(amount * SATOMIS_PER_SOL);
}

export function satomisToSol(satomis: number | BN): number {
  if (typeof satomis === 'number') {
    return Math.abs(satomis) / SATOMIS_PER_SOL;
  }

  let signMultiplier = 1;
  if (satomis.isNeg()) {
    signMultiplier = -1;
  }

  const absSatomis = satomis.abs();
  const satomisString = absSatomis.toString(10).padStart(10, '0');
  const splitIndex = satomisString.length - 9;
  const solString = satomisString.slice(0, splitIndex) + '.' + satomisString.slice(splitIndex);
  return signMultiplier * parseFloat(solString);
}
