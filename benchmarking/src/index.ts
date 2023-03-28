import { WsProvider, ApiPromise } from "@polkadot/api";
import { Keyring } from "@polkadot/keyring";
import { stringToU8a } from "@polkadot/util";
import erc20Json from "../../ressources/erc20.json" assert { type: "json" };
import "@polkadot/api/augment";
import "@polkadot/types/augment";
import * as definitions from "./interfaces/definitions";

import type {
  EntryPointTypeWrapper,
  ContractClassWrapper,
  EntryPointWrapper,
} from "./interfaces";

interface BaseConfig {
  api: ApiPromise;
  user: any;
}


async function init(): Promise<BaseConfig> {
  // extract all types from definitions - fast and dirty approach, flatted on 'types'
  const types = Object.values(definitions).reduce(
    (res, { types }): object => ({ ...res, ...types }),
    {}
  );

  const wsProvider = new WsProvider();
  const api = await ApiPromise.create({
    provider: wsProvider,
    types: { ...types },
  });

  await api.isReady;

  console.log(api.genesisHash.toHex());

  const keyring = new Keyring({ type: "sr25519" });
  const user = keyring.addFromUri(`//Alice`);

  return { api, user };
}

async function declare(
  api: ApiPromise,
  user: any,
  contractAddress: string,
  tokenClassHash: string
) {
  const tx_declare = {
    version: 1, // version of the transaction
    hash: "", // leave empty for now, will be filled in by the runtime
    signature: [], // leave empty for now, will be filled in when signing the transaction
    events: [], // empty vector for now, will be filled in by the runtime
    sender_address: contractAddress, // address of the sender contract
    nonce: 0, // nonce of the transaction
    callEntrypoint: {
      // call entrypoint
      classHash: tokenClassHash, // class hash of the contract
      entrypointSelector: null, // function selector of the transfer function
      calldata: [], // empty vector for now, will be filled in by the runtime
      storageAddress: contractAddress,
      callerAddress: contractAddress,
    },
    contractClass: {
      program: stringToU8a(JSON.stringify(erc20Json.program)),
      entryPointsByType: stringToU8a(JSON.stringify(erc20Json.entry_points_by_type)),
    },
  };

  const extrisinc_declare = api.tx.starknet.addDeclareTransaction(tx_declare);

  try {
    const signedTxDeclare = await extrisinc_declare.signAsync(user, {
      nonce: -1,
    });
    const resultDeclare = await signedTxDeclare.send();
    console.log(resultDeclare.toHuman());
  } catch (error) {
    console.error("Eror while declaring : ", error);
  }
}

async function deploy(
  api: ApiPromise,
  user: any,
  contractAddress: string,
  tokenClassHash: string
) {
  // Deploy contract
  let tx_deploy = {
    version: 1, // version of the transaction
    hash: "", // leave empty for now, will be filled in by the runtime
    signature: [], // leave empty for now, will be filled in when signing the transaction
    events: [], // empty vector for now, will be filled in by the runtime
    sender_address: contractAddress, // address of the sender contract
    nonce: 1, // nonce of the transaction
    callEntrypoint: {
      // call entrypoint
      classHash: tokenClassHash, // class hash of the contract
      entrypointSelector: null, // function selector of the transfer function
      calldata: [
        "0x0000000000000000000000000000000000000000000000000000000000001111",
        "0x0169f135eddda5ab51886052d777a57f2ea9c162d713691b5e04a6d4ed71d47f",
        "0x0000000000000000000000000000000000000000000000000000000000000004",
        tokenClassHash,
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
      ],
      storageAddress: contractAddress,
      callerAddress: contractAddress,
    },
    contractClass: null,
  };

  try {
    const extrisinc_deploy = api.tx.starknet.addInvokeTransaction(tx_deploy);
    const signedTxDeploy = await extrisinc_deploy.signAsync(user, {
      nonce: -1,
    });
    const resultDeploy = await signedTxDeploy.send();
    console.log(resultDeploy.toHuman()?.toString());
  } catch (error) {
    console.error("Eror while deploying : ", error);
  }
}

async function main(): Promise<void> {
  const { api, user } = await init();

  const contractAddress =
    "0x0000000000000000000000000000000000000000000000000000000000000101";
  const tokenClassHash =
    "0x025ec026985a3bf9d0cc1fe17326b245bfdc3ff89b8fde106242a3ea56c5a918";

  await declare(api, user, contractAddress, tokenClassHash);

  await deploy(api, user, contractAddress, tokenClassHash);
}

void main();
