import * as BufferLayout from "buffer-layout";

/**
 * Layout for a public key
 */
export const PublicKeyLayout = (property = "publicKey") => {
  return BufferLayout.blob(32, property);
};

/**
 * Layout for a 64bit unsigned value
 */
export const Uint64Layout = (property = "uint64") => {
  return BufferLayout.blob(8, property);
};

export const StakeStoreLayout = BufferLayout.struct([
  BufferLayout.u8("isInitialized"),
  PublicKeyLayout("manager"),
  BufferLayout.u16("stakedCount"),
  PublicKeyLayout("stakeList"),
]);

export const StakeListHeaderLayout = (property) =>
  BufferLayout.struct(
    [
      BufferLayout.u8("isInitialized"),
      BufferLayout.u16("maxItems"),
      BufferLayout.u16("count"),
      //... need reclaim timeout
    ],
    property
  );
export const StakeItemLayout = BufferLayout.struct([
  PublicKeyLayout("owner"),
  PublicKeyLayout("tokenMint"),
  PublicKeyLayout("holder"),
  Uint64Layout("stakeTime"),
]);

export const StakeListLayout = (n) =>
  BufferLayout.struct([
    StakeListHeaderLayout("header"),
    BufferLayout.seq(StakeItemLayout, n, "items"),
  ]);
