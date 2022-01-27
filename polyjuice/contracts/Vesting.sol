// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/finance/VestingWallet.sol";

contract Vesting is VestingWallet {
    constructor(
        address _beneficiaryAddress,
        uint64 _startTimestamp,
        uint64 _durationSeconds
    ) VestingWallet(_beneficiaryAddress, _startTimestamp, _durationSeconds) {}
}
