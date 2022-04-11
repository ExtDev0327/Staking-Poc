import { sendAndConfirmTransaction as realSendAndConfirmTransaction } from "@solana/web3.js";

export const sendAndConfirmTransaction = async (
  title,
  connection,
  transaction,
  ...signers
) => {
  console.info(`Sending ${title} transaction`);
  // try {
    const txSig = await realSendAndConfirmTransaction(
      connection,
      transaction,
      signers,
      {
        skipPreflight: false,
        commitment: connection.commitment || "confirmed",
        preflightCommitment: connection.commitment || "confirmed",
      }
    );
    console.info(`TxSig: ${txSig}`);
    return txSig;
  // } catch (e) {
  //   console.log("-----------------", e);
  // }
};
