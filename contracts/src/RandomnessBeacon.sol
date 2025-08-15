// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract RandomnessBeacon is Ownable {
    struct Randomness {
        uint32 timestamp;
        uint128 randomness;
    }
    event RandomnessPosted(Randomness randomness);

    Randomness[] public randomnesses;

    constructor(address _owner) Ownable(_owner) {}
    
    function postRandomness(uint128 randomness) public onlyOwner {
        randomnesses.push(Randomness({
            timestamp: uint32(block.timestamp),
            randomness: uint128(randomness)
        }));
        emit RandomnessPosted(Randomness({
            timestamp: uint32(block.timestamp),
            randomness: uint128(randomness)
        }));
    }

    function getRandomness(uint256 index) public view returns (Randomness memory) {
        return randomnesses[index];
    }

    function getRandomnessCount() public view returns (uint256) {
        return randomnesses.length;
    }
}