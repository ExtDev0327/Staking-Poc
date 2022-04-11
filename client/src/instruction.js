import * as BufferLayout from "buffer-layout";
import {
  KeyPair,
  PublicKey,
  TransactionInstruction,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { Uint64Layout } from "./layout";
import BN from "bn.js";

export const createInitStakingInstruction = (
  stakeStoreKey,
  stakeListKey,
  managerKey,
  stakingProgramId
) => {
  const dataLayout = BufferLayout.struct([BufferLayout.u8("instruction")]);
  const data = Buffer.alloc(dataLayout.span);
  const encodeLength = dataLayout.encode(
    { instruction: 0 /* Initialize Instruction */ },
    data
  );
  const keys = [
    { pubkey: stakeStoreKey, isSigner: false, isWritable: true },
    { pubkey: stakeListKey, isSigner: false, isWritable: true },
    { pubkey: managerKey, isSigner: true, isWritable: false },
    { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
  ];
  return new TransactionInstruction({
    keys,
    programId: stakingProgramId,
    data: data.slice(0, encodeLength),
  });
};

export const stakeInstruction = (
  userKey,
  mintKey,
  stakeKey,
  stakeStoreKey,
  stakeListKey,
  tokenProgramId,
  amount,
  stakingProgramId
) => {
  const dataLayout = BufferLayout.struct([
    BufferLayout.u8("instruction"),
    Uint64Layout("amount"),
  ]);
  const data = Buffer.alloc(dataLayout.span);
  const encodeLength = dataLayout.encode(
    {
      instruction: 1 /* Deposit Instruction */,
      amount: Buffer.from(Uint8Array.of(...new BN(amount).toArray("le", 8))),
    },
    data
  );
  const keys = [
    { pubkey: userKey, isSigner: true, isWritable: false },
    { pubkey: mintKey, isSigner: false, isWritable: false },
    { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    { pubkey: stakeKey, isSigner: false, isWritable: true },
    { pubkey: stakeStoreKey, isSigner: false, isWritable: true },
    { pubkey: stakeListKey, isSigner: false, isWritable: true },
    { pubkey: tokenProgramId, isSigner: false, isWritable: false },
  ];
  return new TransactionInstruction({
    keys,
    programId: stakingProgramId,
    data: data.slice(0, encodeLength),
  });
};

export const reclaimInstruction = (
  userKey,
  mintKey,
  stakeStoreKey,
  stakeListKey,
  stakeKey,
  pdaStakeKey,
  tokenProgramId,
  stakingProgramId
) => {
  const dataLayout = BufferLayout.struct([BufferLayout.u8("instruction")]);
  const data = Buffer.alloc(dataLayout.span);
  dataLayout.encode({ instruction: 2 }, data);
  const keys = [
    { pubkey: userKey, isSigner: true, isWritable: false },
    { pubkey: mintKey, isSigner: false, isWritable: false },
    { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    { pubkey: stakeStoreKey, isSigner: false, isWritable: false },
    { pubkey: stakeListKey, isSigner: false, isWritable: false },
    { pubkey: stakeKey, isSigner: false, isWritable: false },
    { pubkey: pdaStakeKey, isSigner: false, isWritable: false },
    { pubkey: tokenProgramId, isSigner: false, isWritable: false },
  ];
  return new TransactionInstruction({
    keys,
    programId: stakingProgramId,
    data,
  });
};
