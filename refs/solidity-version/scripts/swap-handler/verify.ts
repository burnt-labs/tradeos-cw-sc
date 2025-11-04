import { run, network } from "hardhat";
import * as fs from "fs";
import * as path from "path";

async function main() {
    const addressPath = path.join(__dirname, "../../contracts/swap-handler/deployedAddress.json");
    const addresses = JSON.parse(fs.readFileSync(addressPath, "utf8"));
    const networkName = network.name;
    const networkAddresses = addresses[networkName];
    if (!networkAddresses) {
        throw new Error(`No addresses found for network ${networkName}`);
    }

    const swapHandlerAddress = networkAddresses["SwapHandlerV2"];
    const routerAddress = networkAddresses["Router"];
    if (!swapHandlerAddress || !routerAddress) {
        throw new Error(`Required addresses not found for network ${networkName}`);
    }
    console.log(`Verifying contracts on ${networkName}...`);
    console.log("SwapHandlerV2 address:", swapHandlerAddress);

    try {
        const [owner] = await ethers.getSigners();
        await run("verify:verify", {
            address: swapHandlerAddress,
            constructorArguments: [owner.address, routerAddress],
        });
        console.log("Contract verification successful");
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
