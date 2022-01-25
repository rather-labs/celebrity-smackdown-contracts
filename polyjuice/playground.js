require('dotenv').config();
const { ContractFactory, Contract } = require("ethers");
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
//const deployer = new PolyjuiceWallet(process.env.DEPLOYER_PRIVATE_KEY, polyjuiceConfig, rpc);

const slammieBinary = require("./build/contracts/Slammie.json");
const distributionWizardBinary = require("./build/contracts/DistributionWizard.json");

(async () => {
  try {
    const slammieContract = new Contract(
      process.env.DEPLOYED_SLAMMIE_ADDRESS,
      slammieBinary.abi,
      rpc//signer
    );
    console.log("getting slammie name...");
    const slammieName = await slammieContract.name();
    console.log(slammieName);
    
    const distributionWizardContract = new Contract(
      process.env.DEPLOYED_DISTRIBUTION_WIZARD_ADDRESS,
      distributionWizardBinary.abi,
      rpc//signer
    );
    console.log("getting distributionWizard owner...");
    const distributionWizardOwner = await distributionWizardContract.owner();
    console.log(distributionWizardOwner);
  } catch (error) {
    console.error(error);
  }
})();