import { run, network, ethers } from "hardhat";
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
    console.log(`Starting contract verification on ${networkName}...`);

    try {
        const [owner, verifier] = await ethers.getSigners();
        console.log("\nVerifying Vault...");
        await run("verify:verify", {
            address: vaultAddress,
            constructorArguments: [owner.address, verifier.address],
        });
        console.log("Vault verification successful");

        console.log("\nVerifying Points...");
        await run("verify:verify", {
            address: pointsAddress,
            constructorArguments: [owner.address],
        });
        console.log("Points verification successful");
    } catch (error) {
        console.error("Contract verification failed:", error);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
