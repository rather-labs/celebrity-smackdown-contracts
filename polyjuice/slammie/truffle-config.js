module.exports = {
    networks: {
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
    db: {
        enabled: false
    }
};
