import { PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import * as Layout from "./layout";
import { loadAccount } from "./util/account";
import { TOKEN_PROGRAM_ID, ZERO_TS, MAX_STAKE_COUNT } from "./constants";
import * as instructions from "./instruction";
import { sendAndConfirmTransaction } from "./util/send-and-confirm-transaction";

export class StakeStore {
  /**
   * @private
   */
  // connection;
  stakingProgramId;
  tokenProgramId;
  stakeStoreKey;
  stakeListKey;
  managerKey;

  constructor(
    connection,
    stakingProgramId,
    stakeStoreKey,
    stakeListKey,
    tokenProgramId,
    managerKey
  ) {
    // this.connection = connection;
    this.stakingProgramId = stakingProgramId;
    this.tokenProgramId = tokenProgramId;
    this.stakeStoreKey = stakeStoreKey;
    this.stakeListKey = stakeListKey;
    this.managerKey = managerKey;
  }

  static async getMinBlanaceRentForExemptStakingStore(connection) {
    return await connection.getMinimumBalanceForRentExemption(
      Layout.StakeStoreLayout.span
    );
  }

  static async getMinBlanaceRentForExemptStakingList(connection) {
    return await connection.getMinimumBalanceForRentExemption(
      Layout.StakeListLayout(MAX_STAKE_COUNT).span
    );
  }

  static async loadStakingStore(connection, storeKey, programId) {
    const data = await loadAccount(connection, storeKey, programId);
    const stakingStoreData = Layout.StakeStoreLayout.decode(data);
    if (!stakingStoreData.isInitialized) {
      throw new Error(`Invalid staking store state`);
    }

    const [authorityKey] = await PublicKey.findProgramAddress(
      [storeKey.toBuffer()],
      programId
    );

    const manager = new PublicKey(stakingStoreData.manager);
    const stakeList = new PublicKey(stakingStoreData.stakeList);
    const tokenProgramId = TOKEN_PROGRAM_ID;

    return new StakeStore(
      connection,
      programId,
      storeKey,
      tokenProgramId,
      authorityKey,
      manager
    );
  }

  static async createStakingStore(
    connection,
    payerKeyPair,
    stakeStoreKeyPair,
    stakeListKeyPair,
    managerKeyPair,
    tokenProgramId,
    stakingProgramId
  ) {
    let balanceNeeded = await StakeStore.getMinBlanaceRentForExemptStakingStore(
      connection
    );
    // console.info("balanceNeeded for stakeStore", balanceNeeded);
    // console.info("space for stakeStore", Layout.StakeStoreLayout.span);
    let transaction = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: payerKeyPair.publicKey,
        newAccountPubkey: stakeStoreKeyPair.publicKey,
        lamports: balanceNeeded,
        space: Layout.StakeStoreLayout.span,
        programId: stakingProgramId,
      })
    );
    balanceNeeded = await StakeStore.getMinBlanaceRentForExemptStakingList(
      connection
    );
    // console.info("balanceNeeded for stakeList", balanceNeeded);
    // console.info(
    //   "space for stakeList",
    //   Layout.StakeListLayout(MAX_STAKE_COUNT).span
    // );
    try {
      await sendAndConfirmTransaction(
        "create stakeStore account",
        connection,
        transaction,
        payerKeyPair,
        stakeStoreKeyPair
      );
    } catch (e) {
      console.log("error occured on transaction to create stakeStore");
      throw e;
    }

    transaction = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: payerKeyPair.publicKey,
        newAccountPubkey: stakeListKeyPair.publicKey,
        lamports: balanceNeeded,
        space: Layout.StakeListLayout(MAX_STAKE_COUNT).span,
        programId: stakingProgramId,
      })
    );
    try {
      await sendAndConfirmTransaction(
        "create stakeList account",
        connection,
        transaction,
        payerKeyPair,
        stakeListKeyPair
      );
    } catch (e) {
      console.log("error occured on transaction to create stakeList");
      throw e;
    }
    const instruction = instructions.createInitStakingInstruction(
      stakeStoreKeyPair.publicKey,
      stakeListKeyPair.publicKey,
      managerKeyPair.publicKey,
      stakingProgramId
    );
    transaction = new Transaction().add(instruction);
    try {
      await sendAndConfirmTransaction(
        "initialize",
        connection,
        transaction,
        payerKeyPair,
        managerKeyPair
      );
    } catch (e) {
      console.log("error occured on initialize transaction");
      throw e;
    }

    return new StakeStore(
      connection,
      stakingProgramId,
      stakeStoreKeyPair.publicKey,
      stakeListKeyPair.publicKey,
      tokenProgramId,
      managerKeyPair
    );
  }

  stake(userKeypair, mintKeyapir, stakeKey, nftAmount) {
    const instruction = instructions.stakeInstruction(
      userKeypair.publicKey,
      mintKeyapir.publicKey,
      stakeKey,
      this.stakeStoreKey,
      this.stakeListKey,
      this.tokenProgramId,
      nftAmount,
      this.stakingProgramId
    );
    return new Transaction().add(instruction);
  }

  reclaim(userAccountNFT, mintAccount, stakeAccount, pdaStakeAccount) {
    const instruction = instructions.reclaimInstruction(
      userAccountNFT,
      mintAccount.publicKey,
      this.stakeStoreKey,
      this.stakeListKey,
      stakeAccount.publicKey,
      pdaStakeAccount.publicKey,
      this.tokenProgramId,
      this.stakingProgramId
    );
    return new Transaction().add(instruction);
  }
}
