require('dotenv').config();
const { PolyjuiceHDWalletProvider } = require('@polyjuice-provider/truffle');
const { PolyjuiceHttpProvider } = require('@polyjuice-provider/web3');

const polyjuiceConfig = {
  rollupTypeHash: process.env.ROLLUP_TYPE_HASH,
  ethAccountLockCodeHash: process.env.ETH_ACCOUNT_LOCK_CODE_HASH,
  web3Url: process.env.WEB3_PROVIDER_URL,
};
const polyjuiceHttpProvider = new PolyjuiceHttpProvider(
  polyjuiceConfig.web3Url,
  polyjuiceConfig
);
const polyjuiceTruffleProvider = new PolyjuiceHDWalletProvider(
  [
    {
      privateKeys: [process.env.DEPLOYER_PRIVATE_KEY],
      providerOrUrl: polyjuiceHttpProvider,
    },
  ],
  polyjuiceConfig
);

module.exports = {
  networks: {
    development: {
      gasPrice: 0,
      network_id: "*",
      provider: () => polyjuiceTruffleProvider,
    },
  },

  mocha: {
  },

  compilers: {
    solc: {
      version: '0.8.3',
      docker: false,
      settings: {
        optimizer: {
          enabled: false,
          runs: 200
        },
        evmVersion: 'istanbul'
      }
    }
  },
};
