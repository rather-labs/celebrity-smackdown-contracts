require('dotenv').config();

const Slammie = artifacts.require("Slammie");
const DistributionWizard = artifacts.require("DistributionWizard");

module.exports = function (deployer, network, accounts) {
    deployer.deploy(DistributionWizard, Slammie.address, process.env.TREASURY_ADDRESS, 1);
};