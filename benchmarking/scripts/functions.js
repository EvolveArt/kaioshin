// SPDX-License-Identifier: Apache-2.0

const { Keyring } = require("@polkadot/keyring");
const {
  transfer,
  deploy,
  declare,
  initialize,
} = require("../../tests/build/util/starknet");

module.exports = {
  rpcMethods,
  executeERC20Transfer,
};

function rpcMethods(userContext, events, done) {
  const data = { id: 1, jsonrpc: "2.0", method: "rpc_methods" };
  // set the "data" variable for the virtual user to use in the subsequent action
  userContext.vars.data = data;
  return done();
}

async function executeERC20Transfer(userContext, events, done) {
  const { accountName, deployed } = userContext.vars;

  const keyring = new Keyring({ type: "sr25519" });
  const user = keyring.addFromUri(`//${accountName}`);

  const contractAddress =
    "0x0000000000000000000000000000000000000000000000000000000000000101";
  const amount =
    "0x0000000000000000000000000000000000000000000000000000000000000001";
  const mintAmount =
    "0x0000000000000000000000000000000000000000000000000000000000001000";
  const tokenClassHash =
    "0x025ec026985a3bf9d0cc1fe17326b245bfdc3ff89b8fde106242a3ea56c5a918";

  // Setup contract if it doesn't exist
  // let tokenAddress;
  // if (!deployed[tokenClassHash]) {
  //   await declare(userContext.api, user, contractAddress, tokenClassHash);

  //   tokenAddress = await deploy(
  //     userContext.api,
  //     user,
  //     contractAddress,
  //     tokenClassHash
  //   );

  //   console.log("Deployed token address: ", tokenAddress);

  //   // Update userContext deployed dict
  //   userContext.vars.deployed = {
  //     ...userContext.vars.deployed,
  //     [tokenClassHash]: true,
  //   };
  // }

  await transfer(
    userContext.api,
    user,
    contractAddress,
    "0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00",
    contractAddress,
    amount
  );

  return done();
}
