import { ethers, network } from "hardhat";
import * as fs from "fs";
import * as path from "path";

async function main() {
    const addressPath = path.join(__dirname, "../../contracts/vault/deployedAddress.json");
    const addresses = JSON.parse(fs.readFileSync(addressPath, "utf8"));
    const networkName = network.name;
    if (!addresses[networkName]) {
        addresses[networkName] = {};
    }

    const [owner, verifier] = await ethers.getSigners();
    const vaultFactory = await ethers.getContractFactory("Vault");
    const vault = await vaultFactory.deploy(owner.address, verifier.address);
    await vault.waitForDeployment();
    const vaultAddress = await vault.getAddress();
    console.log("Vault deployed to:", vaultAddress);
    console.log("Owner:", owner.address);
    console.log("Verifier:", verifier.address);

    const pointsFactory = await ethers.getContractFactory("Points");
    const points = await pointsFactory.deploy(owner.address);
    await points.waitForDeployment();
    const pointsAddress = await points.getAddress();
    console.log(`\nDeployment Info for network ${networkName}:`);
    console.log("Points deployed to:", pointsAddress);
    console.log("Owner:", owner.address);

    addresses[networkName]["Vault"] = vaultAddress;
    addresses[networkName]["Points"] = pointsAddress;
    fs.writeFileSync(addressPath, JSON.stringify(addresses, null, 2));
    console.log(`\ndeployedAddress.json updated for network ${networkName}`);
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
