// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.28;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

contract Vault is Ownable, ReentrancyGuard {
    using SafeERC20 for IERC20;
    using ECDSA for bytes32;

    struct ClaimInfo {
        address token;
        address to;
        uint256 value;
        uint64 deadline;
        string comment;
    }

    address public verifier;
    mapping(bytes32 => bool) public claimedHashes;

    event VerifierUpdated(address indexed oldVerifier, address indexed newVerifier);
    event Claimed(address indexed token, address indexed to, uint256 value, string comment);

    error InvalidAddress();
    error InvalidValue();
    error InvalidSignature();
    error SignatureExpired();
    error AlreadyClaimed();
    error InsufficientBalance();
    error TransferFailed();

    constructor(address _owner, address _verifier) {
        _transferOwnership(_owner);
        _setVerifier(_verifier);
    }

    function getClaimInfoHash(ClaimInfo memory claimInfo) public view returns (bytes32) {
        return
            keccak256(
                abi.encodePacked(
                    claimInfo.token,
                    claimInfo.to,
                    claimInfo.value,
                    claimInfo.deadline,
                    claimInfo.comment,
                    address(this),
                    block.chainid
                )
            );
    }

    function claim(ClaimInfo memory claimInfo, bytes memory signature) external nonReentrant {
        if (claimInfo.to == address(0)) revert InvalidAddress();
        if (claimInfo.value == 0) revert InvalidValue();
        if (claimInfo.deadline != 0 && claimInfo.deadline < block.timestamp) revert SignatureExpired();

        bytes32 claimInfoHash = getClaimInfoHash(claimInfo);
        bytes32 ethSignedMessageHash = claimInfoHash.toEthSignedMessageHash();
        if (claimedHashes[ethSignedMessageHash]) revert AlreadyClaimed();
        address signer = ethSignedMessageHash.recover(signature);
        if (signer != verifier) revert InvalidSignature();

        if (claimInfo.token == address(0)) {
            if (address(this).balance < claimInfo.value) revert InsufficientBalance();
            payable(claimInfo.to).transfer(claimInfo.value);
        } else {
            IERC20 token = IERC20(claimInfo.token);
            if (token.balanceOf(address(this)) < claimInfo.value) revert InsufficientBalance();
            token.safeTransfer(claimInfo.to, claimInfo.value);
        }
        claimedHashes[ethSignedMessageHash] = true;
        emit Claimed(claimInfo.token, claimInfo.to, claimInfo.value, claimInfo.comment);
    }

    function _setVerifier(address newVerifier) internal {
        if (newVerifier == address(0)) revert InvalidAddress();
        address oldVerifier = verifier;
        verifier = newVerifier;
        emit VerifierUpdated(oldVerifier, newVerifier);
    }

    function setVerifier(address newVerifier) external onlyOwner {
        _setVerifier(newVerifier);
    }

    function emergencyWithdraw(address token, address to, uint256 value) external onlyOwner {
        if (token == address(0)) {
            payable(to).transfer(value);
        } else {
            IERC20(token).safeTransfer(to, value);
        }
    }

    receive() external payable {}
}
