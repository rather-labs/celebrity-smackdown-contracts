require('dotenv').config();
const { ContractFactory, Contract } = require("ethers");
const { PolyjuiceConfig } = require("@polyjuice-provider/base");
const { PolyjuiceWallet, PolyjuiceJsonRpcProvider } = require("@polyjuice-provider/ethers");

const polyjuiceConfig = {
  web3Url: process.env.WEB3_PROVIDER_URL,
};

const rpc = new PolyjuiceJsonRpcProvider(polyjuiceConfig, polyjuiceConfig.web3Url);
const signer = new PolyjuiceWallet(process.env.DEPLOYER_PRIVATE_KEY, polyjuiceConfig, rpc);

const slammieBinary = require("./build/contracts/Slammie.json");

(async () => {
  try {
    const slammieContract = new Contract(
      process.env.DEPLOYED_SLAMMIE_ADDRESS,
      slammieBinary.abi,
      signer
    );
    console.log("calling slammie approve...");
    const txResponse = await slammieContract.approve(process.env.TREASURY_ADDRESS, 1);
    console.log(txResponse);
  } catch (error) {
    console.error(error);
  }
})();