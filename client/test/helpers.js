import fs from "fs";
import { Keypair, PublicKey } from "@solana/web3.js";

export function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function newAccountWithLamports(connection, lamports = 1000000) {
  const keypair = new Keypair();

  let retries = 60;
  try {
    await connection.requestAirdrop(keypair.publicKey, lamports);
  } catch (e) {
    console.error(e);
  }
  for (;;) {
    await sleep(1000);
    if (lamports === (await connection.getBalance(keypair.publicKey))) {
      return keypair;
    }
    if (--retries <= 0) {
      break;
    }
  }
  throw new Error(`Airdrop of ${lamports} failed`);
}

export const getDeploymentInfo = () => {
  const data = fs.readFileSync("../last-deploy.json", "utf-8");
  const deployInfo = JSON.parse(data);
  return {
    clusterUrl: deployInfo.clusterUrl,
    stakingProgramId: new PublicKey(deployInfo.stakingProgramId),
  };
};
