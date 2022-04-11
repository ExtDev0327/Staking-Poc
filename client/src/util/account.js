export const loadAccount = async (
  connection,
  address,
  programId
) => {
  const accountInfo = await connection.getAccountInfo(address);
  if (accountInfo === null) {
    throw new Error("Failed to find account");
  }

  if (!accountInfo.owner.equals(programId)) {
    throw new Error(
      `Invalid owner: expected ${programId.toBase58()}, found ${accountInfo.owner.toBase58()}`
    );
  }

  return Buffer.from(accountInfo.data);
};
