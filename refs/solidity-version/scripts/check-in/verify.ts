import { run, network } from "hardhat";
import * as fs from "fs";
import * as path from "path";

async function main() {
    const addressPath = path.join(__dirname, "../../contracts/check-in/deployedAddress.json");
    const addresses = JSON.parse(fs.readFileSync(addressPath, "utf8"));
    const networkName = network.name;
    const networkAddresses = addresses[networkName];
    if (!networkAddresses) {
        throw new Error(`No addresses found for network ${networkName}`);
    }

    const trustedForwarderAddress = networkAddresses["MinimalForwarder"];
    const checkInAddress = networkAddresses["CheckIn"];
    if (!trustedForwarderAddress) {
        throw new Error(`MinimalForwarder address not found for network ${networkName}`);
    }
    if (!checkInAddress) {
        throw new Error(`CheckIn address not found for network ${networkName}`);
    }
    console.log(`Verifying contracts on ${networkName}...`);
    console.log("CheckIn address:", checkInAddress);

    try {
        await run("verify:verify", {
            address: checkInAddress,
            constructorArguments: [trustedForwarderAddress],
        });
        console.log("Contract verification successful");
    } catch (error) {
        console.error("Contract verification failed", error);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
