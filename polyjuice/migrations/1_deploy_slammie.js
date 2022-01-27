require('dotenv').config();

const Slammie = artifacts.require("Slammie");

module.exports = function (deployer, network, accounts) {
    deployer.deploy(Slammie, process.env.TREASURY_ADDRESS);
};