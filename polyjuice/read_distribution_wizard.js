require('dotenv').config();
const { ContractFactory, Contract } = require("ethers");
const { PolyjuiceConfig } = require("@polyjuice-provider/base");
const { PolyjuiceWallet, PolyjuiceJsonRpcProvider } = require("@polyjuice-provider/ethers");

const polyjuiceConfig = {
    web3Url: process.env.WEB3_PROVIDER_URL,
};

const rpc = new PolyjuiceJsonRpcProvider(polyjuiceConfig, polyjuiceConfig.web3Url);
const distributionWizardBinary = require("./build/contracts/DistributionWizard.json");

(async () => {
    try {
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