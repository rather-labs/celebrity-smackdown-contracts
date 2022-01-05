require('dotenv').config();
const { existsSync } = require('fs');
const Web3 = require('web3');
const { PolyjuiceHttpProvider, PolyjuiceAccounts } = require("@polyjuice-provider/web3");

(async () => {

    const compiledContract = loadContract();
    const web3 = configureWeb3(compiledContract);
    const deployerAccount = getDeployerAccount(web3);

    if (validateBalance(deployerAccount, web3)) {
        console.log(`Deploying contract ${compiledContract.contractName}...`);

        const deployTx = new web3.eth.Contract(compiledContract.abi).deploy({
            data: compiledContract.bytecode,
            arguments: [process.env.TREASURY_ADDRESS]
        }).send({
            from: deployerAccount.address,
            gas: 6000000,
        });

        deployTx.on('transactionHash', onTransactionHash);
        deployTx.on('receipt', onReceipt);
        deployTx.on('error', onError);

        const contract = await deployTx;

        console.log(`Deployed contract address: ${contract.options.address}`);
    }
})();

function loadContract() {
    const contractName = process.argv.slice(2)[0];
    // contractName validation
    if (!contractName) {
        const massage = `
            No compiled contract specified to deploy.
            The contract must exist in /build/contracts and its name must be provided as argument."
        `;
        throw new Error(massage);
    }
    // contract loading
    const filename = `./build/contracts/${contractName}.json`;
    if (existsSync(filename)) {
        return require(filename);
    }
    throw new Error(`Unable to find contract file: ${filename}`);
}

function configureWeb3(compiledContract) {
    const polyjuiceConfig = {
        web3Url: 'https://godwoken-testnet-web3-rpc.ckbapp.dev',
    };
    const provider = new PolyjuiceHttpProvider(
        polyjuiceConfig.web3Url,
        polyjuiceConfig,
    );
    provider.setMultiAbi([compiledContract.abi]);
    const web3 = new Web3(provider);
    web3.eth.accounts = new PolyjuiceAccounts(polyjuiceConfig);
    web3.eth.Contract.setProvider(provider, web3.eth.accounts);
    return web3;
}

function getDeployerAccount(web3) {
    return web3.eth.accounts.wallet.add(process.env.DEPLOYER_PRIVATE_KEY);
}

async function validateBalance(deployerAccount, web3) {
    const balance = BigInt(await web3.eth.getBalance(deployerAccount.address));
    console.log(`Balance: ${balance}`);
    if (balance === 0n) {
        const message = `
            Insufficient balance, can't deploy contract.
            Please deposit funds to your Ethereum address: ${deployerAccount.address}
        `;
        throw new Error(message);
    }
    return true;
}

function onTransactionHash(hash) {
    console.log(`
    Transaction hash: ${hash}
    `);
}

function onReceipt(receipt) {
    console.log(`
    Receipt:
    transactionHash: ${receipt.transactionHash}
    transactionIndex: ${receipt.transactionIndex}
    blockHash: ${receipt.blockHash}
    blockNumber: ${receipt.blockNumber}
    from: ${receipt.from}
    to: ${receipt.to}
    gasUsed: ${receipt.gasUsed}
    cumulativeGasUsed: ${receipt.cumulativeGasUsed}
    contractAddress: ${receipt.contractAddress}
    status: ${receipt.status}
    `);
}

function onError(err) {
    console.log(`
    Error: ${err}
    `);
}