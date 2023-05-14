module.exports = {
    ...require('near-workspaces/ava.testnet.config.cjs'),
    ...require('./ava.config.cjs'),
   };
module.exports.environmentVariables = {
       TESTNET_MASTER_ACCOUNT_ID: 'htamlive.testnet',
   };