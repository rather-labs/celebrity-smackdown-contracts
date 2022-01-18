// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract DistributionWizard is Ownable {
    IERC20 token;
    address treasury;
    uint256 public epoch;
    uint256 public initialTimestamp;

    mapping(address => uint256) public lastTimeWithdrawn;

    constructor(
        IERC20 _token,
        address _treasury,
        uint256 _epoch
    ) {
        token = _token;
        treasury = _treasury;
        epoch = _epoch;
        initialTimestamp = block.timestamp;
    }

    function withdrawWithPermit(
        address _to,
        uint256 _amount,
        uint256 _expires,
        bytes memory _signature
    ) public {
        bool isPermitValid = verify(
            owner(),
            _to,
            _amount,
            _expires,
            _signature
        );
        uint256 cantClaimUntil = lastTimeWithdrawn[_to] + epoch;

        require(isPermitValid, "DistributionWizard: Permit invalid");
        require(
            _expires > block.timestamp,
            "DistributionWizard: Permit expired"
        );
        require(
            block.timestamp > cantClaimUntil,
            "DistributionWizard: Can't withdraw yet"
        );

        lastTimeWithdrawn[_to] = block.timestamp;
        token.transferFrom(treasury, _to, _amount);
    }

    function verify(
        address _signer,
        address _to,
        uint256 _amount,
        uint256 _expires,
        bytes memory _signature
    ) public pure returns (bool) {
        bytes32 messageHash = getMessageHash(_to, _amount, _expires);
        bytes32 ethSignedMessageHash = keccak256(
            abi.encodePacked("\x19Ethereum Signed Message:\n32", messageHash)
        );
        (bytes32 r, bytes32 s, uint8 v) = splitSignature(_signature);
        return ecrecover(ethSignedMessageHash, v, r, s) == _signer;
    }

    function getMessageHash(
        address _to,
        uint256 _amount,
        uint256 _expires
    ) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(_to, _amount, _expires));
    }

    function splitSignature(bytes memory sig)
        public
        pure
        returns (
            bytes32 r,
            bytes32 s,
            uint8 v
        )
    {
        require(
            sig.length == 65,
            "DistributionWizard: Invalid signature length"
        );
        assembly {
            // first 32 bytes, after the length prefix
            r := mload(add(sig, 32))
            // second 32 bytes
            s := mload(add(sig, 64))
            // final byte (first byte of the next 32 bytes)
            v := byte(0, mload(add(sig, 96)))
        }
    }

    function setEpoch(uint256 _epoch) public onlyOwner {
        epoch = _epoch;
    }
}
