// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.28;

import "@openzeppelin/contracts/metatx/ERC2771Context.sol";

contract CheckIn is ERC2771Context {
    struct CheckInInfo {
        uint192 count;
        uint64 lastTime;
    }

    uint256 public checkInTotalCount;
    mapping(address => CheckInInfo) public checkInInfos;

    constructor(address trustedForwarder) ERC2771Context(trustedForwarder) {}

    function userCheckIn() external {
        address sender = _msgSender();
        CheckInInfo storage checkInInfo = checkInInfos[sender];
        uint64 timestamp = uint64(block.timestamp);
        require(timestamp > checkInInfo.lastTime, "Check-in too soon");
        checkInInfo.lastTime = timestamp;
        checkInInfo.count++;
        checkInTotalCount++;
    }
}
