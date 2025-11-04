import { ethers, network } from "hardhat";
import * as fs from "fs";
import * as path from "path";

async function main() {
    const addressPath = path.join(__dirname, "../../contracts/vault/deployedAddress.json");
    const addresses = JSON.parse(fs.readFileSync(addressPath, "utf8"));
    const networkName = network.name;
    const networkAddresses = addresses[networkName];
    if (!networkAddresses) {
        throw new Error(`No addresses found for network ${networkName}`);
    }
    const vaultAddress = networkAddresses["Vault"];
    const pointsAddress = networkAddresses["Points"];
    if (!vaultAddress || !pointsAddress) {
        throw new Error(`Required addresses not found for network ${networkName}`);
    }

    const [owner, verifier, user] = await ethers.getSigners();
    const vaultFactory = await ethers.getContractFactory("Vault");
    const vault = vaultFactory.attach(vaultAddress);
    const pointsFactory = await ethers.getContractFactory("Points");
    const points = pointsFactory.attach(pointsAddress);
    console.log(`\nTesting on network: ${networkName}`);
    console.log("Vault address:", vaultAddress);
    console.log("Points address:", pointsAddress);

    const ethAmount = ethers.parseEther("0.001");
    console.log("\nTransferring ETH to user...");
    const ethTx1 = await owner.sendTransaction({
        to: user.address,
        value: ethAmount,
    });
    await ethTx1.wait();
    console.log(`Transferred ${ethers.formatEther(ethAmount)} ETH to user: ${user.address}`);

    const mintAmount = ethers.parseEther("10000");
    console.log("\nMinting tokens to Vault contract...");
    const mintTx = await points.mint(vaultAddress, mintAmount);
    await mintTx.wait();
    console.log(`Minted ${ethers.formatEther(mintAmount)} tokens to Vault contract`);

    const decimals = await points.decimals();
    const value = BigInt(1000) * BigInt(10) ** BigInt(decimals);
    const deadline = Math.floor(Date.now() / 1000) + 3600;
    const tokenClaimInfo = {
        token: pointsAddress,
        to: user.address,
        value: value,
        deadline: deadline,
        comment: "Test Token Claim",
    };
    const messageHash1 = await vault.getClaimInfoHash(tokenClaimInfo);
    const signature1 = await verifier.signMessage(ethers.getBytes(messageHash1));
    console.log("\nClaim Info:");
    console.log("Token:", pointsAddress);
    console.log("To:", user.address);
    console.log("Value:", value / BigInt(10) ** BigInt(decimals));
    console.log("Deadline:", new Date(deadline * 1000).toLocaleString());

    console.log("\nAttempting to claim tokens...");
    const balanceBefore1 = await points.balanceOf(user.address);
    const tx1 = await vault.connect(user).claim(tokenClaimInfo, signature1);
    await tx1.wait();
    const balanceAfter1 = await points.balanceOf(user.address);
    console.log("Claim successful!");
    console.log("Balance before:", balanceBefore1 / BigInt(10) ** BigInt(decimals));
    console.log("Balance after:", balanceAfter1 / BigInt(10) ** BigInt(decimals));
    console.log("Claimed amount:", (balanceAfter1 - balanceBefore1) / BigInt(10) ** BigInt(decimals));

    console.log("\nTransferring ETH to Vault contract...");
    const ethTx2 = await owner.sendTransaction({
        to: vaultAddress,
        value: ethAmount,
    });
    await ethTx2.wait();
    console.log(`Transferred ${ethers.formatEther(ethAmount)} ETH to Vault contract: ${vaultAddress}`);
    const ethClaimInfo = {
        token: ethers.ZeroAddress,
        to: user.address,
        value: ethAmount,
        deadline: deadline,
        comment: "Test ETH Claim",
    };
    const messageHash2 = await vault.getClaimInfoHash(ethClaimInfo);
    const signature2 = await verifier.signMessage(ethers.getBytes(messageHash2));
    console.log("\nClaim Info:");
    console.log("Token:", ethers.ZeroAddress);
    console.log("To:", user.address);
    console.log("Value:", ethers.formatEther(ethAmount));
    console.log("Deadline:", new Date(deadline * 1000).toLocaleString());

    console.log("\nAttempting to claim ETH...");
    const balanceBefore2 = await ethers.provider.getBalance(user.address);
    const ethTx = await vault.connect(user).claim(ethClaimInfo, signature2);
    await ethTx.wait();
    const balanceAfter2 = await ethers.provider.getBalance(user.address);
    console.log("ETH Claim successful!");
    console.log("ETH Balance before:", ethers.formatEther(balanceBefore2));
    console.log("ETH Balance after:", ethers.formatEther(balanceAfter2));
    console.log("Claimed amount:", ethers.formatEther(balanceAfter2 - balanceBefore2));
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
