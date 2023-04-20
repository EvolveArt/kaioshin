const { ApiPromise, WsProvider } = require("@polkadot/api");
const fs = require("fs");

async function main() {
  const wsProvider = new WsProvider("ws://localhost:9944");
  const api = await ApiPromise.create({ provider: wsProvider });

  const blockHash = await api.rpc.chain.getBlock();
  const blockNumber = blockHash.block.header.number;

  let totalExtrinsics = 0;

  for (let i = blockNumber.toNumber() - 9; i <= blockNumber.toNumber(); i++) {
    const hash = await api.rpc.chain.getBlockHash(i);
    const block = await api.rpc.chain.getBlock(hash);
    totalExtrinsics += block.block.extrinsics.length;
  }

  const avgExtrinsicsPerBlock = totalExtrinsics / 10;
  const avgTps = avgExtrinsicsPerBlock * 50 / 6;

  // Save avgExtrinsicsPerBlock to file reports/metrics.json
  fs.writeFileSync(
    "reports/metrics.json",
    JSON.stringify({ avgExtrinsicsPerBlock, avgTps })
  );

  console.log(
    `Average extrinsics per block in the last 10 blocks: ${avgExtrinsicsPerBlock}`
  );
  console.log(
    `Average TPS: ${avgTps}`
  );
}

main().catch((err) => {
  console.error(err);
  process.exit(-1);
}).then(() => process.exit(0));
