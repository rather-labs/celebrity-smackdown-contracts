require('dotenv').config();
const { ContractFactory } = require("ethers");
const { PolyjuiceConfig } = require("@polyjuice-provider/base");
const { PolyjuiceWallet, PolyjuiceJsonRpcProvider } = require("@polyjuice-provider/ethers");

const polyjuiceConfig = {
  //rollupTypeHash: process.env.ROLLUP_TYPE_HASH, // this is optional 
  //ethAccountLockCodeHash: process.env.ETH_ACCOUNT_LOCK_CODE_HASH, // this is optional
  //creatorId: 'polyjuice creator account id', // this is optional
  //defaultFromAddress: 'a default eth address, which will be used as a default from in ethereum transaction', // this is optional
  //abiItems: ['your abi items array'],
  web3Url: process.env.WEB3_PROVIDER_URL, 
};

const rpc = new PolyjuiceJsonRpcProvider(polyjuiceConfig, polyjuiceConfig.web3Url);
const deployer = new PolyjuiceWallet(process.env.DEPLOYER_PRIVATE_KEY, polyjuiceConfig, rpc);
//console.log(deployer);

const contract = new ethers.Contract(
    'your contract address',
    'your contract abi',
    signer
  );
  let overrides = {
    gasLimit: 0x54d30,
    gasPrice: 0x0,
    value: 0x0,
  };
  const txResponse = await contract.METHOD_NAME(..args, overrides);
  console.log(txResponse);
  