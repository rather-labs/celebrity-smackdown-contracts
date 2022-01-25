require('dotenv').config();
const { ContractFactory, Contract } = require("ethers");
const { PolyjuiceConfig } = require("@polyjuice-provider/base");
const { PolyjuiceWallet, PolyjuiceJsonRpcProvider } = require("@polyjuice-provider/ethers");

const polyjuiceConfig = {
    web3Url: process.env.WEB3_PROVIDER_URL,
};

const rpc = new PolyjuiceJsonRpcProvider(polyjuiceConfig, polyjuiceConfig.web3Url);
const slammieBinary = require("./build/contracts/Slammie.json");

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
    } catch (error) {
        console.error(error);
    }
})();