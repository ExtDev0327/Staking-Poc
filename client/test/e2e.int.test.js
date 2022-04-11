import { Token } from "@solana/spl-token";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import {
  DEFAULT_TOKEN_DECIMALS,
  MAX_STAKE_COUNT,
  TOKEN_PROGRAM_ID,
} from "../src/constants";
import {
  PublicKeyLayout,
  StakeListLayout,
  StakeStoreLayout,
} from "../src/layout";
import { StakeStore } from "../src/staking-store";
import { sendAndConfirmTransaction } from "../src/util/send-and-confirm-transaction";
import { getDeploymentInfo, newAccountWithLamports, sleep } from "./helpers";
import BN from "bn.js";

// Cluster configs
const CLUSTER_URL = "http://localhost:8899";
const BOOTSTRAP_TIMEOUT = 120000;
const RECLAIM_TIMEOUT = 2000;

describe("e2e test", () => {
  // Cluster connection
  let connection;
  // Fee payer
  let payerKeyPair;
  // owner of contract
  let ownerKeyPair;
  // test user
  let userKeyPair;
  let stakingProgramId;
  let stakeStore;
  let mintNFT1, mintNFT2;
  let stakeStoreKeyPair, stakeListKeyPair;

  beforeAll(async () => {
    // Bootstrap Test Environment ...
    connection = new Connection(CLUSTER_URL, "confirmed");
    payerKeyPair = await newAccountWithLamports(
      connection,
      LAMPORTS_PER_SOL * 50
    );
    ownerKeyPair = await newAccountWithLamports(connection, LAMPORTS_PER_SOL);
    userKeyPair = await newAccountWithLamports(connection, LAMPORTS_PER_SOL);

    stakingProgramId = getDeploymentInfo().stakingProgramId;
    stakeStoreKeyPair = new Keypair();
    stakeListKeyPair = new Keypair();
    // let authority, nonce;
    // try {
    //   [authority, nonce] = await PublicKey.findProgramAddress(
    //     [stakeStoreKeyPair.publicKey.toBuffer()],
    //     stakingProgramId
    //   );
    // } catch (e) {
    //   throw new Error(e);
    // }

    try {
      mintNFT1 = await Token.createMint(
        connection,
        payerKeyPair,
        ownerKeyPair.publicKey,
        ownerKeyPair.publicKey,
        DEFAULT_TOKEN_DECIMALS,
        TOKEN_PROGRAM_ID
      );
      mintNFT2 = await Token.createMint(
        connection,
        payerKeyPair,
        ownerKeyPair.publicKey,
        ownerKeyPair.publicKey,
        DEFAULT_TOKEN_DECIMALS,
        TOKEN_PROGRAM_ID
      );
    } catch (e) {
      throw new Error(e);
    }

    try {
      stakeStore = await StakeStore.createStakingStore(
        connection,
        payerKeyPair,
        stakeStoreKeyPair,
        stakeListKeyPair,
        ownerKeyPair,
        TOKEN_PROGRAM_ID,
        stakingProgramId
      );
      await sleep(1000);
      // check initialized state
      const stakeStoreData = StakeStoreLayout.decode(
        (
          await connection.getAccountInfo(
            stakeStore.stakeStoreKey,
            connection.commitment || "confirmed"
          )
        ).data
      );
      expect(stakeStoreData.isInitialized).toBe(1);
      expect(new PublicKey(stakeStoreData.manager).toString()).toBe(
        ownerKeyPair.publicKey.toString()
      );
      expect(new PublicKey(stakeStoreData.stakeList).toString()).toBe(
        stakeListKeyPair.publicKey.toString()
      );
      expect(stakeStoreData.stakedCount).toBe(0);
      const stakeListData = StakeListLayout(stakeStoreData.stakedCount).decode(
        (
          await connection.getAccountInfo(
            stakeListKeyPair.publicKey,
            connection.commitment || "confirmed"
          )
        ).data
      );
      console.log("===", stakeListData.header.count);
      expect(stakeListData.header.isInitialized).toBe(1);
      expect(stakeListData.header.maxItems).toBe(MAX_STAKE_COUNT);
      expect(stakeListData.items.length).toBe(stakeStoreData.stakedCount);
    } catch (e) {
      throw new Error(e);
    }
  }, BOOTSTRAP_TIMEOUT);

  it("stake", async () => {
    console.info("starting stake...");
    let userAccountNFT1Key = await mintNFT1.createAccount(
      userKeyPair.publicKey
    );
    let userAccountNFT2Key = await mintNFT2.createAccount(
      userKeyPair.publicKey
    );
    let stakeAccountNFT1Key = await mintNFT1.createAccount(
      userKeyPair.publicKey
    );
    let stakeAccountNFT2Key = await mintNFT2.createAccount(
      userKeyPair.publicKey
    );
    await mintNFT1.mintTo(userAccountNFT1Key, ownerKeyPair, [], 1);
    await mintNFT1.approve(
      userAccountNFT1Key,
      userKeyPair.publicKey,
      userKeyPair,
      [],
      1
    );
    await mintNFT2.mintTo(userAccountNFT2Key, ownerKeyPair, [], 1);
    await mintNFT2.approve(
      userAccountNFT2Key,
      userKeyPair.publicKey,
      userKeyPair,
      [],
      1
    );
    await sleep(1000);
    // transfer token to stakeAccount
    try {
      let userAccountInfo1 = await mintNFT1.getAccountInfo(
        userAccountNFT1Key,
        connection.commitment || "confirmed"
      );
      let stakeAccountInfo1 = await mintNFT1.getAccountInfo(
        stakeAccountNFT1Key,
        connection.commitment || "confirmed"
      );
      let userAccountInfo2 = await mintNFT2.getAccountInfo(
        userAccountNFT2Key,
        connection.commitment || "confirmed"
      );
      let stakeAccountInfo2 = await mintNFT2.getAccountInfo(
        stakeAccountNFT2Key,
        connection.commitment || "confirmed"
      );
      expect(userAccountInfo1.amount.toNumber()).toBe(1);
      expect(stakeAccountInfo1.amount.toNumber()).toBe(0);
      expect(userAccountInfo2.amount.toNumber()).toBe(1);
      expect(stakeAccountInfo2.amount.toNumber()).toBe(0);
      const transaction = new Transaction().add(
        Token.createTransferInstruction(
          TOKEN_PROGRAM_ID,
          userAccountNFT1Key,
          stakeAccountNFT1Key,
          userKeyPair.publicKey,
          [],
          1
        )
      );
      transaction.add(
        Token.createTransferInstruction(
          TOKEN_PROGRAM_ID,
          userAccountNFT2Key,
          stakeAccountNFT2Key,
          userKeyPair.publicKey,
          [],
          1
        )
      );
      await sendAndConfirmTransaction(
        "transfer nft1, nft2",
        connection,
        transaction,
        userKeyPair
      );
      await sleep(500);
      userAccountInfo1 = await mintNFT1.getAccountInfo(
        userAccountNFT1Key,
        connection.commitment || "confirmed"
      );
      stakeAccountInfo1 = await mintNFT1.getAccountInfo(
        stakeAccountNFT1Key,
        connection.commitment || "confirmed"
      );
      expect(userAccountInfo1.amount.toNumber()).toBe(0);
      expect(stakeAccountInfo1.amount.toNumber()).toBe(1);
      userAccountInfo2 = await mintNFT2.getAccountInfo(
        userAccountNFT2Key,
        connection.commitment || "confirmed"
      );
      stakeAccountInfo2 = await mintNFT2.getAccountInfo(
        stakeAccountNFT2Key,
        connection.commitment || "confirmed"
      );
      expect(userAccountInfo2.amount.toNumber()).toBe(0);
      expect(stakeAccountInfo2.amount.toNumber()).toBe(1);
    } catch (e) {
      throw new Error(e);
    }

    try {
      const transaction = stakeStore.stake(
        userKeyPair,
        mintNFT1,
        stakeAccountNFT1Key,
        1
      );
      transaction.add(
        stakeStore.stake(userKeyPair, mintNFT2, stakeAccountNFT2Key, 1)
      );
      await sendAndConfirmTransaction(
        "stake",
        connection,
        transaction,
        payerKeyPair,
        userKeyPair
      );
      await sleep(500);
    } catch (e) {
      throw new Error(e);
    }
    // check stakeStore and stakeList
    const stakeStoreData = StakeStoreLayout.decode(
      (
        await connection.getAccountInfo(
          stakeStore.stakeStoreKey,
          connection.commitment || "confirmed"
        )
      ).data
    );
    expect(stakeStoreData.stakedCount).toBe(2);
    const stakeListData = StakeListLayout(stakeStoreData.stakedCount).decode(
      (
        await connection.getAccountInfo(
          stakeListKeyPair.publicKey,
          connection.commitment || "confirmed"
        )
      ).data
    );
    console.log("===", stakeListData.header.count);
    expect(stakeListData.items.length).toBe(stakeStoreData.stakedCount);
    const item1 = stakeListData.items[0];
    const item2 = stakeListData.items[1];
    console.log(stakeListData.items.length, stakeStoreData.stakedCount);
    console.log(item1);
    console.log(item2);
    expect(new PublicKey(item1.owner).toString()).toBe(
      userKeyPair.publicKey.toString()
    );
    expect(new PublicKey(item1.tokenMint).toString()).toBe(
      mintNFT1.publicKey.toString()
    );
    expect(new PublicKey(item1.holder).toString()).toBe(
      stakeAccountNFT1Key.publicKey.toString()
    );
    expect(new PublicKey(item2.owner).toString()).toBe(
      userKeyPair.publicKey.toString()
    );
    expect(new PublicKey(item2.tokenMint).toString()).toBe(
      mintNFT2.publicKey.toString()
    );
    expect(new PublicKey(item2.holder).toString()).toBe(
      stakeAccountNFT2Key.publicKey.toString()
    );
  });

  // it("reclaim", async () => {
  //   console.log("starting reclaim...");
  //   let pdaStakeAccount1, pdaStakeAccount2;
  //   // stake to contract
  //   let userAccountNFT1 = await mintNFT1.createAccount(userKeyPair.publicKey);
  //   let userAccountNFT2 = await mintNFT2.createAccount(userKeyPair.publicKey);
  //   await mintNFT1.mintTo(userAccountNFT1, ownerKeyPair, [], 1);
  //   await mintNFT1.approve(userAccountNFT1, userKeyPair, userKeyPair, [], 1);
  //   await mintNFT2.mintTo(userAccountNFT2, ownerKeyPair, [], 1);
  //   await mintNFT2.approve(userAccountNFT2, userKeyPair, userKeyPair, [], 1);
  //   let stakeAccountNFT1 = await mintNFT1.createAccount(userKeyPair.publicKey);
  //   let stakeAccountNFT2 = await mintNFT2.createAccount(userKeyPair.publicKey);
  //   await sleep(1000);
  //   // transfer token to stakeAccount
  //   // ...

  //   try {
  //     const transaction = stakeStore.stake(userAccountNFT1, 1);
  //     await sendAndConfirmTransaction("stake", connection, transaction, payerKeyPair);
  //     // get pdaStakeAccount1 from solana net
  //     // ...
  //     transaction = stakeStore.stake(userAccountNFT2, 1);
  //     // get pdaStakeAccount2 from solana net
  //     // ...
  //     await sendAndConfirmTransaction("stake", connection, transaction, payerKeyPair);
  //   } catch (e) {
  //     throw new Error(e);
  //   }

  //   // need to confirm my guessing relate with test
  //   let info = await mintNFT1.getAccountInfo(stakeAccountNFT1);
  //   expect(info.amount.toNumber()).toBe(1);
  //   expect(info.amount.toNumber()).toBe(2);

  //   // reclaim from contract
  //   await sleep(RECLAIM_TIMEOUT / 2);
  //   try {
  //     const txn = stakeStore.reclaim(userAccountNFT1);
  //     await sendAndConfirmTransaction("reclaim", connection, txn, payerKeyPair);
  //   } catch (e) {
  //     // please check if error is not enough for time to reclaim
  //     // ...
  //     throw new Error(e);
  //   }
  //   await sleep(RECLAIM_TIMEOUT / 2);
  //   try {
  //     const txn = stakeStore.reclaim(userAccountNFT1);
  //     await sendAndConfirmTransaction("reclaim", connection, txn, payerKeyPair);
  //     txn = stakeStore.reclaim(userAccountNFT2);
  //     await sendAndConfirmTransaction("reclaim", connection, txn, payerKeyPair);
  //   } catch (e) {
  //     throw new Error(e);
  //   }

  //   info = await mintNFT1.getAccountInfo(stakeAccountNFT1);
  //   expect(info.amount.toNumber()).toBe(0);
  //   info = await mintNFT2.getAccountInfo(stakeAccountNFT2);
  //   expect(info.amount.toNumber()).toBe(0);
  //   info = await mintNFT1.getAccountInfo(userAccountNFT1);
  //   expect(info.amount.toNumber()).toBe(1);
  //   info = await mintNFT1.getAccountInfo(userAccountNFT2);
  //   expect(info.amount.toNumber()).toBe(1);
  // });
});
