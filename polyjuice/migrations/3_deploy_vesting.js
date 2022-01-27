require('dotenv').config();

const Vesting = artifacts.require("Vesting");

module.exports = function (deployer, network, accounts) {
    deployer.deploy(Vesting, process.env.VESTING_ADDRESS, process.env.VESTING_START_TIMESTAMP, process.env.VESTING_DURATION_SECONDS);
};