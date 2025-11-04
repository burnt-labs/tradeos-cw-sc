import { ethers, network } from "hardhat";
import * as fs from "fs";
import * as path from "path";

async function main() {
    const addressPath = path.join(__dirname, "../../contracts/check-in/deployedAddress.json");
    const addresses = JSON.parse(fs.readFileSync(addressPath, "utf8"));
    const networkName = network.name;
    if (!addresses[networkName]) {
        addresses[networkName] = {};
    }

    const trustedForwarder = addresses[networkName]["MinimalForwarder"];
    if (!trustedForwarder) {
        throw new Error(`MinimalForwarder address not found for network ${networkName}`);
    }
    const checkInFactory = await ethers.getContractFactory("CheckIn");
    const checkIn = await checkInFactory.deploy(trustedForwarder);
    await checkIn.waitForDeployment();
    const checkInAddress = await checkIn.getAddress();
    console.log(`CheckIn contract deployed to ${checkInAddress} on ${networkName}`);
    addresses[networkName]["CheckIn"] = checkInAddress;
    fs.writeFileSync(addressPath, JSON.stringify(addresses, null, 2));
    console.log(`deployedAddress.json updated with CheckIn address for ${networkName}`);
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
