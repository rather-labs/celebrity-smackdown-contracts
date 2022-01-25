// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Votes.sol";

contract Slammie is ERC20Votes {
    constructor(address _treasury)
        ERC20("Slammie", "SLAM")
        ERC20Permit("Slammie")
    {
        _mint(_treasury, 50_000_000 ether);
    }
}